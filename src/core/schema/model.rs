use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbInfo {
    pub name: String,
    pub schemas: Vec<SchemaInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub kind: TableKind,
    pub row_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableKind {
    Table,
    View,
}

impl TableKind {
    pub fn label(&self) -> &'static str {
        match self {
            TableKind::Table => "TABLE",
            TableKind::View => "VIEW",
        }
    }
}
