use crate::theme::{self, ThemeColors};
use crate::ui::atoms::button::compact_button;
use crate::ui::atoms::icon::{Icon, icon_image, svg_icon};
use crate::ui::atoms::input::text_input;
use egui::RichText;

struct DropTarget {
    dragged_id: String,
    new_group: String,
    /// None = append to the end of `new_group`
    before_id: Option<String>,
}

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

    let is_dragging = egui::DragAndDrop::has_payload_of_type::<String>(ui.ctx());
    let mut drop_target: Option<DropTarget> = None;

    let mut scroll_ui = ui.new_child(egui::UiBuilder::new().max_rect(scroll_rect));
    egui::ScrollArea::vertical().show(&mut scroll_ui, |ui| {
        for (group_name, conn_ids) in &groups_owned {
            let collapsed = app.conn_page.collapsed_groups.contains(group_name);
            let rename_active = app
                .conn_page
                .renaming_group
                .as_ref()
                .map(|(orig, _)| orig == group_name)
                .unwrap_or(false);

            ui.add_space(6.0);

            // --- Group header ---
            if rename_active {
                // Inline rename input
                let (done, cancelled) = ui
                    .horizontal(|ui| {
                        ui.add_space(14.0);
                        let r = app.conn_page.renaming_group.as_mut().unwrap();
                        let resp = text_input(ui, &mut r.1, "", None);
                        // Auto-focus on first frame
                        if !resp.has_focus() && !resp.lost_focus() {
                            resp.request_focus();
                        }
                        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                        let apply = (enter || resp.lost_focus()) && !escape;
                        (apply, escape)
                    })
                    .inner;

                if done || cancelled {
                    if done {
                        let (orig, new_name) = app.conn_page.renaming_group.take().unwrap();
                        let new_name = new_name.trim().to_string();
                        if !new_name.is_empty() && new_name != orig {
                            app.rename_group(&orig, &new_name);
                        }
                        // if name unchanged or empty, renaming_group is already None
                    } else {
                        app.conn_page.renaming_group = None;
                    }
                }
            } else {
                // Normal header: chevron + name + pencil button (inline)
                let mut rename_clicked = false;
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
                        ui.add_space(4.0);
                        let pencil = ui.add(
                            icon_image(Icon::PencilLine, 10.0, tc.text_disabled)
                                .sense(egui::Sense::click()),
                        );
                        if pencil.clicked() {
                            rename_clicked = true;
                        }
                    })
                    .response
                    .interact(egui::Sense::click());

                if rename_clicked {
                    app.conn_page.renaming_group = Some((group_name.clone(), group_name.clone()));
                } else if header_resp.clicked()
                    && !app.conn_page.collapsed_groups.remove(group_name)
                {
                    app.conn_page.collapsed_groups.insert(group_name.clone());
                }

                // Group header acts as a drop zone (cross-group move to front of group)
                if is_dragging && let Some(payload) = header_resp.dnd_release_payload::<String>() {
                    drop_target = Some(DropTarget {
                        dragged_id: (*payload).clone(),
                        new_group: group_name.clone(),
                        before_id: conn_ids.first().cloned(),
                    });
                }
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
                        let drag_payload = egui::DragAndDrop::payload::<String>(ui.ctx());
                        let cursor_pos = ui.ctx().input(|i| i.pointer.hover_pos());

                        for (idx, id) in conn_ids.iter().enumerate() {
                            if let Some(c) = app.conn_page.connections.iter().find(|x| &x.id == id)
                            {
                                let selected = app.conn_page.selected_id.as_deref() == Some(id);
                                let c_clone = c.clone();
                                let resp = conn_item(ui, &c_clone, selected);
                                let item_rect = resp.rect;

                                // Make item draggable
                                resp.dnd_set_drag_payload(id.clone());

                                if resp.clicked() {
                                    let id_clone = id.clone();
                                    app.conn_page.select(&id_clone);
                                }

                                // Drop indicator line and drop detection
                                if let Some(ref payload) = drag_payload
                                    && payload.as_str() != id.as_str()
                                    && let Some(pos) = cursor_pos
                                    && item_rect.contains(pos)
                                {
                                    let line_y = if pos.y < item_rect.center().y {
                                        item_rect.top()
                                    } else {
                                        item_rect.bottom()
                                    };
                                    ui.painter().hline(
                                        egui::Rangef::new(item_rect.left(), item_rect.right()),
                                        line_y,
                                        egui::Stroke::new(2.0, tc.button_primary_bg),
                                    );
                                }

                                if let Some(payload) = resp.dnd_release_payload::<String>() {
                                    // Determine before/after based on release position
                                    let release_y =
                                        ui.ctx().input(|i| i.pointer.interact_pos()).map(|p| p.y);
                                    let before_id = if release_y
                                        .map(|y| y < item_rect.center().y)
                                        .unwrap_or(true)
                                    {
                                        Some(id.clone())
                                    } else {
                                        conn_ids.get(idx + 1).cloned()
                                    };
                                    drop_target = Some(DropTarget {
                                        dragged_id: (*payload).clone(),
                                        new_group: group_name.clone(),
                                        before_id,
                                    });
                                }
                            }
                        }

                        // End-of-group drop zone (invisible, 8px tall)
                        if is_dragging {
                            let zone_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(ui.available_width(), 8.0),
                            );
                            let zone_resp = ui.allocate_rect(zone_rect, egui::Sense::hover());

                            // Show indicator at top of zone when hovering
                            if let Some(pos) = cursor_pos
                                && zone_rect.expand(4.0).contains(pos)
                            {
                                ui.painter().hline(
                                    egui::Rangef::new(zone_rect.left(), zone_rect.right()),
                                    zone_rect.top(),
                                    egui::Stroke::new(2.0, tc.button_primary_bg),
                                );
                            }

                            if let Some(payload) = zone_resp.dnd_release_payload::<String>() {
                                drop_target = Some(DropTarget {
                                    dragged_id: (*payload).clone(),
                                    new_group: group_name.clone(),
                                    before_id: None,
                                });
                            }
                        }
                    });
            }
        }
        ui.add_space(14.0);
    });

    // Apply drag-and-drop reorder (done outside the scroll closure to avoid borrow conflicts)
    if let Some(dt) = drop_target {
        let is_noop = dt.before_id.as_deref() == Some(dt.dragged_id.as_str());
        if !is_noop {
            app.reorder_connections(&dt.dragged_id, &dt.new_group, dt.before_id.as_deref());
        }
    }

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
