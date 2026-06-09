use std::sync::{Arc, mpsc};

use crate::core::{
    connections::storage::Storage, drivers::ActiveConnection, results::model::QueryResult,
    schema::model::DbInfo,
};
use crate::pages::{
    connections::state::ConnectionsPageState, explorer::state::ExplorerState,
    settings::state::SettingsState,
};
use crate::theme;

#[derive(Debug)]
pub enum AppEvent {
    Connected {
        conn_id: String,
        conn_name: String,
        databases: Vec<DbInfo>,
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
    pub active_conn: Option<Arc<dyn ActiveConnection>>,
    pub error_modal: Option<String>,

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
            active_conn: None,
            error_modal: None,
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

    fn apply_event(&mut self, event: AppEvent, _ctx: &egui::Context) {
        match event {
            AppEvent::Connected {
                conn_id,
                conn_name,
                databases,
            } => {
                tracing::info!(conn_id = %conn_id, conn = %conn_name, databases = databases.len(), "connected");
                self.explorer = Some(ExplorerState::new(conn_id, conn_name, databases));
                self.screen = Screen::Explorer;
            }
            AppEvent::ConnectError { conn_id, message } => {
                tracing::warn!(conn_id = %conn_id, error = %message, "connection failed");
                self.error_modal = Some(message);
                self.conn_page.test_status = crate::pages::connections::state::TestStatus::Idle;
            }
            AppEvent::TestOk { conn_id } => {
                tracing::debug!(conn_id = %conn_id, "test connection ok");
                if self.conn_page.selected_id.as_deref() == Some(&conn_id) {
                    self.conn_page.test_status = crate::pages::connections::state::TestStatus::Ok;
                }
            }
            AppEvent::TestError { conn_id, message } => {
                tracing::warn!(conn_id = %conn_id, error = %message, "test connection failed");
                if self.conn_page.selected_id.as_deref() == Some(&conn_id) {
                    self.conn_page.test_status =
                        crate::pages::connections::state::TestStatus::Failed(message);
                }
            }
            AppEvent::RowsLoaded { tab_id, result } => {
                tracing::debug!(tab_id = %tab_id, rows = result.rows.len(), "rows loaded");
                if let Some(explorer) = &mut self.explorer
                    && let Some(tab) = explorer.tabs.tabs.iter_mut().find(|t| t.id == tab_id) {
                        tab.result = Some(result);
                        tab.loading = false;
                    }
            }
            AppEvent::RowLoadError { tab_id, message } => {
                tracing::warn!(tab_id = %tab_id, error = %message, "row load failed");
                if let Some(explorer) = &mut self.explorer
                    && let Some(tab) = explorer.tabs.tabs.iter_mut().find(|t| t.id == tab_id) {
                        tab.loading = false;
                        tab.result = None;
                    }
                self.error_modal = Some(message);
            }
            AppEvent::SchemasLoaded { db, schemas } => {
                tracing::debug!(db = %db, schemas = schemas.len(), "schemas loaded");
                if let Some(explorer) = &mut self.explorer
                    && let Some(db_info) = explorer.databases.iter_mut().find(|d| d.name == db) {
                        db_info.schemas = schemas;
                    }
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
            match crate::core::drivers::connect(&conn).await {
                Ok(active) => match active.list_databases().await {
                    Ok(dbs) => {
                        let databases = dbs
                            .into_iter()
                            .map(|name| crate::core::schema::model::DbInfo {
                                name,
                                schemas: vec![],
                            })
                            .collect();
                        let _ = tx.send(AppEvent::Connected {
                            conn_id,
                            conn_name,
                            databases,
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
                },
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
        if let Some(conn) = &self.active_conn {
            tracing::debug!(
                tab_id = %tab_id, db = %db, schema = %schema,
                table = %table, limit, offset, "fetching rows"
            );
            let conn = conn.clone();
            let tx = self.event_tx.clone();
            self.rt.spawn(async move {
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
    }

    pub fn save_connection(&mut self) {
        let c = self.conn_page.form.to_connection();
        tracing::debug!(conn_id = %c.id, name = %c.name, "saving connection");
        let _ = self.storage.save(&c);
        self.conn_page.apply_saved(c);
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
