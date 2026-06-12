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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_kind_labels() {
        assert_eq!(TableKind::Table.label(), "TABLE");
        assert_eq!(TableKind::View.label(), "VIEW");
    }

    #[test]
    fn db_info_serialization_roundtrip() {
        let info = DbInfo {
            name: "mydb".into(),
            schemas: vec![SchemaInfo {
                name: "public".into(),
                tables: vec![TableInfo {
                    name: "users".into(),
                    kind: TableKind::Table,
                    row_count: Some(42),
                }],
            }],
        };
        let json = serde_json::to_string(&info).unwrap();
        let decoded: DbInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "mydb");
        assert_eq!(decoded.schemas[0].tables[0].row_count, Some(42));
        assert_eq!(decoded.schemas[0].tables[0].kind, TableKind::Table);
    }

    #[test]
    fn table_info_view_kind() {
        let t = TableInfo {
            name: "v_orders".into(),
            kind: TableKind::View,
            row_count: None,
        };
        assert_eq!(t.kind.label(), "VIEW");
        assert!(t.row_count.is_none());
    }
}
