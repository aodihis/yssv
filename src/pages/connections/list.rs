use crate::theme::ThemeColors;
use egui::RichText;

pub fn render_list(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    use crate::ui::atoms::button::primary_button;
    use crate::ui::molecules::conn_item::conn_item;

    let tc = ThemeColors::from_ui(ui);

    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 14,
            right: 14,
            top: 16,
            bottom: 10,
        })
        .show(ui, |ui| {
            ui.label(
                RichText::new("Connections")
                    .size(14.0)
                    .strong()
                    .color(tc.foreground),
            );
            ui.allocate_ui(egui::vec2(ui.available_width(), 36.0), |ui| {
                let te_out =
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if primary_button(ui, "+ New").clicked() {
                            app.conn_page.start_new();
                        }
                        egui::TextEdit::singleline(&mut app.conn_page.search_query)
                            .hint_text("Search…")
                            .desired_width(f32::INFINITY)
                            .margin(egui::Margin {
                                left: 30,
                                right: 11,
                                top: 11,
                                bottom: 11,
                            })
                            .show(ui)
                    });
                let rect = te_out.inner.response.rect;
                ui.painter().text(
                    egui::pos2(rect.left() + 11.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🔍",
                    egui::FontId::proportional(12.0),
                    tc.subtle_foreground,
                );
            });
        });

    ui.add_space(1.0);

    let groups = app.conn_page.grouped_connections();
    let groups_owned: Vec<(String, Vec<String>)> = groups
        .iter()
        .map(|(g, items)| (g.clone(), items.iter().map(|c| c.id.clone()).collect()))
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (group_name, conn_ids) in groups_owned {
            let collapsed = app.conn_page.collapsed_groups.contains(&group_name);

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                let chev = if collapsed { "▸" } else { "▾" };
                let header_resp = ui.add(
                    egui::Button::new(
                        RichText::new(format!("{} {}", chev, group_name.to_uppercase()))
                            .size(11.0)
                            .color(tc.subtle_foreground)
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
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        left: 6,
                        right: 6,
                        top: 0,
                        bottom: 0,
                    })
                    .show(ui, |ui| {
                        for id in &conn_ids {
                            if let Some(c) = app.conn_page.connections.iter().find(|x| &x.id == id)
                            {
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

    let _ = ctx;
}
