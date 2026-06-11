pub mod detail;
pub mod list;
pub mod state;

pub use state::ConnectionsPageState;

pub fn render(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let tc = crate::theme::ThemeColors::for_theme(app.settings.theme);

    egui::Panel::left("conn_list_panel")
        .exact_size(307.0)
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame::new().fill(tc.surface))
        .show_inside(ui, |ui| {
            list::render_list(ui, app);
            let r = ui.max_rect();
            ui.painter()
                .vline(r.right(), r.y_range(), egui::Stroke::new(1.0, tc.border));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(tc.background))
        .show_inside(ui, |ui| {
            detail::render_detail(ui, app);
        });

    render_delete_confirm(ui, app);
}

fn render_delete_confirm(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let Some(id) = app.conn_page.pending_delete.clone() else {
        return;
    };
    let conn_name = app
        .conn_page
        .connections
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "this connection".into());

    let tc = crate::theme::ThemeColors::for_theme(app.settings.theme);

    // Scrim — rendered in Foreground layer so it covers the whole window.
    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("delete_confirm_scrim"),
        ))
        .rect_filled(
            ui.ctx().content_rect(),
            egui::CornerRadius::ZERO,
            egui::Color32::from_black_alpha(140),
        );

    let mut confirmed = false;
    let mut cancelled = false;

    // Dialog card — rendered in Tooltip layer (above Foreground).
    egui::Area::new(egui::Id::new("delete_confirm_dialog"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(tc.surface)
                .stroke(egui::Stroke::new(1.0, tc.border))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_min_width(340.0);
                    ui.set_max_width(340.0);

                    // Trash icon in a tinted circle
                    ui.vertical_centered(|ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(44.0, 44.0),
                            egui::Sense::hover(),
                        );
                        let icon_bg = egui::Color32::from_rgba_unmultiplied(
                            tc.error.r(),
                            tc.error.g(),
                            tc.error.b(),
                            30,
                        );
                        ui.painter()
                            .circle_filled(rect.center(), 22.0, icon_bg);
                        ui.painter().circle_stroke(
                            rect.center(),
                            22.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(
                                tc.error.r(),
                                tc.error.g(),
                                tc.error.b(),
                                60,
                            )),
                        );
                        // Inline trash icon via image
                        let icon_rect = egui::Rect::from_center_size(
                            rect.center(),
                            egui::vec2(20.0, 20.0),
                        );
                        let (bytes, uri) = (
                            egui::load::Bytes::Static(include_bytes!(
                                "../../../assets/icons/trash-2.svg"
                            )),
                            "bytes://icon/trash-2.svg",
                        );
                        ui.put(
                            icon_rect,
                            egui::Image::new(egui::ImageSource::Bytes {
                                uri: uri.into(),
                                bytes,
                            })
                            .fit_to_exact_size(egui::vec2(20.0, 20.0))
                            .tint(tc.error),
                        );
                    });

                    ui.add_space(16.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Delete connection?")
                                .size(15.0)
                                .color(tc.text_primary)
                                .family(egui::FontFamily::Name("SemiBold".into())),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "\"{}\" will be permanently removed.",
                                conn_name
                            ))
                            .size(13.0)
                            .color(tc.text_secondary),
                        );
                    });

                    ui.add_space(20.0);

                    // Divider
                    let div_rect = egui::Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(ui.available_width(), 1.0),
                    );
                    ui.painter()
                        .rect_filled(div_rect, egui::CornerRadius::ZERO, tc.border);
                    ui.add_space(1.0);
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let del = egui::Button::new(
                                    egui::RichText::new("Delete")
                                        .size(13.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(tc.error)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(6.0)
                                .min_size(egui::vec2(80.0, 32.0));
                                if ui.add(del).clicked() {
                                    confirmed = true;
                                }

                                ui.add_space(8.0);

                                let cancel = egui::Button::new(
                                    egui::RichText::new("Cancel")
                                        .size(13.0)
                                        .color(tc.text_primary),
                                )
                                .fill(tc.background)
                                .stroke(egui::Stroke::new(1.0, tc.field_border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(80.0, 32.0));
                                if ui.add(cancel).clicked() {
                                    cancelled = true;
                                }
                            },
                        );
                    });
                });
        });

    if confirmed {
        app.conn_page.pending_delete = None;
        app.delete_connection(&id);
    } else if cancelled {
        app.conn_page.pending_delete = None;
    }
}
