use crate::core::{
    connections::model::Connection,
    drivers::{ActiveConnection, DbError, table_type_to_kind},
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo},
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct PgConnection {
    pool: PgPool,
}

pub(crate) fn build_connection_url(conn: &Connection) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        conn.username, conn.password, conn.host, conn.port, conn.database
    )
}

pub(crate) fn build_select_list(cols: &[ColumnDef]) -> String {
    if cols.is_empty() {
        return "*".to_string();
    }
    cols.iter()
        .map(|c| format!("\"{}\"::text AS \"{}\"", c.name, c.name))
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    tracing::debug!(host = %conn.host, port = conn.port, db = %conn.database, "postgres: connecting");
    let url = build_connection_url(conn);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(DbError::from)?;
    tracing::info!(host = %conn.host, port = conn.port, "postgres: connection pool established");
    Ok(Box::new(PgConnection { pool }))
}

pub(crate) fn map_column(
    name: String,
    data_type: String,
    nullable: String,
    pk_flag: Option<String>,
) -> ColumnDef {
    ColumnDef {
        name,
        data_type,
        is_pk: pk_flag.is_some(),
        is_fk: false,
        nullable: nullable == "YES",
    }
}

#[async_trait]
impl ActiveConnection for PgConnection {
    async fn current_database(&self) -> Result<String, DbError> {
        let db = sqlx::query_scalar::<_, String>("SELECT current_database()")
            .fetch_one(&self.pool)
            .await?;
        tracing::debug!(db = %db, "postgres: current_database");
        Ok(db)
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
            .map(|(name, data_type, nullable, pk_flag)| map_column(name, data_type, nullable, pk_flag))
            .collect();
        tracing::debug!(
            schema,
            table,
            columns = cols.len(),
            "postgres: describe_table done"
        );
        Ok(cols)
    }

    async fn fetch_rows(
        &self,
        db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError> {
        tracing::debug!(schema, table, limit, offset, "postgres: fetch_rows");

        let col_defs = self.describe_table(db, schema, table).await?;

        let select_list = build_select_list(&col_defs);

        let query = format!(
            "SELECT {select_list} FROM \"{schema}\".\"{table}\" LIMIT {limit} OFFSET {offset}"
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: col_defs,
                rows: vec![],
                total_rows: Some(0),
            });
        }

        use sqlx::Row;
        let data_rows: Vec<Vec<Option<String>>> = rows
            .iter()
            .map(|row| {
                (0..col_defs.len())
                    .map(|i| row.try_get::<Option<String>, _>(i).ok().flatten())
                    .collect()
            })
            .collect();

