use crate::theme::{self, ThemeColors};
use crate::ui::atoms::button::compact_button;
use crate::ui::atoms::icon::{Icon, svg_icon};
use crate::ui::atoms::input::text_input;
use egui::RichText;

pub fn render_list(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    use crate::ui::molecules::conn_item::conn_item;

    let ctx = ui.ctx().clone();
    let tc = ThemeColors::from_ui(ui);

    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 14,
            right: 14,
            top: 20,
            bottom: 10,
        })
        .show(ui, |ui| {
            ui.label(
                RichText::new("Connections")
                    .size(14.0)
                    .family(egui::FontFamily::Name("SemiBold".into()))
                    .color(tc.text_primary),
            );
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if compact_button(ui, "+ New").clicked() {
                    app.conn_page.start_new();
                }
                text_input(ui, &mut app.conn_page.search_query, "Search…", Some("🔍"));
            });
        });

    ui.add_space(1.0);

    let footer_h = 40.0;
    let available = ui.available_rect_before_wrap();
    let scroll_rect = egui::Rect::from_min_size(
        available.min,
        egui::vec2(available.width(), (available.height() - footer_h).max(0.0)),
    );
    let footer_rect = egui::Rect::from_min_size(
        egui::pos2(available.min.x, available.max.y - footer_h),
        egui::vec2(available.width(), footer_h),
    );

    let groups = app.conn_page.grouped_connections();
    let groups_owned: Vec<(String, Vec<String>)> = groups
        .iter()
        .map(|(g, items)| (g.clone(), items.iter().map(|c| c.id.clone()).collect()))
        .collect();

    let mut scroll_ui = ui.new_child(egui::UiBuilder::new().max_rect(scroll_rect));
    egui::ScrollArea::vertical().show(&mut scroll_ui, |ui| {
        for (group_name, conn_ids) in groups_owned {
            let collapsed = app.conn_page.collapsed_groups.contains(&group_name);

            ui.add_space(6.0);
            let header_resp = ui
                .horizontal(|ui| {
                    ui.add_space(14.0);
                    let chev = if collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    };
                    svg_icon(ui, chev, 10.0, tc.text_disabled);
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new(group_name.to_uppercase())
                            .size(11.0)
                            .color(tc.text_disabled)
                            .family(egui::FontFamily::Name("SemiBold".into())),
                    );
                })
                .response
                .interact(egui::Sense::click());
            if header_resp.clicked() && !app.conn_page.collapsed_groups.remove(&group_name) {
                app.conn_page.collapsed_groups.insert(group_name);
            }

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

    let mut footer_ui = ui.new_child(egui::UiBuilder::new().max_rect(footer_rect));
    footer_ui.painter().hline(
        footer_rect.x_range(),
        footer_rect.top(),
        egui::Stroke::new(1.0, tc.border_muted),
    );
    footer_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        let theme_icon = if app.settings.theme == theme::Theme::Dark {
            "☀"
        } else {
            "🌙"
        };
        let theme_btn = egui::Button::new(
            egui::RichText::new(theme_icon)
                .size(14.0)
                .color(tc.text_secondary),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(28.0, 24.0));
        if ui.add(theme_btn).clicked() {
            app.settings.toggle_theme();
            theme::apply_theme(&ctx, app.settings.theme);
        }
    });
}
