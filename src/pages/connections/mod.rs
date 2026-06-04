pub mod state;
pub use state::ConnectionsPageState;

use egui::RichText;
use crate::theme::ThemeColors;

pub fn render_list(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, _ctx: &egui::Context) {
    use crate::ui::atoms::button::small_primary_button;
    use crate::ui::molecules::conn_item::conn_item;

    let tc = ThemeColors::from_ui(ui);

    // Header — 16px top, 14px sides
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 14, right: 14, top: 16, bottom: 10 })
        .show(ui, |ui| {
            ui.label(RichText::new("Connections").size(14.0).strong().color(tc.text));
            ui.add_space(10.0);
            // Search row
            ui.horizontal(|ui| {
                let available = ui.available_width();
                egui::TextEdit::singleline(&mut app.conn_page.search_query)
                    .hint_text("Search…")
                    .desired_width(available - 52.0)
                    .show(ui);
                ui.add_space(4.0);
                if small_primary_button(ui, "+ New").clicked() {
                    app.conn_page.start_new();
                }
            });
        });

    // Border below header
    let sep_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), 1.0),
    );
    ui.painter().rect_filled(sep_rect, egui::CornerRadius::ZERO, tc.border_faint);
    ui.add_space(1.0);

    let groups = app.conn_page.grouped_connections();
    let groups_owned: Vec<(String, Vec<String>)> = groups
        .iter()
        .map(|(g, items)| (g.clone(), items.iter().map(|c| c.id.clone()).collect()))
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(6.0);
        for (group_name, conn_ids) in groups_owned {
            let collapsed = app.conn_page.collapsed_groups.contains(&group_name);

            // Group label — 11px, uppercase, text-faint
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                let chev = if collapsed { "▸" } else { "▾" };
                let header_resp = ui.add(
                    egui::Button::new(
                        RichText::new(format!("{} {}", chev, group_name.to_uppercase()))
                            .size(11.0)
                            .color(tc.text_faint)
                            .strong(),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
                );
                if header_resp.clicked() {
                    if collapsed {
                        app.conn_page.collapsed_groups.remove(&group_name);
                    } else {
                        app.conn_page.collapsed_groups.insert(group_name.clone());
                    }
                }
            });

            if !collapsed {
                ui.add_space(2.0);
                // Indent items with side padding
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 6, right: 6, top: 0, bottom: 0 })
                    .show(ui, |ui| {
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
                    });
            }
        }
        ui.add_space(14.0);
    });
}

