pub mod state;
pub use state::ConnectionsPageState;

pub fn render(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    // Use egui panels to avoid simultaneous mutable borrow in two closures
    egui::SidePanel::left("conn_list_panel")
        .exact_width(280.0)
        .resizable(false)
        .show_inside(ui, |ui| render_list(ui, app, ctx));
    egui::CentralPanel::default()
        .show_inside(ui, |ui| render_detail(ui, app, ctx));
}

fn render_list(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, _ctx: &egui::Context) {
    use crate::ui::{atoms::button::small_primary_button, molecules::conn_item::conn_item};
    use egui::RichText;

    // Header
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new("Connections").size(14.0).strong());
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        egui::TextEdit::singleline(&mut app.conn_page.search_query)
            .hint_text("Search connections…")
            .desired_width(ui.available_width() - 60.0)
            .show(ui);
        if small_primary_button(ui, "+ New").clicked() {
            app.conn_page.start_new();
        }
    });
    ui.add_space(8.0);

    // Groups
    let groups = app.conn_page.grouped_connections();
    let groups_owned: Vec<(String, Vec<String>)> = groups
        .iter()
        .map(|(g, items)| (g.clone(), items.iter().map(|c| c.id.clone()).collect()))
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (group_name, conn_ids) in groups_owned {
            let collapsed = app.conn_page.collapsed_groups.contains(&group_name);
            let header_text = format!(
                "{} {} ({})",
                if collapsed { "▸" } else { "▾" },
                group_name,
                conn_ids.len()
            );
            ui.add_space(4.0);
            let header_resp = ui.add(
                egui::Button::new(RichText::new(&header_text).size(11.0).strong())
                    .fill(egui::Color32::TRANSPARENT),
            );
            if header_resp.clicked() {
                if collapsed {
                    app.conn_page.collapsed_groups.remove(&group_name);
                } else {
                    app.conn_page.collapsed_groups.insert(group_name.clone());
                }
            }

            if !collapsed {
                for id in &conn_ids {
                    if let Some(c) = app.conn_page.connections.iter().find(|x| &x.id == id) {
                        let selected = app.conn_page.selected_id.as_deref() == Some(id);
                        let c_clone = c.clone();
                        let resp = conn_item(ui, &c_clone, selected);
                        if resp.clicked() {
                            let id_clone = id.clone();
                            app.conn_page.select(&id_clone);
                        }
                    }
                }
            }
        }
    });
}

fn render_detail(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    use crate::pages::connections::state::TestStatus;
    use crate::ui::{
        atoms::{button::primary_button, input::{password_input, text_input}},
        molecules::color_picker::color_picker,
    };
    use egui::RichText;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.vertical(|ui| {
                // Name field
                ui.label(RichText::new("Connection name").size(11.0).strong());
                text_input(ui, &mut app.conn_page.form.name, "My Database");
                ui.add_space(12.0);

                // Group field
                ui.label(RichText::new("Group").size(11.0).strong());
                text_input(ui, &mut app.conn_page.form.group, "Local");
                ui.add_space(12.0);

                // Color
                ui.label(RichText::new("Color label").size(11.0).strong());
                color_picker(ui, &mut app.conn_page.form.color);
                ui.add_space(12.0);

                // Engine selector
                ui.label(RichText::new("Database engine").size(11.0).strong());
                ui.horizontal(|ui| {
                    use crate::core::connections::model::DbEngine;
                    for engine in [DbEngine::Postgres, DbEngine::MySQL] {
                        let selected = app.conn_page.form.engine == engine;
                        let btn = egui::Button::new(
                            RichText::new(engine.label()).size(12.0)
                        )
                        .fill(if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            ui.visuals().widgets.inactive.bg_fill
                        });
                        if ui.add(btn).clicked() {
                            app.conn_page.form.engine = engine;
                            app.conn_page.form.port = engine.default_port().to_string();
                        }
                    }
                });
                ui.add_space(12.0);

                // Connection fields
                ui.label(RichText::new("Connection").size(11.0).strong());
                ui.label(RichText::new("Host").size(11.0));
                text_input(ui, &mut app.conn_page.form.host, "127.0.0.1");
                ui.label(RichText::new("Port").size(11.0));
                text_input(ui, &mut app.conn_page.form.port, "5432");
                ui.label(RichText::new("Database").size(11.0));
                text_input(ui, &mut app.conn_page.form.database, "postgres");
                ui.label(RichText::new("Username").size(11.0));
                text_input(ui, &mut app.conn_page.form.username, "postgres");
                ui.label(RichText::new("Password").size(11.0));
                password_input(ui, &mut app.conn_page.form.password, "");
                ui.add_space(12.0);

                // SSH section
                ui.horizontal(|ui| {
                    ui.label(RichText::new("SSH Tunnel").size(11.0).strong());
                    ui.checkbox(&mut app.conn_page.form.ssh_enabled, "Enable");
                });
                if app.conn_page.form.ssh_enabled {
                    ui.add_space(4.0);
                    ui.label(RichText::new("SSH Host").size(11.0));
                    text_input(ui, &mut app.conn_page.form.ssh_host, "bastion.example.com");
                    ui.label(RichText::new("SSH Port").size(11.0));
                    text_input(ui, &mut app.conn_page.form.ssh_port, "22");
                    ui.label(RichText::new("SSH Username").size(11.0));
                    text_input(ui, &mut app.conn_page.form.ssh_username, "admin");
                    ui.checkbox(&mut app.conn_page.form.ssh_use_key, "Use key file");
                    if app.conn_page.form.ssh_use_key {
                        ui.label(RichText::new("Key file path").size(11.0));
                        text_input(ui, &mut app.conn_page.form.ssh_key_path, "/home/user/.ssh/id_rsa");
                    } else {
                        ui.label(RichText::new("SSH Password").size(11.0));
                        password_input(ui, &mut app.conn_page.form.ssh_password, "");
                    }
                }
                ui.add_space(16.0);

                // Footer actions
                ui.horizontal(|ui| {
                    // Test connection
                    let test_label = match &app.conn_page.test_status {
                        TestStatus::Idle    => "Test Connection",
                        TestStatus::Testing => "Testing…",
                        TestStatus::Ok      => "✓ Connected",
                        TestStatus::Failed(_) => "✗ Failed",
                    };
                    let test_color = match &app.conn_page.test_status {
                        TestStatus::Ok       => egui::Color32::from_rgb(0x2e, 0xa3, 0x6b),
                        TestStatus::Failed(_) => egui::Color32::from_rgb(0xe5, 0x48, 0x4d),
                        _                    => ui.visuals().text_color(),
                    };
                    if ui.add(egui::Button::new(
                        RichText::new(test_label).color(test_color)
                    )).clicked()
                        && app.conn_page.test_status != TestStatus::Testing
                    {
                        app.test_connection(ctx.clone());
                    }
                    if let TestStatus::Failed(msg) = &app.conn_page.test_status.clone() {
                        app.error_modal = Some(msg.clone());
                    }

                    ui.add_space(8.0);

                    // Save
                    if ui.button("Save").clicked() {
                        app.save_connection();
                    }

                    ui.add_space(8.0);

                    // Connect
                    if primary_button(ui, "Connect").clicked() {
                        app.save_connection();
                        app.connect(ctx.clone());
                    }
                });
            });
        });
    });
}
