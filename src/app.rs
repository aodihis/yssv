use std::collections::HashMap;
use std::sync::{Arc, mpsc};

use crate::core::{connections::storage::Storage, drivers::ActiveConnection, schema::model::DbInfo};
use crate::events::{AppEvent, Screen};
use crate::pages::{
    connections::state::ConnectionsPageState, explorer::state::ExplorerState,
    settings::state::SettingsState,
};
use crate::theme;
use crate::utils::data_dir_path;

pub struct YssvApp {
    pub screen: Screen,
    pub settings: SettingsState,
    pub conn_page: ConnectionsPageState,
    pub explorer: Option<ExplorerState>,
    pub error_modal: Option<String>,

    // One cached connection per database name. Created lazily on first use
    // (table open or schema load) and reused for the rest of the session.
    // Keyed by database name so queries always hit the right database —
    // PostgreSQL information_schema and relation lookups are scoped to the
    // current connection's database, not the one named in the query.
    db_conns: HashMap<String, Arc<dyn ActiveConnection>>,
    conn_config: Option<crate::core::connections::model::Connection>,

    storage: Storage,
    rt: Arc<tokio::runtime::Runtime>,
    event_tx: mpsc::SyncSender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
}

impl YssvApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rt: Arc<tokio::runtime::Runtime>) -> Self {
        let settings = SettingsState::load();
        theme::setup_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx, settings.theme);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let storage_path = data_dir_path();
        tracing::info!(path = %storage_path, "opening connection storage");
        if let Some(parent) = std::path::Path::new(&storage_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let storage = Storage::open(&storage_path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "storage open failed, falling back to in-memory");
            Storage::open_in_memory().expect("fallback in-memory storage failed")
        });

        let connections = storage.load_all().unwrap_or_default();
        tracing::info!(count = connections.len(), "loaded saved connections");
        let conn_page = ConnectionsPageState::new(connections);

        let (tx, rx) = mpsc::sync_channel(32);

        Self {
            screen: Screen::Connections,
            settings,
            conn_page,
            explorer: None,
            error_modal: None,
            db_conns: HashMap::new(),
            conn_config: None,
            storage,
            rt,
            event_tx: tx,
            event_rx: rx,
        }
    }

    pub fn drain_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(event, ctx);
        }
    }

    fn apply_event(&mut self, event: AppEvent, ctx: &egui::Context) {
        match event {
            AppEvent::Connected {
                conn_id,
                conn_name,
                default_db,
                databases,
                connection,
                conn_config,
            } => {
                tracing::info!(
                    conn_id = %conn_id, conn = %conn_name,
                    default_db = %default_db, db_count = databases.len(), "connected"
                );
                self.db_conns.clear();
                self.db_conns.insert(default_db.clone(), connection);
                self.conn_config = Some(*conn_config);
                let explorer = ExplorerState::new(conn_id, conn_name, &default_db, databases);
                let load_db = explorer.active_db.clone();
                self.explorer = Some(explorer);
                self.screen = Screen::Explorer;
                tracing::debug!(db = %load_db, "auto-loading schemas for connected database");
                self.load_schemas(ctx.clone(), load_db);
            }
            AppEvent::ConnectError { conn_id, message } => {
                tracing::warn!(conn_id = %conn_id, error = %message, "connection failed");
                self.db_conns.clear();
                self.conn_config = None;
                self.error_modal = Some(message);
                self.conn_page.test_status = crate::pages::connections::state::TestStatus::Idle;
            }
            AppEvent::TestOk {
                conn_id,
                latency_ms,
            } => {
                tracing::debug!(conn_id = %conn_id, latency_ms, "test connection ok");
                self.conn_page.test_status =
                    crate::pages::connections::state::TestStatus::Ok(latency_ms);
            }
            AppEvent::TestError { conn_id, message } => {
                tracing::warn!(conn_id = %conn_id, error = %message, "test connection failed");
                self.conn_page.test_status =
                    crate::pages::connections::state::TestStatus::Failed(message);
            }
            AppEvent::RowsLoaded { tab_id, result } => {
                tracing::debug!(tab_id = %tab_id, rows = result.rows.len(), "rows loaded");
                if let Some(explorer) = &mut self.explorer
                    && let Some(tab) = explorer.tabs.tabs.iter_mut().find(|t| t.id == tab_id)
                {
                    tab.result = Some(result);
                    tab.loading = false;
                }
            }
            AppEvent::RowLoadError { tab_id, message } => {
                tracing::warn!(tab_id = %tab_id, error = %message, "row load failed");
                if let Some(explorer) = &mut self.explorer
                    && let Some(tab) = explorer.tabs.tabs.iter_mut().find(|t| t.id == tab_id)
                {
                    tab.loading = false;
                    tab.result = None;
                }
                self.error_modal = Some(message);
            }
            AppEvent::SchemasLoaded { db, schemas } => {
                tracing::info!(db = %db, schema_count = schemas.len(), "schemas loaded");
                for s in &schemas {
                    tracing::debug!(db = %db, schema = %s.name, table_count = s.tables.len(), "schema ready");
                }
                if let Some(explorer) = &mut self.explorer {
                    if let Some(db_info) = explorer.databases.iter_mut().find(|d| d.name == db) {
                        db_info.schemas = schemas;
                        tracing::debug!(db = %db, "explorer database updated with schemas");
                    } else {
                        tracing::warn!(db = %db, "SchemasLoaded: no matching database in explorer");
                    }
                } else {
                    tracing::warn!(db = %db, "SchemasLoaded: explorer is None");
                }
            }
            AppEvent::StructureLoaded { tab_id, columns } => {
                tracing::debug!(tab_id = %tab_id, columns = columns.len(), "structure loaded");
                if let Some(explorer) = &mut self.explorer
                    && let Some(tab) = explorer.tabs.tabs.iter_mut().find(|t| t.id == tab_id)
                {
                    tab.structure = Some(columns);
                    tab.structure_loading = false;
                }
            }
            AppEvent::DbConnected { db, conn } => {
                tracing::debug!(db = %db, "db connection cached");
                self.db_conns.insert(db, conn);
            }
        }
    }

    pub fn connect(&self, ctx: egui::Context) {
        let conn = self.conn_page.form.to_connection();
        tracing::debug!(
            conn_id = %conn.id, name = %conn.name,
            host = %conn.host, port = conn.port,
            engine = ?conn.engine, "initiating connection"
        );
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            let conn_id = conn.id.clone();
            let conn_name = conn.name.clone();
            let configured_db = conn.database.clone();
            let user_specified_db = !configured_db.is_empty();
            match crate::core::drivers::connect(&conn).await {
                Ok(active) => {
                    let default_db = if configured_db.is_empty() {
                        match active.current_database().await {
                            Ok(db) => {
                                tracing::debug!(db = %db, "resolved current_database for empty config");
                                db
                            }
                            Err(e) => {
                                tracing::warn!(error = %e.message, "current_database failed, falling back to empty");
                                configured_db
                            }
                        }
                    } else {
                        configured_db
                    };
                    match active.list_databases().await {
                        Ok(dbs) => {
                            tracing::info!(
                                conn_id = %conn_id, db_count = dbs.len(),
                                default_db = %default_db, "database list loaded"
                            );
                                // When the user specified a database, only expose that one.
                            // The connection pool is scoped to it, so other databases
                            // can't be queried for schemas anyway.
                            // Use configured_db (not the resolved default_db) so that
                            // an empty form field still shows all databases.
                            let databases: Vec<DbInfo> =
                                if user_specified_db {
                                    vec![DbInfo {
                                        name: default_db.clone(),
                                        schemas: vec![],
                                    }]
                                } else {
                                    dbs.into_iter()
                                        .map(|name| DbInfo {
                                            name,
                                            schemas: vec![],
                                        })
                                        .collect()
                                };
                            let connection: Arc<dyn ActiveConnection> = Arc::from(active);
                            let _ = tx.send(AppEvent::Connected {
                                conn_id,
                                conn_name,
                                default_db,
                                databases,
                                connection,
                                conn_config: Box::new(conn.clone()),
                            });
                        }
                        Err(e) => {
                            let hint = e.install_hint().unwrap_or("").to_string();
                            let msg = if hint.is_empty() {
                                e.message.clone()
                            } else {
                                format!("{}\n\n{}", e.message, hint)
                            };
                            let _ = tx.send(AppEvent::ConnectError {
                                conn_id,
                                message: msg,
                            });
                        }
                    }
                }
                Err(e) => {
                    let hint = e.install_hint().unwrap_or("").to_string();
                    let msg = if hint.is_empty() {
                        e.message.clone()
                    } else {
                        format!("{}\n\n{}", e.message, hint)
                    };
                    let _ = tx.send(AppEvent::ConnectError {
                        conn_id,
                        message: msg,
                    });
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn test_connection(&mut self, ctx: egui::Context) {
        use crate::pages::connections::state::TestStatus;
        self.conn_page.test_status = TestStatus::Testing;
        let conn = self.conn_page.form.to_connection();
        tracing::debug!(conn_id = %conn.id, host = %conn.host, "testing connection");
        let conn_id = conn.id.clone();
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            let start = std::time::Instant::now();
            match crate::core::drivers::connect(&conn).await {
                Ok(_) => {
                    let latency_ms = start.elapsed().as_millis() as u64;
                    let _ = tx.send(AppEvent::TestOk {
                        conn_id,
                        latency_ms,
                    });
                }
                Err(e) => {
                    let hint = e.install_hint().unwrap_or("").to_string();
                    let msg = if hint.is_empty() {
                        e.message
                    } else {
                        format!("{}\n\n{}", e.message, hint)
                    };
                    let _ = tx.send(AppEvent::TestError {
                        conn_id,
                        message: msg,
                    });
                }
            }
            ctx.request_repaint();
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_rows(
        &self,
        ctx: egui::Context,
        tab_id: String,
        db: String,
        schema: String,
        table: String,
        limit: u32,
        offset: u32,
    ) {
        tracing::debug!(
            tab_id = %tab_id, db = %db, schema = %schema,
            table = %table, limit, offset, "fetching rows"
        );
        let conn = self.db_conns.get(&db).cloned();
        let base_config = self.conn_config.clone();
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            let conn = match conn {
                Some(c) => c,
                None => match Self::open_db_conn(base_config, &db, &tx).await {
                    Some(c) => c,
                    None => {
                        let _ = tx.send(AppEvent::RowLoadError {
                            tab_id,
                            message: format!("no connection available for database '{db}'"),
                        });
                        ctx.request_repaint();
                        return;
                    }
                },
            };
            match conn.fetch_rows(&db, &schema, &table, limit, offset).await {
                Ok(result) => {
                    let _ = tx.send(AppEvent::RowsLoaded { tab_id, result });
                }
                Err(e) => {
                    tracing::warn!(error = %e.message, "fetch rows failed");
                    let _ = tx.send(AppEvent::RowLoadError {
                        tab_id,
                        message: e.message,
                    });
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn load_schemas(&self, ctx: egui::Context, db: String) {
        tracing::debug!(db = %db, "load_schemas: requested");
        let Some(base_config) = self.conn_config.clone() else {
            tracing::warn!(db = %db, "load_schemas: no connection config, skipping");
            return;
        };
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            // Open a fresh connection scoped to the target database.
            // PostgreSQL's information_schema is limited to the current connection's
            // database, so reusing the active pool would return the wrong schemas.
            // The connection is dropped at the end of this block (fire-and-forget).
            let mut target = base_config;
            target.database = db.clone();
            tracing::debug!(db = %db, "load_schemas: opening ephemeral connection");
            match crate::core::drivers::connect(&target).await {
                Ok(conn) => {
                    tracing::debug!(db = %db, "load_schemas: starting async fetch");
                    match conn.list_schemas(&db).await {
                        Ok(schemas) => {
                            tracing::info!(db = %db, count = schemas.len(), "load_schemas: success");
                            let _ = tx.send(AppEvent::SchemasLoaded { db, schemas });
                        }
                        Err(e) => {
                            tracing::warn!(db = %db, error = %e.message, "load_schemas: fetch failed");
                        }
                    }
                    // conn drops here — pool closed, connection slot freed
                }
                Err(e) => {
                    tracing::warn!(db = %db, error = %e.message, "load_schemas: connect failed");
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn load_structure(
        &self,
        ctx: egui::Context,
        tab_id: String,
        db: String,
        schema: String,
        table: String,
    ) {
        tracing::debug!(tab_id = %tab_id, db = %db, schema = %schema, table = %table, "load_structure: requested");
        let conn = self.db_conns.get(&db).cloned();
        let base_config = self.conn_config.clone();
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            let conn = match conn {
                Some(c) => c,
                None => match Self::open_db_conn(base_config, &db, &tx).await {
                    Some(c) => c,
                    None => {
                        tracing::warn!(tab_id = %tab_id, db = %db, "load_structure: no connection");
                        ctx.request_repaint();
                        return;
                    }
                },
            };
            match conn.describe_table(&db, &schema, &table).await {
                Ok(columns) => {
                    tracing::debug!(tab_id = %tab_id, column_count = columns.len(), "load_structure: success");
                    let _ = tx.send(AppEvent::StructureLoaded { tab_id, columns });
                }
                Err(e) => {
                    tracing::warn!(tab_id = %tab_id, db = %db, schema = %schema, table = %table, error = %e.message, "load_structure: failed");
                }
            }
            ctx.request_repaint();
        });
    }

    async fn open_db_conn(
        base_config: Option<crate::core::connections::model::Connection>,
        db: &str,
        tx: &mpsc::SyncSender<AppEvent>,
    ) -> Option<Arc<dyn ActiveConnection>> {
        let mut cfg = base_config?;
        cfg.database = db.to_string();
        tracing::debug!(db = %db, "opening connection for database");
        match crate::core::drivers::connect(&cfg).await {
            Ok(conn) => {
                let conn: Arc<dyn ActiveConnection> = Arc::from(conn);
                let _ = tx.send(AppEvent::DbConnected {
                    db: db.to_string(),
                    conn: conn.clone(),
                });
                Some(conn)
            }
            Err(e) => {
                tracing::warn!(db = %db, error = %e.message, "failed to open db connection");
                None
            }
        }
    }

    pub fn save_connection(&mut self) {
        use crate::pages::connections::state::SaveStatus;
        let c = self.conn_page.form.to_connection();
        tracing::debug!(conn_id = %c.id, name = %c.name, "saving connection");
        let _ = self.storage.save(&c);
        self.conn_page.apply_saved(c);
        self.conn_page.save_status = SaveStatus::Saved;
    }

    pub fn delete_connection(&mut self, id: &str) {
        tracing::debug!(conn_id = %id, "deleting connection");
        let _ = self.storage.delete(id);
        self.conn_page.remove(id);
    }

    pub fn duplicate_connection(&mut self) {
        let Some(id) = self.conn_page.selected_id.clone() else {
            tracing::warn!("duplicate_connection: no selected connection");
            return;
        };
        let Some(conn) = self.conn_page.connections.iter().find(|c| c.id == id) else {
            tracing::warn!(conn_id = %id, "duplicate_connection: connection not found");
            return;
        };
        let mut new_conn = conn.clone();
        new_conn.id = uuid::Uuid::new_v4().to_string();
        new_conn.name = format!("{} (copy)", new_conn.name);
        tracing::info!(
            source_id = %id, new_id = %new_conn.id, name = %new_conn.name,
            "connection duplicated"
        );
        let _ = self.storage.save(&new_conn);
        self.conn_page.apply_saved(new_conn);
    }
}


#[cfg(test)]
impl YssvApp {
    fn new_for_test() -> Self {
        let storage = crate::core::connections::storage::Storage::open_in_memory().unwrap();
        let connections = storage.load_all().unwrap_or_default();
        let conn_page = crate::pages::connections::state::ConnectionsPageState::new(connections);
        let (tx, rx) = mpsc::sync_channel(32);
        let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
        Self {
            screen: Screen::Connections,
            settings: crate::pages::settings::state::SettingsState::default(),
            conn_page,
            explorer: None,
            error_modal: None,
            db_conns: HashMap::new(),
            conn_config: None,
            storage,
            rt,
            event_tx: tx,
            event_rx: rx,
        }
    }

    fn send_and_drain(&mut self, event: AppEvent) {
        let _ = self.event_tx.send(event);
        self.drain_events(&egui::Context::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        connections::model::Connection,
        drivers::DbError,
        results::model::{ColumnDef, QueryResult},
        schema::model::{DbInfo, SchemaInfo, TableInfo},
    };
    use crate::events::AppEvent;
    use crate::pages::connections::state::TestStatus;
    use crate::pages::explorer::state::TableTab;

    struct MockConn;

    #[async_trait::async_trait]
    impl ActiveConnection for MockConn {
        async fn current_database(&self) -> Result<String, DbError> {
            Ok("testdb".into())
        }
        async fn list_databases(&self) -> Result<Vec<String>, DbError> {
            Ok(vec![])
        }
        async fn list_schemas(&self, _db: &str) -> Result<Vec<SchemaInfo>, DbError> {
            Ok(vec![])
        }
        async fn list_tables(&self, _db: &str, _schema: &str) -> Result<Vec<TableInfo>, DbError> {
            Ok(vec![])
        }
        async fn fetch_rows(
            &self, _db: &str, _schema: &str, _table: &str, _limit: u32, _offset: u32,
        ) -> Result<QueryResult, DbError> {
            Ok(QueryResult::empty())
        }
        async fn describe_table(
            &self, _db: &str, _schema: &str, _table: &str,
        ) -> Result<Vec<ColumnDef>, DbError> {
            Ok(vec![])
        }
    }

    fn make_app_with_explorer() -> (YssvApp, String) {
        let mut app = YssvApp::new_for_test();
        let db = "testdb".to_string();
        let databases = vec![DbInfo { name: db.clone(), schemas: vec![] }];
        app.explorer = Some(crate::pages::explorer::state::ExplorerState::new(
            "conn1".into(), "Test".into(), &db, databases,
        ));
        (app, db)
    }

    #[test]
    fn connect_error_sets_modal_and_clears_conns() {
        let mut app = YssvApp::new_for_test();
        app.db_conns.insert("db".into(), Arc::new(MockConn));
        app.send_and_drain(AppEvent::ConnectError {
            conn_id: "c1".into(),
            message: "refused".into(),
        });
        assert_eq!(app.error_modal.as_deref(), Some("refused"));
        assert!(app.db_conns.is_empty());
        assert!(matches!(app.conn_page.test_status, TestStatus::Idle));
    }

    #[test]
    fn test_ok_sets_status() {
        let mut app = YssvApp::new_for_test();
        app.send_and_drain(AppEvent::TestOk { conn_id: "c1".into(), latency_ms: 42 });
        assert!(matches!(app.conn_page.test_status, TestStatus::Ok(42)));
    }

    #[test]
    fn test_error_sets_status() {
        let mut app = YssvApp::new_for_test();
        app.send_and_drain(AppEvent::TestError {
            conn_id: "c1".into(),
            message: "bad creds".into(),
        });
        assert!(matches!(
            &app.conn_page.test_status,
            TestStatus::Failed(m) if m == "bad creds"
        ));
    }

    #[test]
    fn connected_switches_to_explorer_screen() {
        let mut app = YssvApp::new_for_test();
        let conn: Arc<dyn ActiveConnection> = Arc::new(MockConn);
        app.send_and_drain(AppEvent::Connected {
            conn_id: "c1".into(),
            conn_name: "Local".into(),
            default_db: "testdb".into(),
            databases: vec![DbInfo { name: "testdb".into(), schemas: vec![] }],
            connection: conn,
            conn_config: Box::new(Connection::new_postgres()),
        });
        assert!(matches!(app.screen, Screen::Explorer));
        assert!(app.explorer.is_some());
        assert!(app.db_conns.contains_key("testdb"));
    }

    #[test]
    fn rows_loaded_updates_tab_result() {
        let (mut app, db) = make_app_with_explorer();
        let tab = TableTab::new("users", "public", &db);
        let tab_id = tab.id.clone();
        app.explorer.as_mut().unwrap().tabs.tabs.push(tab);

        let result = QueryResult {
            columns: vec![],
            rows: vec![vec![Some("1".into())]],
            total_rows: Some(1),
        };
        app.send_and_drain(AppEvent::RowsLoaded { tab_id: tab_id.clone(), result });

        let tab = app.explorer.unwrap().tabs.tabs.into_iter().find(|t| t.id == tab_id).unwrap();
        assert!(tab.result.is_some());
        assert!(!tab.loading);
    }

    #[test]
    fn row_load_error_sets_modal_and_clears_tab_result() {
        let (mut app, db) = make_app_with_explorer();
        let tab = TableTab::new("users", "public", &db);
        let tab_id = tab.id.clone();
        app.explorer.as_mut().unwrap().tabs.tabs.push(tab);

        app.send_and_drain(AppEvent::RowLoadError {
            tab_id: tab_id.clone(),
            message: "query failed".into(),
        });

        assert_eq!(app.error_modal.as_deref(), Some("query failed"));
        let tab = app.explorer.unwrap().tabs.tabs.into_iter().find(|t| t.id == tab_id).unwrap();
        assert!(!tab.loading);
        assert!(tab.result.is_none());
    }

    #[test]
    fn schemas_loaded_updates_explorer_database() {
        let (mut app, db) = make_app_with_explorer();
        let schemas = vec![SchemaInfo { name: "public".into(), tables: vec![] }];
        app.send_and_drain(AppEvent::SchemasLoaded { db: db.clone(), schemas });

        let explorer = app.explorer.unwrap();
        let db_info = explorer.databases.iter().find(|d| d.name == db).unwrap();
        assert_eq!(db_info.schemas.len(), 1);
        assert_eq!(db_info.schemas[0].name, "public");
    }

    #[test]
    fn schemas_loaded_with_no_explorer_is_noop() {
        let mut app = YssvApp::new_for_test();
        app.send_and_drain(AppEvent::SchemasLoaded {
            db: "ghost".into(),
            schemas: vec![],
        });
        assert!(app.explorer.is_none());
    }

    #[test]
    fn structure_loaded_updates_tab() {
        let (mut app, db) = make_app_with_explorer();
        let tab = TableTab::new("orders", "public", &db);
        let tab_id = tab.id.clone();
        app.explorer.as_mut().unwrap().tabs.tabs.push(tab);

        let columns = vec![ColumnDef {
            name: "id".into(),
            data_type: "int4".into(),
            is_pk: true,
            is_fk: false,
            nullable: false,
        }];
        app.send_and_drain(AppEvent::StructureLoaded { tab_id: tab_id.clone(), columns });

        let tab = app.explorer.unwrap().tabs.tabs.into_iter().find(|t| t.id == tab_id).unwrap();
        assert!(!tab.structure_loading);
        assert_eq!(tab.structure.unwrap().len(), 1);
    }

    #[test]
    fn db_connected_caches_connection() {
        let mut app = YssvApp::new_for_test();
        let conn: Arc<dyn ActiveConnection> = Arc::new(MockConn);
        app.send_and_drain(AppEvent::DbConnected { db: "analytics".into(), conn });
        assert!(app.db_conns.contains_key("analytics"));
    }

    #[test]
    fn save_connection_persists_and_updates_page() {
        let mut app = YssvApp::new_for_test();
        app.conn_page.form.name = "Prod".into();
        app.conn_page.form.host = "db.prod.io".into();
        app.save_connection();
        assert!(!app.conn_page.connections.is_empty());
        assert_eq!(app.conn_page.connections[0].name, "Prod");
    }

    #[test]
    fn delete_connection_removes_from_page() {
        let mut app = YssvApp::new_for_test();
        app.conn_page.form.name = "Dev".into();
        app.conn_page.form.host = "localhost".into();
        app.save_connection();
        let id = app.conn_page.connections[0].id.clone();
        app.delete_connection(&id);
        assert!(app.conn_page.connections.is_empty());
    }

    #[test]
    fn duplicate_connection_creates_copy_with_new_id() {
        let mut app = YssvApp::new_for_test();
        app.conn_page.form.name = "Staging".into();
        app.conn_page.form.host = "staging.db".into();
        app.save_connection();
        let original_id = app.conn_page.connections[0].id.clone();
        app.conn_page.selected_id = Some(original_id.clone());
        app.duplicate_connection();
        assert_eq!(app.conn_page.connections.len(), 2);
        let copy = app.conn_page.connections.iter().find(|c| c.id != original_id).unwrap();
        assert!(copy.name.contains("copy"));
        assert_ne!(copy.id, original_id);
    }

    #[test]
    fn duplicate_connection_without_selection_is_noop() {
        let mut app = YssvApp::new_for_test();
        app.conn_page.selected_id = None;
        app.duplicate_connection();
        assert!(app.conn_page.connections.is_empty());
    }
}

impl eframe::App for YssvApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain_events(&ctx);

        crate::ui::error_modal::render(ui, self);

        match self.screen {
            Screen::Connections => crate::pages::connections::render(ui, self),
            Screen::Explorer => crate::pages::explorer::render(ui, self),
        }
    }
}
