use crate::pages::connections::state::TestStatus;
use crate::theme::ThemeColors;
use crate::ui::atoms::icon::{Icon, icon_image, svg_icon};
use egui::RichText;

pub fn render_detail(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let ctx = ui.ctx().clone();
    use crate::core::connections::model::DbEngine;
    use crate::pages::connections::state::SaveStatus;
    use crate::pages::connections::state::SshAuthMethod;
    use crate::ui::atoms::dropdown::dropdown;
    use crate::ui::atoms::group_input::group_input;
    use crate::ui::atoms::input::{file_input, password_input, text_input};
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
                top: 50,
                bottom: 28,
            })
            .show(ui, |ui| {

                section_label(ui, "GENERAL", &tc);

                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 10.0) / 2.0;
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        field_label(ui, "Connection name", &tc);
                        text_input(ui, &mut app.conn_page.form.name, "My Database", None);
                    });

                    ui.add_space(4.0);

                    ui.vertical(|ui| {
                        field_label(ui, "Group", &tc);
                        let groups = app.conn_page.groups();
                        group_input(ui, &mut app.conn_page.form.group, &groups);
                    });

                });
                ui.add_space(16.0);
                field_label(ui, "Color label", &tc);
                color_picker(ui, &mut app.conn_page.form.color);
                ui.add_space(14.0);

                section_label(ui, "Database Engine", &tc);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for engine in [DbEngine::Postgres, DbEngine::MySQL] {
                        let selected = app.conn_page.form.engine == engine;
                        let (bg, border) = if selected {
                            (tc.button_secondary_bg, tc.button_primary_bg)
                        } else {
                            (tc.background, tc.field_border)
                        };
                        let btn = egui::Button::new(
                            RichText::new(engine.label()).size(13.0).color(if selected {
                                tc.button_primary_bg
                            } else {
                                tc.text_primary
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
                ui.add_space(16.0);

                section_label(ui, "CONNECTION", &tc);

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 100.0);
                        field_label(ui, "Host", &tc);
                        text_input(ui, &mut app.conn_page.form.host, "127.0.0.1", None);
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        field_label(ui, "Port", &tc);
                        text_input(ui, &mut app.conn_page.form.port, "5432", None);
                    });
                });
                ui.add_space(14.0);

                field_label(ui, "Database", &tc);
                text_input(ui, &mut app.conn_page.form.database, "postgres", None);
                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 10.0) / 2.0;
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        field_label(ui, "Username", &tc);
                        text_input(ui, &mut app.conn_page.form.username, "postgres", None);
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        field_label(ui, "Password", &tc);
                        password_input(ui, &mut app.conn_page.form.password, "", None);
                    });
                });
                ui.add_space(14.0);

                section_label(ui, "SSH TUNNEL", &tc);

                egui::Frame::default()
                    .stroke(egui::Stroke::new(1.0, tc.field_border))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        egui::Frame::default()
                            .inner_margin(egui::Margin { left: 16, right: 16, top: 14, bottom: 14 })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    svg_icon(ui, Icon::SquareTerminal, 24.0, tc.text_primary);
                                    ui.add_space(10.0);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Connect through SSH").family(egui::FontFamily::Name("SemiBold".into())));
                                        ui.label(
                                            egui::RichText::new(
                                                "Traffic is tunneled through SSH to the database server.",
                                            )
                                                .small()
                                                .weak(),
                                        );
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            light_switch(ui, &mut app.conn_page.form.ssh_enabled);
                                        },
                                    );
                                });
                            });

                        if app.conn_page.form.ssh_enabled {
                            let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
                            ui.painter().rect_filled(sep, egui::CornerRadius::ZERO, tc.field_border);
                            ui.add_space(1.0);

                            egui::Frame::default()
                                .inner_margin(egui::Margin { left: 16, right: 16, top: 14, bottom: 14 })
                                .fill(tc.surface_secondary)
                                .corner_radius(egui::CornerRadius { nw: 0, ne: 0, sw: 5, se: 5 })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.set_width(ui.available_width() - 100.0);
                                            field_label(ui, "SSH Host", &tc);
                                            text_input(
                                                ui,
                                                &mut app.conn_page.form.ssh_host,
                                                "bastion.example.com",
                                                None,
                                            );
                                        });
                                        ui.add_space(10.0);
                                        ui.vertical(|ui| {
                                            field_label(ui, "Port", &tc);
                                            text_input(ui, &mut app.conn_page.form.ssh_port, "22", None);
                                        });
                                    });
                                    ui.add_space(14.0);

                                    ui.horizontal(|ui| {
                                        let width = (ui.available_width() - 10.0) / 2.0;
                                        ui.vertical(|ui| {
                                            ui.set_width(width);

                                            field_label(ui, "SSH User", &tc);
                                            text_input(ui, &mut app.conn_page.form.ssh_username, "admin", None);
                                        });

                                        ui.add_space(4.0);

                                        ui.vertical(|ui| {
                                            field_label(ui, "Authentication", &tc);
                                            dropdown(
                                                ui,
                                                "ssh_auth_method",
                                                &mut app.conn_page.form.ssh_auth_method,
                                                &[
                                                    (SshAuthMethod::Password, "Password"),
                                                    (SshAuthMethod::KeyFile, "Key File"),
                                                    (SshAuthMethod::Agent, "SSH Agent"),
                                                ],
                                            );
                                        });

                                    });

                                    ui.add_space(14.0);

                                    match app.conn_page.form.ssh_auth_method {
                                        SshAuthMethod::Password => {
                                            field_label(ui, "SSH Password", &tc);
                                            password_input(ui, &mut app.conn_page.form.ssh_password, "", None);
                                        }
                                        SshAuthMethod::KeyFile => {
                                            field_label(ui, "Key file path", &tc);
                                            file_input(
                                                ui,
                                                &mut app.conn_page.form.ssh_key_path,
                                                "/home/user/.ssh/id_rsa",
                                            );
                                        }
                                        SshAuthMethod::Agent => {
                                            ui.label(
                                                egui::RichText::new("Uses the system SSH agent (ssh-agent / Pageant). No credentials needed.")
                                                    .size(12.0)
                                                    .weak(),
                                            );
                                        }
                                    }
                                    ui.add_space(4.0);
                                });
                        }
                    });

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    // Delete / Duplicate — left side, saved connections only (hidden while testing)
                    if !app.conn_page.is_new && app.conn_page.test_status != TestStatus::Testing {
                        let del_btn = egui::Button::image(
                            icon_image(Icon::Trash2, 14.0, tc.error),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, tc.field_border))
                        .min_size(egui::vec2(32.0, 32.0));
                        if ui.add(del_btn).clicked()
                            && let Some(id) = app.conn_page.selected_id.clone() {
                                app.conn_page.pending_delete = Some(id);
                            }

                        ui.add_space(4.0);

                        let dup_btn = egui::Button::image(
                            icon_image(Icon::Copy, 14.0, tc.text_secondary),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, tc.field_border))
                        .min_size(egui::vec2(32.0, 32.0));
                        if ui.add(dup_btn).on_hover_text("Duplicate connection").clicked() {
                            app.duplicate_connection();
                        }
                    }

                    // Test / Save / Connect — right side
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let connect_btn = egui::Button::image_and_text(
                            icon_image(Icon::CornerDownLeft, 14.0, egui::Color32::WHITE),
                            RichText::new("Connect").size(13.0).color(egui::Color32::WHITE),
                        )
                        .fill(tc.button_primary_bg)
                        .stroke(egui::Stroke::NONE)
                        .min_size(egui::vec2(0.0, 32.0));
                        if ui.add(connect_btn).clicked() {
                            app.connect(ctx.clone());
                        }

                        ui.add_space(8.0);

                        let save_btn =
                            egui::Button::new(RichText::new("Save").size(13.0).color(tc.text_primary))
                                .fill(tc.background)
                                .stroke(egui::Stroke::new(1.0, tc.field_border))
                                .min_size(egui::vec2(0.0, 32.0));
                        if ui.add(save_btn).clicked() {
                            app.save_connection();
                        }

                        ui.add_space(8.0);

                        let (test_label, test_color) = match &app.conn_page.test_status {
                            TestStatus::Idle | TestStatus::Ok(_) | TestStatus::Failed(_) => ("Test Connection", tc.text_secondary),
                            TestStatus::Testing => ("Testing…", tc.text_disabled),
                        };
                        let test_btn = egui::Button::image_and_text(
                            icon_image(Icon::Plug2, 14.0, test_color),
                            RichText::new(test_label).size(13.0).color(test_color),
                        )
                        .fill(tc.background)
                        .stroke(egui::Stroke::new(1.0, tc.field_border))
                        .min_size(egui::vec2(0.0, 32.0));
                        if ui.add(test_btn).clicked()
                            && app.conn_page.test_status != TestStatus::Testing
                        {
                            app.test_connection(ctx.clone());
                        }

                        let status_msg: Option<(String, egui::Color32)> =
                            if app.conn_page.save_status == SaveStatus::Saved {
                                Some(("Saved".into(), tc.success))
                            } else {
                                match &app.conn_page.test_status {
                                    TestStatus::Ok(ms) => Some((
                                        format!("Connected ({}ms)", ms),
                                        tc.success,
                                    )),
                                    TestStatus::Failed(_) => {
                                        Some(("Connection failed".into(), tc.error))
                                    }
                                    _ => None,
                                }
                            };
                        if let Some((msg, color)) = status_msg {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(msg).size(12.0).color(color));
                                });
                            });
                        }
                    });
                });


            });
    });
}

fn field_label(ui: &mut egui::Ui, text: &str, tc: &ThemeColors) {
    ui.label(
        RichText::new(text)
            .size(11.5)
            .color(tc.text_secondary)
            .family(egui::FontFamily::Name("SemiBold".into())),
    );
    ui.add_space(4.0);
}

fn section_label(ui: &mut egui::Ui, text: &str, tc: &ThemeColors) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text)
                .size(11.0)
                .color(tc.text_disabled)
                .family(egui::FontFamily::Name("SemiBold".into())),
        );
        ui.add_space(8.0);
        let r = ui.available_rect_before_wrap();
        let y = r.center().y;
        ui.painter()
            .hline(r.x_range(), y, egui::Stroke::new(1.0, tc.border));
        ui.allocate_space(r.size());
    });
}