pub fn render_detail(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    use crate::pages::connections::state::TestStatus;
    use crate::ui::atoms::button::primary_button;
    use crate::ui::atoms::input::{password_input, text_input};
    use crate::ui::molecules::color_picker::color_picker;
    use crate::core::connections::model::DbEngine;

    let tc = ThemeColors::from_ui(ui);

    egui::ScrollArea::vertical().show(ui, |ui| {
        let available = ui.available_width();
        let content_w = (720.0_f32).min(available - 80.0);
        let h_pad = ((available - content_w) / 2.0).max(40.0);

        egui::Frame::new()
            .inner_margin(egui::Margin { left: h_pad as i8, right: h_pad as i8, top: 30, bottom: 28 })
            .show(ui, |ui| {
                ui.set_max_width(content_w);

                // Name + Group
                field_label(ui, "Connection name", &tc);
                text_input(ui, &mut app.conn_page.form.name, "My Database");
                ui.add_space(14.0);

                field_label(ui, "Group", &tc);
                text_input(ui, &mut app.conn_page.form.group, "Local");
                ui.add_space(14.0);

                // Color label
                field_label(ui, "Color label", &tc);
                color_picker(ui, &mut app.conn_page.form.color);
                ui.add_space(14.0);

                // Engine selection
                field_label(ui, "Database engine", &tc);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for engine in [DbEngine::Postgres, DbEngine::MySQL] {
                        let selected = app.conn_page.form.engine == engine;
                        let (bg, border) = if selected {
                            (tc.accent_soft, tc.accent)
                        } else {
                            (tc.bg_base, tc.border_strong)
                        };
                        let btn = egui::Button::new(
                            RichText::new(engine.label())
                                .size(13.0)
                                .color(if selected { tc.accent } else { tc.text }),
                        )
                        .fill(bg)
                        .stroke(egui::Stroke::new(1.5, border))
                        .min_size(egui::vec2(120.0, 40.0));
                        if ui.add(btn).clicked() {
                            app.conn_page.form.engine = engine;
                            app.conn_page.form.port = engine.default_port().to_string();
                        }
                    }
                });
                ui.add_space(4.0);

                // ── CONNECTION ──────────────────────────────────────
                section_label(ui, "CONNECTION", &tc);

                // Host + Port (two columns)
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 100.0);
                        field_label(ui, "Host", &tc);
                        text_input(ui, &mut app.conn_page.form.host, "127.0.0.1");
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        field_label(ui, "Port", &tc);
                        text_input(ui, &mut app.conn_page.form.port, "5432");
                    });
                });
                ui.add_space(14.0);

                field_label(ui, "Database", &tc);
                text_input(ui, &mut app.conn_page.form.database, "postgres");
                ui.add_space(14.0);

                field_label(ui, "Username", &tc);
                text_input(ui, &mut app.conn_page.form.username, "postgres");
                ui.add_space(14.0);

                field_label(ui, "Password", &tc);
                password_input(ui, &mut app.conn_page.form.password, "");
                ui.add_space(4.0);

                // ── SSH TUNNEL ──────────────────────────────────────
                section_label(ui, "SSH TUNNEL", &tc);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Enable SSH tunnel").size(13.0).color(tc.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut app.conn_page.form.ssh_enabled, "");
                    });
                });

                if app.conn_page.form.ssh_enabled {
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 80.0);
                            field_label(ui, "SSH Host", &tc);
                            text_input(ui, &mut app.conn_page.form.ssh_host, "bastion.example.com");
                        });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            field_label(ui, "Port", &tc);
                            text_input(ui, &mut app.conn_page.form.ssh_port, "22");
                        });
                    });
                    ui.add_space(14.0);

                    field_label(ui, "SSH Username", &tc);
                    text_input(ui, &mut app.conn_page.form.ssh_username, "admin");
                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Use key file").size(13.0).color(tc.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut app.conn_page.form.ssh_use_key, "");
                        });
                    });
                    ui.add_space(10.0);

                    if app.conn_page.form.ssh_use_key {
                        field_label(ui, "Key file path", &tc);
                        text_input(ui, &mut app.conn_page.form.ssh_key_path, "/home/user/.ssh/id_rsa");
                    } else {
                        field_label(ui, "SSH Password", &tc);
                        password_input(ui, &mut app.conn_page.form.ssh_password, "");
                    }
                    ui.add_space(4.0);
                }

                // ── Actions ─────────────────────────────────────────
                ui.add_space(24.0);
                ui.separator();
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    // Test connection
                    let (test_label, test_color) = match &app.conn_page.test_status {
                        TestStatus::Idle      => ("Test Connection", tc.text_muted),
                        TestStatus::Testing   => ("Testing…",        tc.text_faint),
                        TestStatus::Ok        => ("✓ Connected",     tc.ok),
                        TestStatus::Failed(_) => ("✗ Failed",        tc.err),
                    };
                    let test_btn = egui::Button::new(
                        RichText::new(test_label).size(13.0).color(test_color)
                    )
                    .fill(tc.bg_base)
                    .stroke(egui::Stroke::new(1.0, tc.border_strong))
                    .min_size(egui::vec2(0.0, 32.0));

                    if ui.add(test_btn).clicked()
                        && app.conn_page.test_status != TestStatus::Testing
                    {
                        app.test_connection(ctx.clone());
                    }
                    if let TestStatus::Failed(msg) = &app.conn_page.test_status.clone() {
                        app.error_modal = Some(msg.clone());
                    }

                    ui.add_space(8.0);

                    let save_btn = egui::Button::new(RichText::new("Save").size(13.0).color(tc.text))
                        .fill(tc.bg_base)
                        .stroke(egui::Stroke::new(1.0, tc.border_strong))
                        .min_size(egui::vec2(0.0, 32.0));
                    if ui.add(save_btn).clicked() {
                        app.save_connection();
                    }

                    ui.add_space(8.0);

                    if primary_button(ui, "Connect").clicked() {
                        app.save_connection();
                        app.connect(ctx.clone());
                    }
                });
            });
    });
}

fn field_label(ui: &mut egui::Ui, text: &str, tc: &ThemeColors) {
    ui.label(
        RichText::new(text)
            .size(11.5)
            .color(tc.text_muted)
            .strong(),
    );
    ui.add_space(4.0);
}

fn section_label(ui: &mut egui::Ui, text: &str, tc: &ThemeColors) {
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text)
                .size(11.0)
                .color(tc.text_faint)
                .strong(),
        );
        ui.add_space(8.0);
        let r = ui.available_rect_before_wrap();
        let y = r.center().y;
        ui.painter().hline(r.x_range(), y, egui::Stroke::new(1.0, tc.border));
        ui.allocate_space(r.size());
    });
    ui.add_space(12.0);
}
