pub mod error;
pub mod mysql;
pub mod postgres;

use crate::core::{
    connections::model::Connection,
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo, TableKind},
    ssh::{SshTunnel, TunnelStatusHandle},
};
use async_trait::async_trait;
pub use error::DbError;

#[async_trait]
pub trait ActiveConnection: Send + Sync {
    async fn current_database(&self) -> Result<String, DbError>;
    async fn list_databases(&self) -> Result<Vec<String>, DbError>;
    async fn list_schemas(&self, db: &str) -> Result<Vec<SchemaInfo>, DbError>;
    async fn list_tables(&self, db: &str, schema: &str) -> Result<Vec<TableInfo>, DbError>;
    async fn fetch_rows(
        &self,
        db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError>;
    async fn describe_table(
        &self,
        db: &str,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnDef>, DbError>;

    async fn execute_single(&self, sql: &str) -> Result<QueryResult, DbError>;

    /// Run every statement inside a single transaction, returning the total
    /// number of affected rows. Any failure rolls the whole batch back. This
    /// backs the DataGrip-style "commit pending changes" action.
    async fn execute_batch(&self, statements: &[String]) -> Result<u64, DbError>;

    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DbError> {
        let stmts = split_statements(sql);
        tracing::debug!(
            stmt_count = stmts.len(),
            sql_len = sql.len(),
            "execute_query"
        );
        if stmts.is_empty() {
            return Ok(QueryResult::empty());
        }
        let mut last = QueryResult::empty();
        for stmt in &stmts {
            last = self.execute_single(stmt).await?;
        }
        Ok(last)
    }

    /// Live SSH tunnel status, when this connection is routed through one.
    fn tunnel_status(&self) -> Option<TunnelStatusHandle> {
        None
    }
}

/// Wraps a driver connection that is routed through an SSH tunnel, keeping the
/// tunnel alive for the connection's lifetime and exposing its status.
pub struct TunneledConnection {
    inner: Box<dyn ActiveConnection>,
    tunnel: SshTunnel,
}

impl TunneledConnection {
    pub fn new(inner: Box<dyn ActiveConnection>, tunnel: SshTunnel) -> Self {
        Self { inner, tunnel }
    }
}

#[async_trait]
impl ActiveConnection for TunneledConnection {
    async fn current_database(&self) -> Result<String, DbError> {
        self.inner.current_database().await
    }

    async fn describe_table(
        &self,
        db: &str,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnDef>, DbError> {
        self.inner.describe_table(db, schema, table).await
    }

    async fn execute_batch(&self, statements: &[String]) -> Result<u64, DbError> {
        self.inner.execute_batch(statements).await
    }

    async fn execute_single(&self, sql: &str) -> Result<QueryResult, DbError> {
        self.inner.execute_single(sql).await
    }

    async fn fetch_rows(
        &self,
        db: &str,
        schema: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DbError> {
        self.inner
            .fetch_rows(db, schema, table, limit, offset)
            .await
    }

    async fn list_databases(&self) -> Result<Vec<String>, DbError> {
        self.inner.list_databases().await
    }

    async fn list_schemas(&self, db: &str) -> Result<Vec<SchemaInfo>, DbError> {
        self.inner.list_schemas(db).await
    }

    async fn list_tables(&self, db: &str, schema: &str) -> Result<Vec<TableInfo>, DbError> {
        self.inner.list_tables(db, schema).await
    }

    fn tunnel_status(&self) -> Option<TunnelStatusHandle> {
        Some(self.tunnel.status_handle())
    }
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    match &conn.ssh {
        Some(ssh) => {
            let tunnel = SshTunnel::open(ssh, &conn.host, conn.port).await?;
            let mut effective = conn.clone();
            effective.host = "127.0.0.1".into();
            effective.port = tunnel.local_port();
            effective.ssh = None;
            tracing::debug!(
                local_port = tunnel.local_port(),
                "driver connect: routing through SSH tunnel"
            );
            let inner = connect_direct(&effective).await?;
            Ok(Box::new(TunneledConnection::new(inner, tunnel)))
        }
        None => connect_direct(conn).await,
    }
}

async fn connect_direct(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    use crate::core::connections::model::DbEngine;
    match conn.engine {
        DbEngine::Postgres => postgres::driver::connect(conn).await,
        DbEngine::MySQL => mysql::driver::connect(conn).await,
    }
}

/// Shared helper: map a SQL table_type string to `TableKind`.
pub(super) fn table_type_to_kind(ttype: &str) -> TableKind {
    if ttype == "VIEW" {
        TableKind::View
    } else {
        TableKind::Table
    }
}

/// Split a SQL string into individual statements at `;` boundaries, respecting
/// quoted strings, identifiers, and comments so embedded semicolons are ignored.
pub(crate) fn split_statements(sql: &str) -> Vec<String> {
    let bytes = sql.as_bytes();
    let n = bytes.len();
    let mut stmts = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < n {
        // Line comment: -- ...
        if i + 1 < n && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
        // Block comment: /* ... */
        } else if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < n {
                i += 2;
            }
        // Single-quoted string literal
        } else if bytes[i] == b'\'' {
            i += 1;
            while i < n {
                if bytes[i] == b'\\' {
                    i += (n - i).min(2);
                } else if bytes[i] == b'\'' {
                    i += 1;
                    if i < n && bytes[i] == b'\'' {
                        i += 1; // '' escape
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        // Double-quoted identifier
        } else if bytes[i] == b'"' {
            i += 1;
            while i < n && bytes[i] != b'"' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
        // Backtick identifier (MySQL)
        } else if bytes[i] == b'`' {
            i += 1;
            while i < n && bytes[i] != b'`' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
        // Statement separator
        } else if bytes[i] == b';' {
            let stmt = sql[start..i].trim();
            if !stmt.is_empty() {
                stmts.push(stmt.to_string());
            }
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    let stmt = sql[start..].trim();
    if !stmt.is_empty() {
        stmts.push(stmt.to_string());
    }

    stmts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_type_view_maps_to_view() {
        assert_eq!(table_type_to_kind("VIEW"), TableKind::View);
    }

    #[test]
    fn table_type_base_table_maps_to_table() {
        assert_eq!(table_type_to_kind("BASE TABLE"), TableKind::Table);
    }

    #[test]
    fn table_type_unknown_maps_to_table() {
        assert_eq!(table_type_to_kind("MATERIALIZED VIEW"), TableKind::Table);
    }

    #[test]
    fn split_single_statement_no_semicolon() {
        assert_eq!(split_statements("SELECT 1"), vec!["SELECT 1"]);
    }

    #[test]
    fn split_single_statement_trailing_semicolon() {
        assert_eq!(split_statements("SELECT 1;"), vec!["SELECT 1"]);
    }

    #[test]
    fn split_two_statements() {
        assert_eq!(
            split_statements("SELECT 1; SELECT 2"),
            vec!["SELECT 1", "SELECT 2"]
        );
    }

    #[test]
    fn split_ignores_semicolon_in_string() {
        assert_eq!(split_statements("SELECT 'a;b'"), vec!["SELECT 'a;b'"]);
    }

    #[test]
    fn split_ignores_semicolon_in_line_comment() {
        assert_eq!(
            split_statements("SELECT 1 -- ; this is a comment\n, 2"),
            vec!["SELECT 1 -- ; this is a comment\n, 2"]
        );
    }

    #[test]
    fn split_ignores_semicolon_in_block_comment() {
        assert_eq!(
            split_statements("SELECT /* ; */ 1"),
            vec!["SELECT /* ; */ 1"]
        );
    }

    #[test]
    fn split_empty_string_returns_empty() {
        assert!(split_statements("").is_empty());
    }

    #[test]
    fn split_whitespace_only_returns_empty() {
        assert!(split_statements("   \n  ").is_empty());
    }

    #[test]
    fn split_trims_whitespace_from_statements() {
        assert_eq!(
            split_statements("  SELECT 1  ;  SELECT 2  "),
            vec!["SELECT 1", "SELECT 2"]
        );
    }

    #[test]
    fn split_escaped_quote_in_string() {
        assert_eq!(
            split_statements("SELECT 'it''s';SELECT 2"),
            vec!["SELECT 'it''s'", "SELECT 2"]
        );
    }

    #[test]
    fn split_ignores_semicolon_in_backtick_ident() {
        assert_eq!(
            split_statements("SELECT `col;name` FROM t"),
            vec!["SELECT `col;name` FROM t"]
        );
    }

    #[test]
    fn split_ignores_semicolon_in_double_quoted_ident() {
        assert_eq!(
            split_statements(r#"SELECT "col;name" FROM t"#),
            vec![r#"SELECT "col;name" FROM t"#]
        );
    }

    #[test]
    fn split_backslash_escape_in_string() {
        let result = split_statements(r"SELECT 'it\'s ok';SELECT 2");
        assert_eq!(result, vec![r"SELECT 'it\'s ok'", "SELECT 2"]);
    }

    #[test]
    fn split_only_semicolons_returns_empty() {
        assert!(split_statements(";;;").is_empty());
    }

    #[test]
    fn split_consecutive_empty_stmts_are_skipped() {
        assert_eq!(
            split_statements("SELECT 1;; SELECT 2"),
            vec!["SELECT 1", "SELECT 2"]
        );
    }

    // --- execute_query (default trait method) ---

    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct MockConn {
        call_count: Arc<AtomicUsize>,
        fixed_result: Result<QueryResult, String>,
    }

    impl MockConn {
        fn new(result: QueryResult) -> (Self, Arc<AtomicUsize>) {
            let counter = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    call_count: counter.clone(),
                    fixed_result: Ok(result),
                },
                counter,
            )
        }

        fn failing(msg: &str) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                fixed_result: Err(msg.into()),
            }
        }
    }

    #[async_trait::async_trait]
    impl ActiveConnection for MockConn {
        async fn current_database(&self) -> Result<String, DbError> {
            unimplemented!()
        }
        async fn list_databases(&self) -> Result<Vec<String>, DbError> {
            unimplemented!()
        }
        async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
            unimplemented!()
        }
        async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
            unimplemented!()
        }
        async fn fetch_rows(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: u32,
            _: u32,
        ) -> Result<QueryResult, DbError> {
            unimplemented!()
        }
        async fn describe_table(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<Vec<ColumnDef>, DbError> {
            unimplemented!()
        }
        async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            self.fixed_result.clone().map_err(DbError::new)
        }
        async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn execute_query_empty_sql_returns_empty() {
        let (conn, count) = MockConn::new(QueryResult::empty());
        let r = conn.execute_query("").await.unwrap();
        assert!(r.columns.is_empty() && r.rows.is_empty());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn execute_query_whitespace_only_returns_empty() {
        let (conn, count) = MockConn::new(QueryResult::empty());
        let r = conn.execute_query("   \n  ").await.unwrap();
        assert!(r.rows.is_empty());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn execute_query_single_statement_calls_execute_single_once() {
        let fixed = QueryResult {
            columns: vec![ColumnDef {
                name: "n".into(),
                data_type: "int".into(),
                is_pk: false,
                is_fk: false,
                nullable: false,
            }],
            rows: vec![vec![Some("42".into())]],
            total_rows: Some(1),
        };
        let (conn, count) = MockConn::new(fixed);
        let r = conn.execute_query("SELECT 42").await.unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!(r.rows[0][0], Some("42".into()));
    }

    #[tokio::test]
    async fn execute_query_multi_statement_calls_execute_single_per_stmt() {
        let (conn, count) = MockConn::new(QueryResult::empty());
        let _ = conn
            .execute_query("SELECT 1; SELECT 2; SELECT 3")
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn execute_query_propagates_execute_single_error() {
        let conn = MockConn::failing("forced error");
        let err = conn.execute_query("SELECT 1").await.unwrap_err();
        assert!(err.message.contains("forced error"));
    }

    // --- tunnel_status default impl ---

    #[tokio::test]
    async fn tunnel_status_default_returns_none() {
        let (conn, _) = MockConn::new(QueryResult::empty());
        assert!(conn.tunnel_status().is_none());
    }

    // --- TunneledConnection ---
    // TunneledConnection is constructed with a fake SshTunnel (no real SSH needed).

    fn simple_result() -> QueryResult {
        QueryResult {
            columns: vec![ColumnDef {
                name: "x".into(),
                data_type: "int".into(),
                is_pk: false,
                is_fk: false,
                nullable: false,
            }],
            rows: vec![vec![Some("1".into())]],
            total_rows: Some(1),
        }
    }

    #[tokio::test]
    async fn tunneled_connection_current_database_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                Ok("mydb".into())
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9001).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        assert_eq!(tc.current_database().await.unwrap(), "mydb");
    }

    #[tokio::test]
    async fn tunneled_connection_tunnel_status_returns_some() {
        let (inner, _) = MockConn::new(QueryResult::empty());
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9002).await;
        let tc = TunneledConnection::new(Box::new(inner), tunnel);
        assert!(tc.tunnel_status().is_some());
    }

    #[tokio::test]
    async fn tunneled_connection_list_databases_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                Ok(vec!["a".into(), "b".into()])
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9003).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        assert_eq!(tc.list_databases().await.unwrap(), vec!["a", "b"]);
    }

    #[tokio::test]
    async fn tunneled_connection_list_schemas_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                Ok(vec![SchemaInfo {
                    name: "public".into(),
                    tables: vec![],
                }])
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9004).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        let schemas = tc.list_schemas("mydb").await.unwrap();
        assert_eq!(schemas[0].name, "public");
    }

