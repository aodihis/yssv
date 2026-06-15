use crate::pages::explorer::state::Tab;
use crate::theme::ThemeColors;
use crate::ui::atoms::icon::{Icon, icon_image};
use egui::{Color32, FontId, Ui};

const TAB_H: f32 = 38.0;
const CLOSE_W: f32 = 10.0;
const ARROW_W: f32 = 28.0;

#[derive(Debug, Clone)]
pub enum TabContextAction {
    Close(usize),
    CloseAll,
    CloseOthers(usize),
    CloseToLeft(usize),
    CloseToRight(usize),
}

/// Returns (activate_index, tab_action)
pub fn tab_bar(
    ui: &mut Ui,
    tabs: &[Tab],
    active: usize,
) -> (Option<usize>, Option<TabContextAction>) {
    if tabs.is_empty() {
        return (None, None);
    }

    let tc = ThemeColors::from_ui(ui);
    let mut tab_action: Option<TabContextAction> = None;
    let mut activate_req: Option<usize> = None;

    let avail_w = ui.available_width();
    let strip_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(avail_w, TAB_H));

    // Pre-compute per-tab label widths; full tab width = padding + label + close icon
    let label_font = FontId::proportional(12.5);
    let active_font = FontId::new(12.5, egui::FontFamily::Name("SemiBold".into()));
    let text_widths: Vec<f32> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let font = if i == active { active_font.clone() } else { label_font.clone() };
            ui.painter()
                .layout_no_wrap(tab.label(), font, Color32::WHITE)
                .size()
                .x
        })
        .collect();
    let tab_widths: Vec<f32> = text_widths.iter().map(|&w| tab_width(w)).collect();

    let total_w: f32 = tab_widths.iter().sum();
    let needs_arrows = total_w > avail_w;
    let tabs_avail_w = if needs_arrows {
        (avail_w - 2.0 * ARROW_W).max(0.0)
    } else {
        avail_w
    };

    // Load persisted scroll offset + the active tab from the previous frame.
    let scroll_id = ui.id().with("tab_bar_scroll");
    let active_id = ui.id().with("tab_bar_active");
    let mut scroll: usize = ui.memory(|m| m.data.get_temp::<usize>(scroll_id).unwrap_or(0));
    let prev_active: Option<usize> = ui.memory(|m| m.data.get_temp::<usize>(active_id));
    scroll = scroll.min(tabs.len() - 1);

    // Reveal the active tab only when it actually changes (opened / switched).
    // Doing this every frame would pin the scroll to the active tab and silently
    // undo manual arrow scrolling. When everything fits, reset to the start.
    if !needs_arrows {
        scroll = 0;
    } else if prev_active != Some(active) {
        if active < scroll {
            scroll = active;
        } else {
            loop {
                let last = last_visible_from(scroll, &tab_widths, tabs_avail_w);
                if active <= last || scroll + 1 >= tabs.len() {
                    break;
                }
                scroll += 1;
            }
        }
    }

    let last_vis = last_visible_from(scroll, &tab_widths, tabs_avail_w);
    let can_left = scroll > 0;
    let can_right = last_vis < tabs.len() - 1;

    // Strip background
    ui.painter()
        .rect_filled(strip_rect, egui::CornerRadius::ZERO, tc.surface);

    // Arrow buttons — register interactions before the tab loop so hover state
    // is available, but defer all painting until after the tabs so arrows
    // always render on top of any overflowing tab content.
    let arrow_state = if needs_arrows {
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(strip_rect.right() - 2.0 * ARROW_W, strip_rect.top()),
            egui::vec2(ARROW_W, TAB_H),
        );
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(strip_rect.right() - ARROW_W, strip_rect.top()),
            egui::vec2(ARROW_W, TAB_H),
        );

        let left_resp = ui.interact(left_rect, ui.id().with("tab_left"), egui::Sense::click());
        let right_resp = ui.interact(right_rect, ui.id().with("tab_right"), egui::Sense::click());

        if left_resp.clicked() && can_left {
            scroll -= 1;
        }
        if right_resp.clicked() && can_right {
            scroll += 1;
        }

        Some((left_rect, right_rect, left_resp, right_resp))
    } else {
        None
    };

    // Persist updated scroll + active for next frame's change detection.
    ui.memory_mut(|m| {
        m.data.insert_temp(scroll_id, scroll);
        m.data.insert_temp(active_id, active);
    });

    // Render tabs clipped to their allocated area
    let tabs_clip = egui::Rect::from_min_size(strip_rect.min, egui::vec2(tabs_avail_w, TAB_H));
    let painter = ui.painter().with_clip_rect(tabs_clip);

    let tab_count = tabs.len();
    let mut tab_x = strip_rect.left();
    for i in scroll..tab_count {
        let tw = tab_widths[i];
        // Stop only when the tab starts beyond the clip (fully off-screen).
        // Partially visible tabs still render — the clip rect handles the cutoff.
        if tab_x >= tabs_clip.right() {
            break;
        }

        let tab_rect =
            egui::Rect::from_min_size(egui::pos2(tab_x, strip_rect.top()), egui::vec2(tw, TAB_H));
        let is_active = i == active;
        let center_y = tab_rect.center().y;
        let label_x = tab_rect.left() + 12.0;
        let text_w = text_widths[i];

        let close_center = egui::pos2(label_x + text_w + 8.0 + CLOSE_W / 2.0, center_y);
        let close_rect = egui::Rect::from_center_size(close_center, egui::vec2(20.0, 20.0));

        // Clip the hit-test rect to the visible area so interactions don't
        // register in the portion hidden behind the arrow buttons.
        let visible_tab_rect = tab_rect.intersect(tabs_clip);
        let tab_resp =
            ui.interact(visible_tab_rect, ui.id().with(("tab", i)), egui::Sense::click());
        // Close button: only interactive when fully visible.
        let close_visible = close_rect.max.x <= tabs_clip.right();
        let close_resp = ui.interact(
            if close_visible { close_rect } else { egui::Rect::NOTHING },
            ui.id().with(("tab_close", i)),
            egui::Sense::click(),
        );

        // Background
        let bg = if is_active {
            tc.background
        } else if tab_resp.hovered() {
            tc.surface_secondary
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(tab_rect, egui::CornerRadius::ZERO, bg);

        // Active underline
        if is_active {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(tab_rect.left(), tab_rect.bottom() - 2.0),
                    egui::vec2(tw, 2.0),
                ),
                egui::CornerRadius::ZERO,
                tc.button_primary_bg,
            );
        }

        // Right separator (inactive tabs only)
        if !is_active {
            painter.vline(
                tab_rect.right(),
                tab_rect.top()..=tab_rect.bottom(),
                egui::Stroke::new(1.0, tc.border_muted),
            );
        }

        // Label
        let text_color = if is_active {
            tc.text_primary
        } else {
            tc.text_secondary
        };
        let draw_font = if is_active { active_font.clone() } else { label_font.clone() };
        painter.text(
            egui::pos2(label_x, center_y),
            egui::Align2::LEFT_CENTER,
            tabs[i].label(),
            draw_font,
            text_color,
        );

        // Close icon
        let close_color = if close_resp.hovered() {
            tc.text_primary
        } else {
            tc.text_disabled
        };
        icon_image(Icon::X, CLOSE_W, close_color).paint_at(ui, close_rect.shrink(5.0));

        if close_resp.clicked() {
            tab_action = Some(TabContextAction::Close(i));
        } else if tab_resp.clicked() {
            activate_req = Some(i);
        }

        // Right-click context menu
        let action_ref = &mut tab_action;
        tab_resp.context_menu(|ui| {
            if ui.button("Close").clicked() {
                *action_ref = Some(TabContextAction::Close(i));
                ui.close();
            }
            if ui.button("Close All").clicked() {
                *action_ref = Some(TabContextAction::CloseAll);
                ui.close();
            }
            let others_resp =
                ui.add_enabled(tab_count > 1, egui::Button::new("Close Others"));
            if others_resp.clicked() {
                *action_ref = Some(TabContextAction::CloseOthers(i));
                ui.close();
            }
            ui.separator();
            let left_resp = ui.add_enabled(i > 0, egui::Button::new("Close to the Left"));
            if left_resp.clicked() {
                *action_ref = Some(TabContextAction::CloseToLeft(i));
                ui.close();
            }
            let right_resp =
                ui.add_enabled(i + 1 < tab_count, egui::Button::new("Close to the Right"));
            if right_resp.clicked() {
                *action_ref = Some(TabContextAction::CloseToRight(i));
                ui.close();
            }
        });

        tab_x += tw;
    }

    // Bottom border across full strip
    ui.painter().hline(
        strip_rect.x_range(),
        strip_rect.bottom(),
        egui::Stroke::new(1.0, tc.border),
    );

    // Paint arrow buttons on top of all tab content
    if let Some((left_rect, right_rect, left_resp, right_resp)) = arrow_state {
        let p = ui.painter();

        // Separator before arrow area
        p.vline(
            left_rect.left(),
            strip_rect.top()..=strip_rect.bottom(),
            egui::Stroke::new(1.0, tc.border),
        );

        // Left arrow
        p.rect_filled(
            left_rect,
            egui::CornerRadius::ZERO,
            if can_left && left_resp.hovered() { tc.surface_secondary } else { tc.surface },
        );
        p.text(
            left_rect.center(),
            egui::Align2::CENTER_CENTER,
            "‹",
            FontId::proportional(16.0),
            if can_left { tc.text_secondary } else { tc.text_disabled },
        );

        // Separator between arrows
        p.vline(
            right_rect.left(),
            strip_rect.top()..=strip_rect.bottom(),
            egui::Stroke::new(1.0, tc.border),
        );

        // Right arrow
        p.rect_filled(
            right_rect,
            egui::CornerRadius::ZERO,
            if can_right && right_resp.hovered() { tc.surface_secondary } else { tc.surface },
        );
        p.text(
            right_rect.center(),
            egui::Align2::CENTER_CENTER,
            "›",
            FontId::proportional(16.0),
            if can_right { tc.text_secondary } else { tc.text_disabled },
        );
    }

    // Advance the layout cursor past the strip
    let _ = ui.allocate_rect(strip_rect, egui::Sense::hover());

    (activate_req, tab_action)
}

fn tab_width(text_w: f32) -> f32 {
    12.0 + text_w + 8.0 + CLOSE_W + 12.0
}

fn last_visible_from(scroll: usize, widths: &[f32], avail: f32) -> usize {
    let mut w = 0.0;
    let mut last = scroll;
    for (i, width) in widths.iter().enumerate().skip(scroll) {
        if w + width > avail {
            break;
        }
        w += width;
        last = i;
    }
    last
}
