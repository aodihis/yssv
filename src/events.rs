use std::sync::Arc;

use crate::core::{
    drivers::ActiveConnection,
    results::model::{ColumnDef, QueryResult},
    schema::model::DbInfo,
};

pub enum AppEvent {
    Connected {
        conn_id: String,
        conn_name: String,
        default_db: String,
        databases: Vec<DbInfo>,
        connection: Arc<dyn ActiveConnection>,
        conn_config: Box<crate::core::connections::model::Connection>,
    },
    ConnectError {
        conn_id: String,
        message: String,
    },
    TestOk {
        conn_id: String,
        latency_ms: u64,
    },
    TestError {
        conn_id: String,
        message: String,
    },
    SchemasLoaded {
        db: String,
        schemas: Vec<crate::core::schema::model::SchemaInfo>,
    },
    RowsLoaded {
        tab_id: String,
        result: QueryResult,
    },
    RowLoadError {
        tab_id: String,
        message: String,
    },
    StructureLoaded {
        tab_id: String,
        columns: Vec<ColumnDef>,
    },
    DbConnected {
        db: String,
        conn: Arc<dyn ActiveConnection>,
    },
}

pub enum Screen {
    Connections,
    Explorer,
}