    #[tokio::test]
    async fn tunneled_connection_list_tables_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                Ok(vec![TableInfo {
                    name: "users".into(),
                    kind: TableKind::Table,
                    row_count: Some(10),
                }])
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9005).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        let tables = tc.list_tables("mydb", "public").await.unwrap();
        assert_eq!(tables[0].name, "users");
    }

    #[tokio::test]
    async fn tunneled_connection_describe_table_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                Ok(vec![ColumnDef {
                    name: "id".into(),
                    data_type: "int".into(),
                    is_pk: true,
                    is_fk: false,
                    nullable: false,
                }])
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9006).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        let cols = tc.describe_table("mydb", "public", "users").await.unwrap();
        assert!(cols[0].is_pk);
    }

    #[tokio::test]
    async fn tunneled_connection_fetch_rows_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                Ok(simple_result())
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9007).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        let result = tc
            .fetch_rows("mydb", "public", "users", 10, 0)
            .await
            .unwrap();
        assert_eq!(result.total_rows, Some(1));
    }

    #[tokio::test]
    async fn tunneled_connection_execute_single_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                Ok(simple_result())
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                unimplemented!()
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9008).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        let result = tc.execute_single("SELECT 1").await.unwrap();
        assert_eq!(result.total_rows, Some(1));
    }

    #[tokio::test]
    async fn tunneled_connection_execute_batch_delegates() {
        struct DbMock;
        #[async_trait::async_trait]
        impl ActiveConnection for DbMock {
            async fn current_database(&self) -> Result<String, DbError> {
                unimplemented!()
            }
            async fn list_databases(&self) -> Result<Vec<String>, DbError> {
                unimplemented!()
            }
            async fn list_schemas(&self, _: &str) -> Result<Vec<SchemaInfo>, DbError> {
                unimplemented!()
            }
            async fn list_tables(&self, _: &str, _: &str) -> Result<Vec<TableInfo>, DbError> {
                unimplemented!()
            }
            async fn fetch_rows(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: u32,
                _: u32,
            ) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn describe_table(
                &self,
                _: &str,
                _: &str,
                _: &str,
            ) -> Result<Vec<ColumnDef>, DbError> {
                unimplemented!()
            }
            async fn execute_single(&self, _: &str) -> Result<QueryResult, DbError> {
                unimplemented!()
            }
            async fn execute_batch(&self, _: &[String]) -> Result<u64, DbError> {
                Ok(3)
            }
        }
        let tunnel = crate::core::ssh::SshTunnel::new_for_test(9009).await;
        let tc = TunneledConnection::new(Box::new(DbMock), tunnel);
        assert_eq!(
            tc.execute_batch(&["UPDATE t SET x=1".into()])
                .await
                .unwrap(),
            3
        );
    }

    // --- split_statements edge cases: uncovered branches ---

    #[test]
    fn split_unterminated_block_comment_does_not_panic() {
        // `if i + 1 < n { i += 2; }` false branch: no closing */
        let result = split_statements("SELECT /* unterminated");
        assert_eq!(result, vec!["SELECT /* unterminated"]);
    }

    #[test]
    fn split_unterminated_double_quoted_ident_does_not_panic() {
        // `if i < n { i += 1; }` false branch for double-quoted identifier
        let result = split_statements("SELECT \"col");
        assert_eq!(result, vec!["SELECT \"col"]);
    }

    #[test]
    fn split_unterminated_backtick_ident_does_not_panic() {
        // `if i < n { i += 1; }` false branch for backtick identifier
        let result = split_statements("SELECT `col");
        assert_eq!(result, vec!["SELECT `col"]);
    }

    #[test]
    fn split_backslash_at_end_of_string_advances_by_one() {
        // `(n - i).min(2)` returns 1 when backslash is the very last byte
        let result = split_statements("'hello\\");
        assert_eq!(result, vec!["'hello\\"]);
    }

    #[test]
    fn split_line_comment_at_eof_without_trailing_newline() {
        // Inner `while i < n && bytes[i] != b'\n'` exits via `i >= n` (not via newline)
        let result = split_statements("SELECT 1 -- no newline");
        assert_eq!(result, vec!["SELECT 1 -- no newline"]);
    }
}
