use std::collections::HashSet;
use crate::core::{results::model::QueryResult, schema::model::DbInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum TabView {
    #[default]
    Data,
    Structure,
}


#[derive(Debug, Clone)]
pub struct TableTab {
    pub id: String,
    pub table: String,
    pub schema: String,
    pub database: String,
    pub view: TabView,
    pub result: Option<QueryResult>,
    pub loading: bool,
    pub page: u32,
    pub page_size: u32,
}

impl TableTab {
    pub fn new(table: &str, schema: &str, database: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            table: table.to_string(),
            schema: schema.to_string(),
            database: database.to_string(),
            view: TabView::Data,
            result: None,
            loading: false,
            page: 0,
            page_size: 100,
        }
    }

    pub fn total_pages(&self) -> u64 {
        self.result
            .as_ref()
            .map(|r| r.page_count(self.page_size))
            .unwrap_or(1)
    }

    pub fn offset(&self) -> u32 {
        self.page * self.page_size
    }

    pub fn can_go_prev(&self) -> bool {
        self.page > 0
    }

    pub fn can_go_next(&self) -> bool {
        (self.page as u64 + 1) < self.total_pages()
    }
}

#[derive(Debug, Default)]
pub struct TabState {
    pub tabs: Vec<TableTab>,
    pub active: usize,
}

impl TabState {
    /// Open a tab for the given table, or activate if already open.
    pub fn open(&mut self, table: &str, schema: &str, database: &str) -> &mut TableTab {
        if let Some(pos) = self
            .tabs
            .iter()
            .position(|t| t.table == table && t.schema == schema && t.database == database)
        {
            self.active = pos;
        } else {
            self.tabs.push(TableTab::new(table, schema, database));
            self.active = self.tabs.len() - 1;
        }
        &mut self.tabs[self.active]
    }

    pub fn close(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active >= self.tabs.len() && !self.tabs.is_empty() {
                self.active = self.tabs.len() - 1;
            } else if self.tabs.is_empty() {
                self.active = 0;
            }
        }
    }

    pub fn active_tab(&self) -> Option<&TableTab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TableTab> {
        self.tabs.get_mut(self.active)
    }
}

#[derive(Debug)]
pub struct ExplorerState {
    pub conn_id: String,
    pub conn_name: String,
    pub databases: Vec<DbInfo>,
    pub open_nodes: HashSet<String>,
    pub active_db: String,
    pub filter: String,
    pub tabs: TabState,
}

impl ExplorerState {
    pub fn new(conn_id: String, conn_name: String, databases: Vec<DbInfo>) -> Self {
        let active_db = databases.first().map(|d| d.name.clone()).unwrap_or_default();
        let mut open_nodes = HashSet::new();
        if let Some(db) = databases.first() {
            open_nodes.insert(format!("db:{}", db.name));
        }
        Self {
            conn_id,
            conn_name,
            databases,
            open_nodes,
            active_db,
            filter: String::new(),
            tabs: TabState::default(),
        }
    }

    pub fn toggle_node(&mut self, key: &str) {
        if self.open_nodes.contains(key) {
            self.open_nodes.remove(key);
        } else {
            self.open_nodes.insert(key.to_string());
        }
    }

    pub fn is_open(&self, key: &str) -> bool {
        self.open_nodes.contains(key)
    }

    /// Filtered tables for the active database + schema, respecting search filter.
    pub fn filtered_tables<'a>(
        &'a self,
        schema_name: &str,
    ) -> Vec<&'a crate::core::schema::model::TableInfo> {
        let q = self.filter.to_lowercase();
        self.databases
            .iter()
            .find(|db| db.name == self.active_db)
            .and_then(|db| db.schemas.iter().find(|s| s.name == schema_name))
            .map(|sc| {
                sc.tables
                    .iter()
                    .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema::model::{DbInfo, SchemaInfo, TableInfo, TableKind};

    fn make_db() -> Vec<DbInfo> {
        vec![DbInfo {
            name: "mydb".into(),
            schemas: vec![SchemaInfo {
                name: "public".into(),
                tables: vec![
                    TableInfo { name: "users".into(), kind: TableKind::Table, row_count: Some(100) },
                    TableInfo { name: "orders".into(), kind: TableKind::Table, row_count: Some(500) },
                ],
            }],
        }]
    }

    #[test]
    fn open_tab_activates_existing() {
        let mut ts = TabState::default();
        ts.open("users", "public", "mydb");
        ts.open("orders", "public", "mydb");
        assert_eq!(ts.tabs.len(), 2);
        ts.open("users", "public", "mydb");
        assert_eq!(ts.tabs.len(), 2);
        assert_eq!(ts.active, 0);
    }

    #[test]
    fn close_tab_adjusts_active() {
        let mut ts = TabState::default();
        ts.open("a", "s", "db");
        ts.open("b", "s", "db");
        ts.active = 1;
        ts.close(1);
        assert_eq!(ts.tabs.len(), 1);
        assert_eq!(ts.active, 0);
    }

    #[test]
    fn tab_pagination_math() {
        let mut tab = TableTab::new("t", "s", "db");
        tab.page_size = 100;
        tab.result = Some(QueryResult { columns: vec![], rows: vec![], total_rows: Some(250) });
        assert_eq!(tab.total_pages(), 3);
        assert!(tab.can_go_next());
        tab.page = 2;
        assert!(!tab.can_go_next());
        assert!(tab.can_go_prev());
    }

    #[test]
    fn toggle_node_opens_and_closes() {
        let mut state = ExplorerState::new("id".into(), "name".into(), make_db());
        let key = "sc:public";
        assert!(!state.is_open(key));
        state.toggle_node(key);
        assert!(state.is_open(key));
        state.toggle_node(key);
        assert!(!state.is_open(key));
    }

    #[test]
    fn filter_reduces_visible_tables() {
        let mut state = ExplorerState::new("id".into(), "name".into(), make_db());
        state.filter = "user".into();
        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "users");
    }
}
