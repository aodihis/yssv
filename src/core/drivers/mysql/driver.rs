use async_trait::async_trait;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;
use crate::core::{
    connections::model::Connection,
    drivers::{ActiveConnection, DbError},
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo, TableKind},
};

pub struct MyConnection {
    pool: MySqlPool,
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    tracing::debug!(host = %conn.host, port = conn.port, db = %conn.database, "mysql: connecting");
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        conn.username, conn.password, conn.host, conn.port, conn.database
    );
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(DbError::from)?;
    tracing::info!(host = %conn.host, port = conn.port, "mysql: connection pool established");
    Ok(Box::new(MyConnection { pool }))
}

#[async_trait]
impl ActiveConnection for MyConnection {
    async fn list_databases(&self) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query_scalar::<_, String>("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn list_schemas(&self, db: &str) -> Result<Vec<SchemaInfo>, DbError> {
        // MySQL has no schemas; the database itself is the schema.
        let tables = self.list_tables(db, db).await?;
        Ok(vec![SchemaInfo { name: db.to_string(), tables }])
    }

    async fn list_tables(&self, _db: &str, schema: &str) -> Result<Vec<TableInfo>, DbError> {
        let rows = sqlx::query_as::<_, (String, String, Option<i64>)>(
            "SELECT table_name, table_type, table_rows
             FROM information_schema.tables
             WHERE table_schema = ?
             ORDER BY table_name"
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(name, ttype, row_count)| {
                let kind = if ttype == "VIEW" { TableKind::View } else { TableKind::Table };
                TableInfo {
                    name,
                    kind,
                    row_count: row_count.map(|n| n as u64),
                }
            })
            .collect())
    }

    async fn fetch_rows(
        &self,
        _db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError> {
        tracing::debug!(schema, table, limit, offset, "mysql: fetch_rows");
        let query = format!(
            "SELECT * FROM `{schema}`.`{table}` LIMIT {limit} OFFSET {offset}"
        );
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

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
                    .map(|i| {
                        row.try_get::<Option<String>, _>(i)
                            .ok()
                            .flatten()
                    })
                    .collect()
            })
            .collect();

        let count_query = format!("SELECT COUNT(*) FROM `{schema}`.`{table}`");
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
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
            "SELECT column_name, data_type, is_nullable, column_key
             FROM information_schema.columns
             WHERE table_schema = ? AND table_name = ?
             ORDER BY ordinal_position"
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(name, data_type, nullable, key)| ColumnDef {
                name,
                data_type,
                is_pk: key.as_deref() == Some("PRI"),
                is_fk: key.as_deref() == Some("MUL"),
                nullable: nullable == "YES",
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires a running MySQL instance; set YSSV_TEST_MYSQL_URL to enable"]
    async fn integration_list_databases() {
        let url = std::env::var("YSSV_TEST_MYSQL_URL")
            .unwrap_or_else(|_| "mysql://root:root@localhost/mysql".into());
        let pool = MySqlPool::connect(&url).await.unwrap();
        let my = MyConnection { pool };
        let dbs = my.list_databases().await.unwrap();
        assert!(!dbs.is_empty());
    }
}
