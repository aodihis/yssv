use std::collections::HashMap;
use std::sync::{Arc, mpsc};

use crate::core::{
    connections::storage::Storage,
    drivers::ActiveConnection,
    results::model::{ColumnDef, QueryResult},
    schema::model::DbInfo,
};
use crate::pages::{
    connections::state::ConnectionsPageState, explorer::state::ExplorerState,
    settings::state::SettingsState,
};
use crate::theme;

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
            AppEvent::TestOk { conn_id } => {
                tracing::debug!(conn_id = %conn_id, "test connection ok");
                self.conn_page.test_status = crate::pages::connections::state::TestStatus::Ok;
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
            match crate::core::drivers::connect(&conn).await {
                Ok(_) => {
                    let _ = tx.send(AppEvent::TestOk { conn_id });
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
}

fn data_dir_path() -> String {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        format!("{}\\yssv\\connections.db", base)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/.config/yssv/connections.db", home)
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
