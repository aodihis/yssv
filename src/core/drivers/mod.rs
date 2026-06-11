pub mod error;
pub mod mysql;
pub mod postgres;

use crate::core::{
    connections::model::Connection,
    results::model::{ColumnDef, QueryResult},
    schema::model::{SchemaInfo, TableInfo},
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
}

pub async fn connect(conn: &Connection) -> Result<Box<dyn ActiveConnection>, DbError> {
    use crate::core::connections::model::DbEngine;
    match conn.engine {
        DbEngine::Postgres => postgres::driver::connect(conn).await,
        DbEngine::MySQL => mysql::driver::connect(conn).await,
    }
}
