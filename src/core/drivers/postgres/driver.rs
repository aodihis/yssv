use crate::core::{
    connections::model::Connection,
    drivers::{ActiveConnection, DbError},
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo, TableKind},
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct PgConnection {
    pool: PgPool,
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    tracing::debug!(host = %conn.host, port = conn.port, db = %conn.database, "postgres: connecting");
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        conn.username, conn.password, conn.host, conn.port, conn.database
    );
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(DbError::from)?;
    tracing::info!(host = %conn.host, port = conn.port, "postgres: connection pool established");
    Ok(Box::new(PgConnection { pool }))
}

#[async_trait]
impl ActiveConnection for PgConnection {
    async fn list_databases(&self) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname",
        )
        .fetch_all(&self.pool)
        .await?;
        tracing::debug!(count = rows.len(), "postgres: list_databases");
        Ok(rows)
    }

    async fn list_schemas(&self, db: &str) -> Result<Vec<SchemaInfo>, DbError> {
        tracing::debug!(db, "postgres: list_schemas");
        let schema_names = sqlx::query_scalar::<_, String>(
            "SELECT schema_name FROM information_schema.schemata
             WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
             ORDER BY schema_name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut schemas = Vec::new();
        for name in schema_names {
            let tables = self.list_tables(db, &name).await?;
            schemas.push(SchemaInfo { name, tables });
        }
        tracing::debug!(db, count = schemas.len(), "postgres: list_schemas done");
        Ok(schemas)
    }

    async fn list_tables(&self, _db: &str, schema: &str) -> Result<Vec<TableInfo>, DbError> {
        tracing::debug!(schema, "postgres: list_tables");
        let rows = sqlx::query_as::<_, (String, String, Option<i64>)>(
            "SELECT t.table_name, t.table_type,
                    CASE WHEN c.reltuples < 0 THEN NULL ELSE c.reltuples::bigint END
             FROM information_schema.tables t
             LEFT JOIN pg_namespace n ON n.nspname = t.table_schema
             LEFT JOIN pg_class c ON c.relname = t.table_name AND c.relnamespace = n.oid
             WHERE t.table_schema = $1
             ORDER BY t.table_name",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let tables: Vec<TableInfo> = rows
            .into_iter()
            .map(|(name, ttype, row_count)| {
                let kind = if ttype == "VIEW" {
                    TableKind::View
                } else {
                    TableKind::Table
                };
                TableInfo {
                    name,
                    kind,
                    row_count: row_count.map(|n| n as u64),
                }
            })
            .collect();
        tracing::debug!(schema, count = tables.len(), "postgres: list_tables done");
        Ok(tables)
    }

    async fn fetch_rows(
        &self,
        _db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError> {
        tracing::debug!(schema, table, limit, offset, "postgres: fetch_rows");
        let query = format!("SELECT * FROM \"{schema}\".\"{table}\" LIMIT {limit} OFFSET {offset}");
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        if rows.is_empty() {
            return Ok(QueryResult::empty());
        }

        use sqlx::Column;
        use sqlx::Row;
        use sqlx::TypeInfo;

        let columns: Vec<ColumnDef> = rows[0]
            .columns()
            .iter()
            .map(|c| ColumnDef {
                name: c.name().to_string(),
                data_type: c.type_info().name().to_string(),
                is_pk: false,
                is_fk: false,
                nullable: true,
            })
            .collect();

        let data_rows: Vec<Vec<Option<String>>> = rows
            .iter()
            .map(|row| {
                (0..columns.len())
                    .map(|i| row.try_get::<Option<String>, _>(i).ok().flatten())
                    .collect()
            })
            .collect();

        // Get total count
        let count_query = format!("SELECT COUNT(*) FROM \"{schema}\".\"{table}\"");
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        Ok(QueryResult {
            columns,
            rows: data_rows,
            total_rows: Some(total as u64),
        })
    }

    async fn describe_table(
        &self,
        _db: &str,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnDef>, DbError> {
        tracing::debug!(schema, table, "postgres: describe_table");
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
            "SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                (SELECT 'PK' FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage kcu
                   ON tc.constraint_name = kcu.constraint_name
                  AND tc.table_schema = kcu.table_schema
                 WHERE tc.constraint_type = 'PRIMARY KEY'
                   AND tc.table_schema = c.table_schema
                   AND tc.table_name = c.table_name
                   AND kcu.column_name = c.column_name
                 LIMIT 1) AS pk_flag
             FROM information_schema.columns c
             WHERE c.table_schema = $1 AND c.table_name = $2
             ORDER BY c.ordinal_position",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let cols: Vec<ColumnDef> = rows
            .into_iter()
            .map(|(name, data_type, nullable, pk_flag)| ColumnDef {
                name,
                data_type,
                is_pk: pk_flag.is_some(),
                is_fk: false,
                nullable: nullable == "YES",
            })
            .collect();
        tracing::debug!(schema, table, columns = cols.len(), "postgres: describe_table done");
        Ok(cols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance; set YSSV_TEST_PG_URL to enable"]
    async fn integration_list_databases() {
        let url = std::env::var("YSSV_TEST_PG_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".into());
        let pool = PgPool::connect(&url).await.unwrap();
        let pg = PgConnection { pool };
        let dbs = pg.list_databases().await.unwrap();
        assert!(!dbs.is_empty());
    }
}
