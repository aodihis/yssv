use crate::core::{
    results::model::{ColumnDef, QueryResult},
    schema::model::DbInfo,
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    pub selected_row: Option<usize>,
    pub structure: Option<Vec<ColumnDef>>,
    pub structure_loading: bool,
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
            selected_row: None,
            structure: None,
            structure_loading: false,
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
    pub fn new(
        conn_id: String,
        conn_name: String,
        default_db: &str,
        databases: Vec<DbInfo>,
    ) -> Self {
        // Pre-open the connected database (not just the first alphabetically).
        // The connection pool is scoped to default_db, so schema queries
        // always return data from that database.
        let active_db = if databases.iter().any(|d| d.name == default_db) {
            default_db.to_string()
        } else {
            databases
                .first()
                .map(|d| d.name.clone())
                .unwrap_or_default()
        };
        tracing::debug!(
            conn_id = %conn_id, default_db = %default_db,
            active_db = %active_db, db_count = databases.len(),
            "ExplorerState created"
        );
        let mut open_nodes = HashSet::new();
        open_nodes.insert(format!("db:{}", active_db));
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
                    TableInfo {
                        name: "users".into(),
                        kind: TableKind::Table,
                        row_count: Some(100),
                    },
                    TableInfo {
                        name: "orders".into(),
                        kind: TableKind::Table,
                        row_count: Some(500),
                    },
                ],
            }],
        }]
    }

    fn make_empty_db() -> Vec<DbInfo> {
        vec![DbInfo {
            name: "mydb".into(),
            schemas: vec![],
        }]
    }

    // --- TabState tests ---

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
        tab.result = Some(QueryResult {
            columns: vec![],
            rows: vec![],
            total_rows: Some(250),
        });
        assert_eq!(tab.total_pages(), 3);
        assert!(tab.can_go_next());
        tab.page = 2;
        assert!(!tab.can_go_next());
        assert!(tab.can_go_prev());
    }

    // --- ExplorerState: open_nodes ---

    #[test]
    fn default_db_is_pre_opened() {
        let state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        assert!(state.is_open("db:mydb"), "connected db must be pre-opened");
    }

    #[test]
    fn non_default_db_not_pre_opened() {
        let dbs = vec![
            DbInfo {
                name: "alpha".into(),
                schemas: vec![],
            },
            DbInfo {
                name: "mydb".into(),
                schemas: vec![],
            },
        ];
        let state = ExplorerState::new("id".into(), "name".into(), "mydb", dbs);
        assert!(!state.is_open("db:alpha"), "other dbs must start closed");
        assert!(state.is_open("db:mydb"));
    }

    #[test]
    fn active_db_falls_back_to_first_when_default_missing() {
        let dbs = vec![
            DbInfo {
                name: "alpha".into(),
                schemas: vec![],
            },
            DbInfo {
                name: "beta".into(),
                schemas: vec![],
            },
        ];
        let state = ExplorerState::new("id".into(), "name".into(), "missing", dbs);
        assert_eq!(state.active_db, "alpha");
        assert!(state.is_open("db:alpha"));
    }

    #[test]
    fn toggle_node_opens_and_closes() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        let key = "sc:mydb:public";
        assert!(!state.is_open(key));
        state.toggle_node(key);
        assert!(state.is_open(key));
        state.toggle_node(key);
        assert!(!state.is_open(key));
    }

    // --- ExplorerState: schema-loaded simulation (mirrors apply_event SchemasLoaded) ---

    #[test]
    fn schemas_loaded_event_populates_tables() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_empty_db());
        assert_eq!(
            state.filtered_tables("public").len(),
            0,
            "empty before load"
        );

        // Simulate what apply_event SchemasLoaded does
        if let Some(db) = state.databases.iter_mut().find(|d| d.name == "mydb") {
            db.schemas = vec![SchemaInfo {
                name: "public".into(),
                tables: vec![
                    TableInfo {
                        name: "users".into(),
                        kind: TableKind::Table,
                        row_count: Some(42),
                    },
                    TableInfo {
                        name: "posts".into(),
                        kind: TableKind::Table,
                        row_count: None,
                    },
                ],
            }];
        }

        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 2, "both tables visible after load");
        assert_eq!(tables[0].name, "users");
        assert_eq!(tables[1].name, "posts");
    }

    #[test]
    fn schemas_loaded_wrong_db_name_is_noop() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_empty_db());

        // Simulate SchemasLoaded arriving with a db name that doesn't match
        let result = state.databases.iter_mut().find(|d| d.name == "wrongdb");
        assert!(result.is_none(), "mismatch must not update any db");

        assert_eq!(state.filtered_tables("public").len(), 0);
    }

    // --- ExplorerState: filter ---

    #[test]
    fn filter_reduces_visible_tables() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        state.filter = "user".into();
        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "users");
    }

    #[test]
    fn filter_is_case_insensitive() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        state.filter = "USER".into();
        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "users");
    }

    #[test]
    fn empty_filter_shows_all_tables() {
        let state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let mut state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        state.filter = "zzz_no_match".into();
        let tables = state.filtered_tables("public");
        assert_eq!(tables.len(), 0);
    }

    // --- schema count logic (mirrors what sidebar computes for the count badge) ---

    #[test]
    fn schema_table_count_correct_before_schema_open() {
        let state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        let db = state.databases.iter().find(|d| d.name == "mydb").unwrap();
        let sc = db.schemas.iter().find(|s| s.name == "public").unwrap();
        // This mirrors the sidebar's filtered_count calculation for an empty filter
        let count = sc.tables.len();
        assert_eq!(count, 2, "count must reflect actual tables, not open-state");
    }

    #[test]
    fn schema_table_count_with_filter() {
        let state = ExplorerState::new("id".into(), "name".into(), "mydb", make_db());
        let db = state.databases.iter().find(|d| d.name == "mydb").unwrap();
        let sc = db.schemas.iter().find(|s| s.name == "public").unwrap();
        let q = "user";
        let count = sc
            .tables
            .iter()
            .filter(|t| t.name.to_lowercase().contains(q))
            .count();
        assert_eq!(count, 1);
    }
}
