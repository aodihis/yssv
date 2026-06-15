use crate::core::{
    connections::model::Connection,
    drivers::{ActiveConnection, DbError, table_type_to_kind},
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo},
};
use async_trait::async_trait;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

pub struct MyConnection {
    pool: MySqlPool,
}

pub(crate) fn build_connection_url(conn: &Connection) -> String {
    format!(
        "mysql://{}:{}@{}:{}/{}?charset=utf8mb4",
        conn.username, conn.password, conn.host, conn.port, conn.database
    )
}

pub(crate) fn build_select_list(cols: &[ColumnDef]) -> String {
    if cols.is_empty() {
        return "*".to_string();
    }
    cols.iter()
        .map(|c| {
            let q = quote_ident(&c.name);
            if is_temporal(&c.data_type) {
                format!("CAST({} AS CHAR) AS {}", q, q)
            } else {
                q
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    tracing::debug!(host = %conn.host, port = conn.port, db = %conn.database, "mysql: connecting");
    let url = build_connection_url(conn);
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(DbError::from)?;
    tracing::info!(host = %conn.host, port = conn.port, "mysql: connection pool established");
    Ok(Box::new(MyConnection { pool }))
}

pub(crate) fn map_column(
    name: String,
    data_type: String,
    nullable: String,
    key: Option<String>,
) -> ColumnDef {
    ColumnDef {
        is_pk: key.as_deref() == Some("PRI"),
        is_fk: key.as_deref() == Some("MUL"),
        nullable: nullable == "YES",
        name,
        data_type,
    }
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
        let rows = sqlx::query_as::<_, (String, String, Option<u64>)>(
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
            .map(|(name, ttype, row_count)| TableInfo {
                kind: table_type_to_kind(&ttype),
                name,
                row_count,
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

        let select_list = build_select_list(&col_defs);

        let fetch_sql = format!(
            "SELECT {} FROM {}.{} LIMIT ? OFFSET ?",
            select_list,
            quote_ident(schema),
            quote_ident(table),
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(fetch_sql))
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

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

        let count_sql = format!(
            "SELECT COUNT(*) FROM {}.{}",
            quote_ident(schema),
            quote_ident(table),
        );
        let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(count_sql))
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
            .map(|(name, data_type, nullable, key)| map_column(name, data_type, nullable, key))
            .collect();
        tracing::debug!(
            schema,
            table,
            columns = cols.len(),
            "mysql: describe_table done"
        );
        Ok(cols)
    }

    async fn execute_single(&self, sql: &str) -> Result<QueryResult, DbError> {
        use sqlx::{Column, Row, TypeInfo};
        tracing::debug!(sql_len = sql.len(), "mysql: execute_single");

        let trimmed = sql.trim().to_uppercase();
        let is_fetch = trimmed.starts_with("SELECT")
            || trimmed.starts_with("WITH")
            || trimmed.starts_with("SHOW")
            || trimmed.starts_with("EXPLAIN");

        if !is_fetch {
            let result = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(&self.pool)
                .await?;
            let affected = result.rows_affected();
            tracing::info!(rows_affected = affected, "mysql: execute_single (DML)");
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

        let rows = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()))
            .fetch_all(&self.pool)
            .await?;
        if rows.is_empty() {
            tracing::debug!("mysql: execute_single returned 0 rows");
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
                    .map(|i| decode_col_mysql(row, i))
                    .collect()
            })
            .collect();

        let row_count = data_rows.len() as u64;
        tracing::info!(rows = row_count, "mysql: execute_single success");

        Ok(QueryResult {
            columns,
            rows: data_rows,
            total_rows: Some(row_count),
        })
    }

    async fn execute_batch(&self, statements: &[String]) -> Result<u64, DbError> {
        tracing::debug!(
            count = statements.len(),
            "mysql: execute_batch (transaction begin)"
        );
        let mut tx = self.pool.begin().await?;
        let mut affected = 0u64;
        for stmt in statements {
            tracing::debug!(stmt = %stmt, "mysql: execute_batch statement");
            let result = sqlx::query(sqlx::AssertSqlSafe(stmt.to_owned()))
                .execute(&mut *tx)
                .await?;
            affected += result.rows_affected();
        }
        tx.commit().await?;
        tracing::info!(
            count = statements.len(),
            affected,
            "mysql: execute_batch committed"
        );
        Ok(affected)
    }
}

fn decode_col_mysql(row: &sqlx::mysql::MySqlRow, i: usize) -> Option<String> {
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
    if let Ok(Some(v)) = row.try_get::<Option<i8>, _>(i) {
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

fn quote_ident(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

fn is_temporal(data_type: &str) -> bool {
    matches!(
        data_type.to_lowercase().as_str(),
        "datetime" | "timestamp" | "date" | "time" | "year"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{connections::model::Connection, results::model::ColumnDef};

    #[test]
    fn quote_ident_wraps_in_backticks() {
        assert_eq!(quote_ident("table"), "`table`");
    }

    #[test]
    fn quote_ident_escapes_embedded_backtick() {
        assert_eq!(quote_ident("my`table"), "`my``table`");
    }

    #[test]
    fn build_connection_url_format() {
        let mut conn = Connection::new_mysql();
        conn.username = "bob".into();
        conn.password = "pass".into();
        conn.host = "mysql.host".into();
        conn.port = 3307;
        conn.database = "shop".into();
        let url = build_connection_url(&conn);
        assert_eq!(url, "mysql://bob:pass@mysql.host:3307/shop?charset=utf8mb4");
    }

    #[test]
    fn build_select_list_empty_returns_star() {
        assert_eq!(build_select_list(&[]), "*");
    }

    #[test]
    fn build_select_list_non_temporal_uses_backtick() {
        let cols = vec![
            ColumnDef {
                name: "id".into(),
                data_type: "int".into(),
                is_pk: true,
                is_fk: false,
                nullable: false,
            },
            ColumnDef {
                name: "name".into(),
                data_type: "varchar".into(),
                is_pk: false,
                is_fk: false,
                nullable: true,
            },
        ];
        let list = build_select_list(&cols);
        assert_eq!(list, "`id`, `name`");
    }

    #[test]
    fn build_select_list_temporal_uses_cast() {
        let cols = vec![ColumnDef {
            name: "created_at".into(),
            data_type: "datetime".into(),
            is_pk: false,
            is_fk: false,
            nullable: true,
        }];
        let list = build_select_list(&cols);
        assert_eq!(list, "CAST(`created_at` AS CHAR) AS `created_at`");
    }

    #[test]
    fn build_select_list_mixed_temporal_and_regular() {
        let cols = vec![
            ColumnDef {
                name: "id".into(),
                data_type: "int".into(),
                is_pk: true,
                is_fk: false,
                nullable: false,
            },
            ColumnDef {
                name: "ts".into(),
                data_type: "timestamp".into(),
                is_pk: false,
                is_fk: false,
                nullable: true,
            },
        ];
        let list = build_select_list(&cols);
        assert_eq!(list, "`id`, CAST(`ts` AS CHAR) AS `ts`");
    }

    #[test]
    fn is_temporal_matches_all_date_types() {
        for t in &[
            "datetime",
            "DATETIME",
            "timestamp",
            "TIMESTAMP",
            "date",
            "time",
            "year",
        ] {
            assert!(is_temporal(t), "{t} should be temporal");
        }
    }

    #[test]
    fn is_temporal_rejects_non_date_types() {
        for t in &["int", "varchar", "text", "float", "json"] {
            assert!(!is_temporal(t), "{t} should not be temporal");
        }
    }

    #[test]
    fn map_column_pri_key_sets_is_pk() {
        let col = map_column("id".into(), "int".into(), "NO".into(), Some("PRI".into()));
        assert!(col.is_pk);
        assert!(!col.is_fk);
        assert!(!col.nullable);
    }

    #[test]
    fn map_column_mul_key_sets_is_fk() {
        let col = map_column(
            "user_id".into(),
            "int".into(),
            "NO".into(),
            Some("MUL".into()),
        );
        assert!(!col.is_pk);
        assert!(col.is_fk);
    }

    #[test]
    fn map_column_no_key_clears_pk_and_fk() {
        let col = map_column("notes".into(), "text".into(), "YES".into(), None);
        assert!(!col.is_pk);
        assert!(!col.is_fk);
        assert!(col.nullable);
    }

    #[test]
    fn map_column_nullable_yes_sets_nullable() {
        let col = map_column("bio".into(), "text".into(), "YES".into(), None);
        assert!(col.nullable);
    }

    #[test]
    fn map_column_nullable_no_clears_nullable() {
        let col = map_column("email".into(), "varchar".into(), "NO".into(), None);
        assert!(!col.nullable);
    }
}
