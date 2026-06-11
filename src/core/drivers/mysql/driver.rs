use crate::core::{
    connections::model::Connection,
    drivers::{ActiveConnection, DbError},
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo, TableKind},
};
use async_trait::async_trait;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

pub struct MyConnection {
    pool: MySqlPool,
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    tracing::debug!(host = %conn.host, port = conn.port, db = %conn.database, "mysql: connecting");
    let url = format!(
        "mysql://{}:{}@{}:{}/{}?charset=utf8mb4",
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
    async fn current_database(&self) -> Result<String, DbError> {
        let db = sqlx::query_scalar::<_, String>("SELECT DATABASE()")
            .fetch_one(&self.pool)
            .await?;
        tracing::debug!(db = %db, "mysql: current_database");
        Ok(db)
    }

    async fn list_databases(&self) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query_scalar::<_, String>("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await?;
        tracing::debug!(count = rows.len(), "mysql: list_databases");
        Ok(rows)
    }

    async fn list_schemas(&self, db: &str) -> Result<Vec<SchemaInfo>, DbError> {
        tracing::debug!(db, "mysql: list_schemas (db is schema)");
        let tables = self.list_tables(db, db).await?;
        Ok(vec![SchemaInfo {
            name: db.to_string(),
            tables,
        }])
    }

    async fn list_tables(&self, _db: &str, schema: &str) -> Result<Vec<TableInfo>, DbError> {
        tracing::debug!(schema, "mysql: list_tables");
        let rows = sqlx::query_as::<_, (String, String, Option<i64>)>(
            "SELECT table_name, table_type, table_rows
             FROM information_schema.tables
             WHERE table_schema = ?
             ORDER BY table_name",
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
        tracing::debug!(schema, count = tables.len(), "mysql: list_tables done");
        Ok(tables)
    }

    async fn fetch_rows(
        &self,
        db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError> {
        tracing::debug!(schema, table, limit, offset, "mysql: fetch_rows");

        let col_defs = self.describe_table(db, schema, table).await?;

        let select_list = if col_defs.is_empty() {
            "*".to_string()
        } else {
            col_defs
                .iter()
                .map(|c| {
                    let name = &c.name;
                    if is_temporal(&c.data_type) {
                        format!("CAST(`{name}` AS CHAR) AS `{name}`")
                    } else {
                        format!("`{name}`")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let query =
            format!("SELECT {select_list} FROM `{schema}`.`{table}` LIMIT {limit} OFFSET {offset}");
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: col_defs,
                rows: vec![],
                total_rows: Some(0),
            });
        }

        let data_rows: Vec<Vec<Option<String>>> = rows
            .iter()
            .map(|row| {
                (0..col_defs.len())
                    .map(|i| decode_col_mysql(row, i))
                    .collect()
            })
            .collect();

        let count_query = format!("SELECT COUNT(*) FROM `{schema}`.`{table}`");
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(schema, table, error = %e, "mysql: COUNT(*) failed, using 0");
                0
            });

        Ok(QueryResult {
            columns: col_defs,
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
        tracing::debug!(schema, table, "mysql: describe_table");
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
            "SELECT column_name, data_type, is_nullable, column_key
             FROM information_schema.columns
             WHERE table_schema = ? AND table_name = ?
             ORDER BY ordinal_position",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let cols: Vec<ColumnDef> = rows
            .into_iter()
            .map(|(name, data_type, nullable, key)| ColumnDef {
                name,
                data_type,
                is_pk: key.as_deref() == Some("PRI"),
                is_fk: key.as_deref() == Some("MUL"),
                nullable: nullable == "YES",
            })
            .collect();
        tracing::debug!(
            schema,
            table,
            columns = cols.len(),
            "mysql: describe_table done"
        );
        Ok(cols)
    }
}

fn is_temporal(data_type: &str) -> bool {
    matches!(
        data_type.to_lowercase().as_str(),
        "datetime" | "timestamp" | "date" | "time" | "year"
    )
}

fn decode_col_mysql(row: &sqlx::mysql::MySqlRow, i: usize) -> Option<String> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<Option<String>, _>(i) {
        return v;
    }
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(i) {
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