        let count_query = format!("SELECT COUNT(*) FROM \"{schema}\".\"{table}\"");
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(schema, table, error = %e, "postgres: COUNT(*) failed, using 0");
                0
            });

        Ok(QueryResult {
            columns: col_defs,
            rows: data_rows,
            total_rows: Some(total as u64),
        })
    }

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
            "SELECT t.table_name,
                    t.table_type,
                    (SELECT CASE WHEN c.reltuples < 0 THEN NULL
                                 ELSE c.reltuples::bigint END
                     FROM pg_class c
                     JOIN pg_namespace n ON c.relnamespace = n.oid
                     WHERE n.nspname = t.table_schema
                       AND c.relname = t.table_name
                       AND c.relkind IN ('r','v','m','p','f')
                     LIMIT 1) AS row_count
             FROM information_schema.tables t
             WHERE t.table_schema = $1
             ORDER BY t.table_name",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let tables: Vec<TableInfo> = rows
            .into_iter()
            .map(|(name, ttype, row_count)| TableInfo {
                kind: table_type_to_kind(&ttype),
                name,
                row_count: row_count.map(|n| n as u64),
            })
            .collect();
        tracing::debug!(schema, count = tables.len(), "postgres: list_tables done");
        Ok(tables)
    }

    async fn execute_single(&self, sql: &str) -> Result<QueryResult, DbError> {
        use sqlx::{Column, Row, TypeInfo};
        tracing::debug!(sql_len = sql.len(), "postgres: execute_single");

        let trimmed = sql.trim().to_uppercase();
        let is_fetch = trimmed.starts_with("SELECT")
            || trimmed.starts_with("WITH")
            || trimmed.starts_with("SHOW")
            || trimmed.starts_with("EXPLAIN")
            || trimmed.starts_with("TABLE");

        if !is_fetch {
            let result = sqlx::query(sql).execute(&self.pool).await?;
            let affected = result.rows_affected();
            tracing::info!(rows_affected = affected, "postgres: execute_single (DML)");
            return Ok(QueryResult {
                columns: vec![ColumnDef {
                    name: "result".into(),
                    data_type: "text".into(),
                    is_pk: false,
                    is_fk: false,
                    nullable: false,
                }],
                rows: vec![vec![Some(format!("Query OK, {affected} rows affected"))]],
                total_rows: Some(1),
            });
        }

        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        if rows.is_empty() {
            tracing::debug!("postgres: execute_single returned 0 rows");
            return Ok(QueryResult::empty());
        }

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
                    .map(|i| decode_col_pg(row, i))
                    .collect()
            })
            .collect();

        let row_count = data_rows.len() as u64;
        tracing::info!(rows = row_count, "postgres: execute_single success");

        Ok(QueryResult {
            columns,
            rows: data_rows,
            total_rows: Some(row_count),
        })
    }
}

fn decode_col_pg(row: &sqlx::postgres::PgRow, i: usize) -> Option<String> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<Option<String>, _>(i) {
        return v;
    }
    if let Ok(Some(v)) = row.try_get::<Option<i32>, _>(i) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(i) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<i16>, _>(i) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<f32>, _>(i) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(i) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<bool>, _>(i) {
        return Some(v.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{connections::model::Connection, results::model::ColumnDef};

    #[test]
    fn build_connection_url_format() {
        let mut conn = Connection::new_postgres();
        conn.username = "alice".into();
        conn.password = "secret".into();
        conn.host = "db.example.com".into();
        conn.port = 5433;
        conn.database = "mydb".into();
        let url = build_connection_url(&conn);
        assert_eq!(url, "postgres://alice:secret@db.example.com:5433/mydb");
    }

    #[test]
    fn build_select_list_empty_cols_returns_star() {
        assert_eq!(build_select_list(&[]), "*");
    }

    #[test]
    fn build_select_list_casts_each_column_to_text() {
        let cols = vec![
            ColumnDef { name: "id".into(), data_type: "int4".into(), is_pk: true, is_fk: false, nullable: false },
            ColumnDef { name: "name".into(), data_type: "text".into(), is_pk: false, is_fk: false, nullable: true },
        ];
        let list = build_select_list(&cols);
        assert_eq!(list, r#""id"::text AS "id", "name"::text AS "name""#);
    }

    #[test]
    fn map_column_pk_flag_some_sets_is_pk() {
        let col = map_column("id".into(), "int4".into(), "NO".into(), Some("PK".into()));
        assert!(col.is_pk);
        assert!(!col.is_fk);
        assert!(!col.nullable);
    }

    #[test]
    fn map_column_pk_flag_none_clears_is_pk() {
        let col = map_column("email".into(), "text".into(), "YES".into(), None);
        assert!(!col.is_pk);
        assert!(col.nullable);
    }

    #[test]
    fn map_column_nullable_yes_sets_nullable() {
        let col = map_column("notes".into(), "text".into(), "YES".into(), None);
        assert!(col.nullable);
    }

    #[test]
    fn map_column_nullable_no_clears_nullable() {
        let col = map_column("code".into(), "text".into(), "NO".into(), None);
        assert!(!col.nullable);
    }

}
