use crate::theme::ThemeColors;
use egui::RichText;

pub fn render_detail(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let ctx = ui.ctx().clone();
    use crate::core::connections::model::DbEngine;
    use crate::pages::connections::state::TestStatus;
    use crate::ui::atoms::button::primary_button;
    use crate::ui::atoms::input::{password_input, text_input};
    use crate::ui::atoms::light_switch::light_switch;
    use crate::ui::molecules::color_picker::color_picker;

    let tc = ThemeColors::from_ui(ui);

    egui::ScrollArea::vertical().show(ui, |ui| {
        let available = ui.available_width();
        let content_w = (720.0_f32).min(available - 80.0);
        let h_pad = ((available - content_w) / 2.0).max(40.0);

        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: h_pad.clamp(0.0, i8::MAX as f32) as i8,
                right: h_pad.clamp(0.0, i8::MAX as f32) as i8,
                top: 30,
                bottom: 28,
            })
            .show(ui, |ui| {
                field_label(ui, "Connection name", &tc);
                text_input(ui, &mut app.conn_page.form.name, "My Database");
                ui.add_space(14.0);

                field_label(ui, "Group", &tc);
                text_input(ui, &mut app.conn_page.form.group, "Local");
                ui.add_space(14.0);

                field_label(ui, "Color label", &tc);
                color_picker(ui, &mut app.conn_page.form.color);
                ui.add_space(14.0);

                field_label(ui, "Database engine", &tc);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for engine in [DbEngine::Postgres, DbEngine::MySQL] {
                        let selected = app.conn_page.form.engine == engine;
                        let (bg, border) = if selected {
                            (tc.primary_muted, tc.primary)
                        } else {
                            (tc.background, tc.input)
                        };
                        let btn = egui::Button::new(
                            RichText::new(engine.label()).size(13.0).color(if selected {
                                tc.primary
                            } else {
                                tc.foreground
                            }),
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

                section_label(ui, "CONNECTION", &tc);

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

                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 10.0) / 2.0;
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        field_label(ui, "Username", &tc);
                        text_input(ui, &mut app.conn_page.form.username, "postgres");
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        field_label(ui, "Password", &tc);
                        password_input(ui, &mut app.conn_page.form.password, "");
                    });
                });
                ui.add_space(14.0);

                section_label(ui, "SSH TUNNEL", &tc);

                egui::Frame::default()
                    .stroke(egui::Stroke::new(0.5, egui::Color32::GRAY))
                    .inner_margin(20.0)
                    .corner_radius(5.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("").size(24.0));
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Connect through SSH").strong());
                                ui.label(
                                    egui::RichText::new(
                                        "Traffic is tunneled through SSH to the database server.",
                                    )
                                    .small()
                                    .weak(),
                                );
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                light_switch(ui, &mut app.conn_page.form.ssh_enabled);
                            });
                        });

                        if app.conn_page.form.ssh_enabled {
                            ui.add_space(14.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width() - 80.0);
                                    field_label(ui, "SSH Host", &tc);
                                    text_input(
                                        ui,
                                        &mut app.conn_page.form.ssh_host,
                                        "bastion.example.com",
                                    );
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
                                ui.label(
                                    RichText::new("Use key file").size(13.0).color(tc.foreground),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.checkbox(&mut app.conn_page.form.ssh_use_key, "");
                                    },
                                );
                            });
                            ui.add_space(10.0);

                            if app.conn_page.form.ssh_use_key {
                                field_label(ui, "Key file path", &tc);
                                text_input(
                                    ui,
                                    &mut app.conn_page.form.ssh_key_path,
                                    "/home/user/.ssh/id_rsa",
                                );
                            } else {
                                field_label(ui, "SSH Password", &tc);
                                password_input(ui, &mut app.conn_page.form.ssh_password, "");
                            }
                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    let (test_label, test_color) = match &app.conn_page.test_status {
                        TestStatus::Idle => ("Test Connection", tc.muted_foreground),
                        TestStatus::Testing => ("Testing…", tc.subtle_foreground),
                        TestStatus::Ok => ("✓ Connected", tc.success),
                        TestStatus::Failed(_) => ("✗ Failed", tc.destructive),
                    };
                    let test_btn =
                        egui::Button::new(RichText::new(test_label).size(13.0).color(test_color))
                            .fill(tc.background)
                            .stroke(egui::Stroke::new(1.0, tc.input))
                            .min_size(egui::vec2(0.0, 32.0));

                    if ui.add(test_btn).clicked()
                        && app.conn_page.test_status != TestStatus::Testing
                    {
                        app.test_connection(ctx.clone());
                    }
                    if let TestStatus::Failed(msg) = &app.conn_page.test_status {
                        let msg = msg.clone();
                        app.error_modal = Some(msg);
                    }

                    ui.add_space(8.0);

                    let save_btn =
                        egui::Button::new(RichText::new("Save").size(13.0).color(tc.foreground))
                            .fill(tc.background)
                            .stroke(egui::Stroke::new(1.0, tc.input))
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
    ui.label(RichText::new(text).size(11.5).color(tc.muted_foreground).strong());
    ui.add_space(4.0);
}

fn section_label(ui: &mut egui::Ui, text: &str, tc: &ThemeColors) {
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(text).size(11.0).color(tc.subtle_foreground).strong());
        ui.add_space(8.0);
        let r = ui.available_rect_before_wrap();
        let y = r.center().y;
        ui.painter()
            .hline(r.x_range(), y, egui::Stroke::new(1.0, tc.border));
        ui.allocate_space(r.size());
    });
    ui.add_space(12.0);
}

