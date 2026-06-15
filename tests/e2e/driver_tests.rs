// E2E driver tests using testcontainers — requires Docker.
// Each test spins up a real database container, runs assertions, then tears it down.
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{mysql::Mysql, postgres::Postgres};
use yssv::core::{connections::model::Connection, drivers};

fn pg_conn_config(port: u16) -> Connection {
    let mut conn = Connection::new_postgres();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "postgres".into();
    conn.username = "postgres".into();
    conn.password = "postgres".into();
    conn
}

fn mysql_conn_config(port: u16) -> Connection {
    let mut conn = Connection::new_mysql();
    conn.host = "127.0.0.1".into();
    conn.port = port;
    conn.database = "mysql".into();
    conn.username = "root".into();
    conn.password = String::new();
    conn
}

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
    println!("port: {}", port);
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

// ---------------------------------------------------------------------------
// PostgreSQL — extended coverage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_current_database_returns_postgres() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();
    assert_eq!(active.current_database().await.unwrap(), "postgres");
}

#[tokio::test]
async fn pg_execute_single_select_returns_columns_and_row() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();

    let result = active.execute_single("SELECT 42 AS answer").await.unwrap();
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "answer");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Some("42".into()));
}

#[tokio::test]
async fn pg_execute_single_dml_returns_affected_message() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE yssv_dml_pg (id INT)")
        .await
        .unwrap();
    let result = active
        .execute_single("INSERT INTO yssv_dml_pg VALUES (1), (2)")
        .await
        .unwrap();
    assert_eq!(result.columns[0].name, "result");
    let msg = result.rows[0][0].as_deref().unwrap();
    assert!(msg.contains("rows affected"), "unexpected message: {msg}");
}

#[tokio::test]
async fn pg_execute_batch_commits_and_returns_affected() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE yssv_batch_pg (id INT)")
        .await
        .unwrap();

    let stmts = vec![
        "INSERT INTO yssv_batch_pg VALUES (1)".into(),
        "INSERT INTO yssv_batch_pg VALUES (2)".into(),
        "INSERT INTO yssv_batch_pg VALUES (3)".into(),
    ];
    let affected = active.execute_batch(&stmts).await.unwrap();
    assert_eq!(affected, 3);

    let count_result = active
        .execute_single("SELECT COUNT(*) FROM yssv_batch_pg")
        .await
        .unwrap();
    assert_eq!(count_result.rows[0][0], Some("3".into()));
}

#[tokio::test]
async fn pg_describe_table_returns_column_metadata() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();

    active
        .execute_single(
            "CREATE TABLE yssv_desc_pg (id INT PRIMARY KEY, name TEXT NOT NULL, bio TEXT)",
        )
        .await
        .unwrap();

    let cols = active
        .describe_table("postgres", "public", "yssv_desc_pg")
        .await
        .unwrap();
    assert_eq!(cols.len(), 3);

    let id = cols.iter().find(|c| c.name == "id").unwrap();
    assert!(id.is_pk, "id should be PK");
    assert!(!id.nullable);

    let name = cols.iter().find(|c| c.name == "name").unwrap();
    assert!(!name.is_pk);
    assert!(!name.nullable);

    let bio = cols.iter().find(|c| c.name == "bio").unwrap();
    assert!(bio.nullable);
}

#[tokio::test]
async fn pg_fetch_rows_returns_inserted_data() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let active = drivers::connect(&pg_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE yssv_fetch_pg (id INT, val TEXT)")
        .await
        .unwrap();
    active
        .execute_single("INSERT INTO yssv_fetch_pg VALUES (1, 'hello'), (2, 'world')")
        .await
        .unwrap();

    let result = active
        .fetch_rows("postgres", "public", "yssv_fetch_pg", 10, 0)
        .await
        .unwrap();
    assert_eq!(result.columns.len(), 2);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.total_rows, Some(2));
}

// ---------------------------------------------------------------------------
// MySQL — extended coverage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mysql_current_database_returns_mysql() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();
    assert_eq!(active.current_database().await.unwrap(), "mysql");
}

#[tokio::test]
async fn mysql_execute_single_select_returns_columns_and_row() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();

    let result = active
        .execute_single("SELECT 'hello' AS greeting")
        .await
        .unwrap();
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "greeting");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Some("hello".into()));
}

#[tokio::test]
async fn mysql_execute_single_dml_returns_affected_message() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE mysql.yssv_dml_my (id INT)")
        .await
        .unwrap();
    let result = active
        .execute_single("INSERT INTO mysql.yssv_dml_my VALUES (1)")
        .await
        .unwrap();
    assert_eq!(result.columns[0].name, "result");
    let msg = result.rows[0][0].as_deref().unwrap();
    assert!(msg.contains("rows affected"), "unexpected message: {msg}");
}

#[tokio::test]
async fn mysql_execute_batch_commits_and_returns_affected() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE mysql.yssv_batch_my (id INT)")
        .await
        .unwrap();

    let stmts = vec![
        "INSERT INTO mysql.yssv_batch_my VALUES (1)".into(),
        "INSERT INTO mysql.yssv_batch_my VALUES (2)".into(),
    ];
    let affected = active.execute_batch(&stmts).await.unwrap();
    assert_eq!(affected, 2);

    let count_result = active
        .execute_single("SELECT COUNT(*) FROM mysql.yssv_batch_my")
        .await
        .unwrap();
    assert_eq!(count_result.rows[0][0], Some("2".into()));
}

#[tokio::test]
async fn mysql_describe_table_returns_column_metadata() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();

    active
        .execute_single(
            "CREATE TABLE mysql.yssv_desc_my (id INT PRIMARY KEY, name VARCHAR(50) NOT NULL, bio TEXT)",
        )
        .await
        .unwrap();

    let cols = active
        .describe_table("mysql", "mysql", "yssv_desc_my")
        .await
        .unwrap();
    assert_eq!(cols.len(), 3);

    let id = cols.iter().find(|c| c.name == "id").unwrap();
    assert!(id.is_pk, "id should be PK");
    assert!(!id.nullable);

    let name = cols.iter().find(|c| c.name == "name").unwrap();
    assert!(!name.is_pk);
    assert!(!name.nullable);

    let bio = cols.iter().find(|c| c.name == "bio").unwrap();
    assert!(bio.nullable);
}

#[tokio::test]
async fn mysql_fetch_rows_returns_inserted_data() {
    let container = Mysql::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let active = drivers::connect(&mysql_conn_config(port)).await.unwrap();

    active
        .execute_single("CREATE TABLE mysql.yssv_fetch_my (id INT, val VARCHAR(50))")
        .await
        .unwrap();
    active
        .execute_single(
            "INSERT INTO mysql.yssv_fetch_my VALUES (1, 'hello'), (2, 'world')",
        )
        .await
        .unwrap();

    let result = active
        .fetch_rows("mysql", "mysql", "yssv_fetch_my", 10, 0)
        .await
        .unwrap();
    assert_eq!(result.columns.len(), 2);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.total_rows, Some(2));
}
