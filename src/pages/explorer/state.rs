use crate::core::{
    edit::TableEdits,
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

/// Which logical row an edit targets: an existing row in the loaded page, or a
/// not-yet-committed insert row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowRef {
    Original(usize),
    Insert(usize),
}

/// Transient state for the cell currently being edited inline in the grid.
#[derive(Debug, Clone)]
pub struct EditingCell {
    pub row: RowRef,
    pub col: usize,
    pub buffer: String,
    pub request_focus: bool,
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
    // Boxed: pending edits are empty for the common read-only tab, so keeping
    // them off the inline struct keeps `Tab::Table` from dwarfing `Tab::Query`.
    pub edits: Box<TableEdits>,
    pub editing: Option<EditingCell>,
    pub committing: bool,
    pub show_commit_dialog: bool,
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
            edits: Box::new(TableEdits::default()),
            editing: None,
            committing: false,
            show_commit_dialog: false,
        }
    }

    /// Number of columns in the currently loaded result, or 0 if none.
    pub fn column_count(&self) -> usize {
        self.result.as_ref().map(|r| r.columns.len()).unwrap_or(0)
    }

    /// Append a blank insert row, select it, and immediately begin editing its
    /// first cell so the new row is obviously editable. Returns the insert index.
    pub fn add_insert_row(&mut self) -> Option<usize> {
        let cols = self.column_count();
        if cols == 0 {
            return None;
        }
        self.edits.inserts.push(vec![None; cols]);
        let idx = self.edits.inserts.len() - 1;
        let orig = self.result.as_ref().map(|r| r.rows.len()).unwrap_or(0);
        self.selected_row = Some(orig + idx);
        self.editing = Some(EditingCell {
            row: RowRef::Insert(idx),
            col: 0,
            buffer: String::new(),
            request_focus: true,
        });
        Some(idx)
    }

    /// Toggle deletion of an existing row, or drop an uncommitted insert row.
    /// `display` is the index as shown in the grid (originals first, then inserts).
    pub fn toggle_delete_display_row(&mut self, display: usize) {
        let orig = self.result.as_ref().map(|r| r.rows.len()).unwrap_or(0);
        if display < orig {
            if !self.edits.deletes.insert(display) {
                self.edits.deletes.remove(&display);
            }
        } else {
            let i = display - orig;
            if i < self.edits.inserts.len() {
                self.edits.inserts.remove(i);
            }
        }
        self.editing = None;
        self.selected_row = None;
    }

    /// Discard all pending edits and any in-progress cell edit.
    pub fn revert_edits(&mut self) {
        self.edits.clear();
        self.editing = None;
        self.show_commit_dialog = false;
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

#[derive(Debug, Clone)]
pub struct QueryTab {
    pub id: String,
    pub counter: u32,
    pub sql: String,
    pub database: String,
    pub result: Option<QueryResult>,
    pub error: Option<String>,
    pub loading: bool,
    pub selected_row: Option<usize>,
}

impl QueryTab {
    pub fn new(database: &str, counter: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            counter,
            sql: String::new(),
            database: database.to_string(),
            result: None,
            error: None,
            loading: false,
            selected_row: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tab {
    Table(TableTab),
    Query(QueryTab),
}

impl Tab {
    pub fn id(&self) -> &str {
        match self {
            Tab::Table(t) => &t.id,
            Tab::Query(q) => &q.id,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Tab::Table(t) => format!("{}.{}", t.schema, t.table),
            Tab::Query(q) => format!("SQL {}", q.counter),
        }
    }

    pub fn as_table(&self) -> Option<&TableTab> {
        match self {
            Tab::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_table_mut(&mut self) -> Option<&mut TableTab> {
        match self {
            Tab::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_query(&self) -> Option<&QueryTab> {
        match self {
            Tab::Query(q) => Some(q),
            _ => None,
        }
    }

    pub fn as_query_mut(&mut self) -> Option<&mut QueryTab> {
        match self {
            Tab::Query(q) => Some(q),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct TabState {
    pub tabs: Vec<Tab>,
    pub active: usize,
    next_query_counter: u32,
}

impl TabState {
    /// Open a table tab, or activate if already open.
    pub fn open_table(&mut self, table: &str, schema: &str, database: &str) -> &mut TableTab {
        if let Some(pos) = self.tabs.iter().position(|t| {
            matches!(t, Tab::Table(tt) if tt.table == table && tt.schema == schema && tt.database == database)
        }) {
            self.active = pos;
        } else {
            self.tabs.push(Tab::Table(TableTab::new(table, schema, database)));
            self.active = self.tabs.len() - 1;
        }
        self.tabs[self.active].as_table_mut().unwrap()
    }

    /// Open a new query tab (always creates a fresh one).
    pub fn open_query(&mut self, database: &str) -> &mut QueryTab {
        self.next_query_counter += 1;
        self.tabs.push(Tab::Query(QueryTab::new(database, self.next_query_counter)));
        self.active = self.tabs.len() - 1;
        self.tabs[self.active].as_query_mut().unwrap()
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

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    pub fn active_table_tab(&self) -> Option<&TableTab> {
        self.tabs.get(self.active)?.as_table()
    }

    pub fn active_table_tab_mut(&mut self) -> Option<&mut TableTab> {
        self.tabs.get_mut(self.active)?.as_table_mut()
    }

    pub fn active_query_tab(&self) -> Option<&QueryTab> {
        self.tabs.get(self.active)?.as_query()
    }

    pub fn active_query_tab_mut(&mut self) -> Option<&mut QueryTab> {
        self.tabs.get_mut(self.active)?.as_query_mut()
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
    pub tunnel_status: Option<crate::core::ssh::TunnelStatusHandle>,
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
            tunnel_status: None,
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
        ts.open_table("users", "public", "mydb");
        ts.open_table("orders", "public", "mydb");
        assert_eq!(ts.tabs.len(), 2);
        ts.open_table("users", "public", "mydb");
        assert_eq!(ts.tabs.len(), 2);
        assert_eq!(ts.active, 0);
    }

    #[test]
    fn close_tab_adjusts_active() {
        let mut ts = TabState::default();
        ts.open_table("a", "s", "db");
        ts.open_table("b", "s", "db");
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

    // --- TableTab editing helpers ---

    fn tab_with_two_rows() -> TableTab {
        let mut tab = TableTab::new("users", "public", "mydb");
        tab.result = Some(QueryResult {
            columns: vec![
                ColumnDef { name: "id".into(), data_type: "int4".into(), is_pk: true, is_fk: false, nullable: false },
                ColumnDef { name: "name".into(), data_type: "text".into(), is_pk: false, is_fk: false, nullable: true },
            ],
            rows: vec![
                vec![Some("1".into()), Some("a".into())],
                vec![Some("2".into()), Some("b".into())],
            ],
            total_rows: Some(2),
        });
        tab
    }

    #[test]
    fn add_insert_row_appends_blank_row_sized_to_columns() {
        let mut tab = tab_with_two_rows();
        let idx = tab.add_insert_row();
        assert_eq!(idx, Some(0));
        assert_eq!(tab.edits.inserts.len(), 1);
        assert_eq!(tab.edits.inserts[0], vec![None, None]);
    }

    #[test]
    fn add_insert_row_without_result_is_none() {
        let mut tab = TableTab::new("t", "s", "db");
        assert_eq!(tab.add_insert_row(), None);
    }

    #[test]
    fn toggle_delete_marks_and_unmarks_original_row() {
        let mut tab = tab_with_two_rows();
        tab.toggle_delete_display_row(1);
        assert!(tab.edits.deletes.contains(&1));
        tab.toggle_delete_display_row(1);
        assert!(!tab.edits.deletes.contains(&1));
    }

    #[test]
    fn toggle_delete_drops_insert_row() {
        let mut tab = tab_with_two_rows();
        tab.add_insert_row();
        // display index 2 = first insert (originals are 0,1)
        tab.toggle_delete_display_row(2);
        assert!(tab.edits.inserts.is_empty());
        assert!(tab.edits.deletes.is_empty());
    }

    #[test]
    fn revert_edits_clears_everything() {
        let mut tab = tab_with_two_rows();
        tab.edits.updates.insert((0, 1), Some("x".into()));
        tab.add_insert_row();
        tab.edits.deletes.insert(1);
        tab.revert_edits();
        assert!(tab.edits.is_empty());
        assert!(tab.editing.is_none());
    }

    #[test]
    fn open_query_tab_increments_counter() {
        let mut ts = TabState::default();
        ts.open_query("mydb");
        ts.open_query("mydb");
        assert_eq!(ts.tabs.len(), 2);
        assert_eq!(ts.tabs[0].as_query().unwrap().counter, 1);
        assert_eq!(ts.tabs[1].as_query().unwrap().counter, 2);
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
