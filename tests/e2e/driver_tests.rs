// E2E driver tests using testcontainers — requires Docker.
// Each test spins up a real database container, runs assertions, then tears it down.

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{mysql::Mysql, postgres::Postgres};
use yssv::core::{connections::model::Connection, drivers};

// ---------------------------------------------------------------------------
// PostgreSQL
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_list_databases_contains_postgres() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let mut conn = Connection::new_postgres();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "postgres".into();
    conn.username = "postgres".into();
    conn.password = "postgres".into();

    let active = drivers::connect(&conn).await.expect("pg connect failed");
    let dbs = active
        .list_databases()
        .await
        .expect("list_databases failed");
    assert!(!dbs.is_empty());
    assert!(dbs.contains(&"postgres".to_string()));
}

#[tokio::test]
async fn pg_list_schemas_contains_public() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let mut conn = Connection::new_postgres();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "postgres".into();
    conn.username = "postgres".into();
    conn.password = "postgres".into();

    let active = drivers::connect(&conn).await.expect("pg connect failed");
    let schemas = active
        .list_schemas("postgres")
        .await
        .expect("list_schemas failed");
    let names: Vec<&str> = schemas.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"public"));
}

#[tokio::test]
async fn pg_fetch_rows_from_information_schema() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let mut conn = Connection::new_postgres();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "postgres".into();
    conn.username = "postgres".into();
    conn.password = "postgres".into();

    let active = drivers::connect(&conn).await.expect("pg connect failed");
    let tables = active
        .list_tables("postgres", "information_schema")
        .await
        .expect("list_tables failed");
    assert!(!tables.is_empty(), "information_schema should have tables");
}

// ---------------------------------------------------------------------------
// MySQL
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mysql_list_databases_contains_mysql() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();

    let mut conn = Connection::new_mysql();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "mysql".into();
    conn.username = "root".into();
    conn.password = String::new();

    let active = drivers::connect(&conn).await.expect("mysql connect failed");
    let dbs = active
        .list_databases()
        .await
        .expect("list_databases failed");
    assert!(!dbs.is_empty());
    assert!(dbs.contains(&"mysql".to_string()));
}

#[tokio::test]
async fn mysql_list_schemas_returns_results() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();

    let mut conn = Connection::new_mysql();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "mysql".into();
    conn.username = "root".into();
    conn.password = String::new();

    let active = drivers::connect(&conn).await.expect("mysql connect failed");
    let schemas = active
        .list_schemas("mysql")
        .await
        .expect("list_schemas failed");
    assert!(!schemas.is_empty());
}
