pub mod error;
pub mod mysql;
pub mod postgres;

use crate::core::{
    connections::model::Connection,
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo, TableKind},
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

    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DbError> {
        let stmts = split_statements(sql);
        tracing::debug!(stmt_count = stmts.len(), sql_len = sql.len(), "execute_query");
        if stmts.is_empty() {
            return Ok(QueryResult::empty());
        }
        let mut last = QueryResult::empty();
        for stmt in &stmts {
            last = self.execute_single(stmt).await?;
        }
        Ok(last)
    }
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    use crate::core::connections::model::DbEngine;
    match conn.engine {
        DbEngine::Postgres => postgres::driver::connect(conn).await,
        DbEngine::MySQL => mysql::driver::connect(conn).await,
    }
}

/// Shared helper: map a SQL table_type string to `TableKind`.
pub(super) fn table_type_to_kind(ttype: &str) -> TableKind {
    if ttype == "VIEW" { TableKind::View } else { TableKind::Table }
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
        assert_eq!(
            split_statements("SELECT 'a;b'"),
            vec!["SELECT 'a;b'"]
        );
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
}
