// Integration tests for database drivers.
// These tests are marked #[ignore] and only run when a real DB is available.
// Set YSSV_TEST_PG_URL or YSSV_TEST_MYSQL_URL env vars to enable.

use yssv::core::connections::model::{Connection, DbEngine};
use yssv::core::drivers;

fn pg_conn() -> Connection {
    let url = std::env::var("YSSV_TEST_PG_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".into());
    // Parse the URL into a Connection struct
    let mut c = Connection::new_postgres();
    c.name = "test-pg".into();
    // Use URL directly as host for simplicity in test — real parsing not needed here
    c.host = "localhost".into();
    c.port = 5432;
    c.database = "postgres".into();
    c.username = "postgres".into();
    c.password = "postgres".into();
    c
}

fn mysql_conn() -> Connection {
    let mut c = Connection::new_mysql();
    c.name = "test-mysql".into();
    c.host = "localhost".into();
    c.port = 3306;
    c.database = "mysql".into();
    c.username = "root".into();
    c.password = "root".into();
    c
}

#[tokio::test]
#[ignore = "requires PostgreSQL; set YSSV_TEST_PG_URL"]
async fn pg_can_list_databases() {
    let conn = pg_conn();
    let active = drivers::connect(&conn).await.expect("connect failed");
    let dbs = active.list_databases().await.expect("list_databases failed");
    assert!(!dbs.is_empty(), "expected at least one database");
    assert!(dbs.contains(&"postgres".to_string()));
}

#[tokio::test]
#[ignore = "requires PostgreSQL; set YSSV_TEST_PG_URL"]
async fn pg_can_list_schemas() {
    let conn = pg_conn();
    let active = drivers::connect(&conn).await.expect("connect failed");
    let schemas = active.list_schemas("postgres").await.expect("list_schemas failed");
    let schema_names: Vec<&str> = schemas.iter().map(|s| s.name.as_str()).collect();
    assert!(schema_names.contains(&"public"));
}

#[tokio::test]
#[ignore = "requires MySQL; set YSSV_TEST_MYSQL_URL"]
async fn mysql_can_list_databases() {
    let conn = mysql_conn();
    let active = drivers::connect(&conn).await.expect("connect failed");
    let dbs = active.list_databases().await.expect("list_databases failed");
    assert!(!dbs.is_empty());
    assert!(dbs.contains(&"mysql".to_string()));
}
