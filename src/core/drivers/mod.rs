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

    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DbError>;
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
}
