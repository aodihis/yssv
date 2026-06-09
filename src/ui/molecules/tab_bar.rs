use crate::pages::explorer::state::TableTab;
use crate::theme::ThemeColors;
use egui::{Color32, FontId, Ui};

const TAB_H: f32 = 36.0;

/// Returns (activate_index, close_index)
pub fn tab_bar(ui: &mut Ui, tabs: &[TableTab], active: usize) -> (Option<usize>, Option<usize>) {
    let tc = ThemeColors::from_ui(ui);
    let mut close_req: Option<usize> = None;
    let mut activate_req: Option<usize> = None;

    // Reserve full width strip, TAB_H tall
    let strip_rect =
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), TAB_H));
    // Fill strip bg
    ui.painter()
        .rect_filled(strip_rect, egui::CornerRadius::ZERO, tc.surface);

    ui.horizontal(|ui| {
        ui.set_height(TAB_H);
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        // "×" is constant — measure once outside the loop
        let close_font = FontId::proportional(12.0);
        let close_w = ui
            .painter()
            .layout_no_wrap("×".into(), close_font.clone(), tc.foreground)
            .size()
            .x;

        for (i, tab) in tabs.iter().enumerate() {
            let is_active = i == active;
            let label = format!("{}.{}", tab.schema, tab.table);

            // Measure label to allocate exact tab width
            let label_font = FontId::proportional(12.5);
            let text_w = ui
                .painter()
                .layout_no_wrap(label.clone(), label_font.clone(), tc.foreground)
                .size()
                .x;
            let tab_w = 12.0 + text_w + 8.0 + close_w + 12.0;

            let (tab_rect, tab_resp) =
                ui.allocate_exact_size(egui::vec2(tab_w, TAB_H), egui::Sense::click());

            if !ui.is_rect_visible(tab_rect) {
                continue;
            }

            let painter = ui.painter();

            // Tab background
            let bg = if is_active {
                tc.background
            } else if tab_resp.hovered() {
                tc.accent
            } else {
                Color32::TRANSPARENT
            };
            painter.rect_filled(tab_rect, egui::CornerRadius::ZERO, bg);

            // Active primary underline (2px at bottom)
            if is_active {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(tab_rect.left(), tab_rect.bottom() - 2.0),
                        egui::vec2(tab_rect.width(), 2.0),
                    ),
                    egui::CornerRadius::ZERO,
                    tc.primary,
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

            let center_y = tab_rect.center().y;
            let text_color = if is_active {
                tc.foreground
            } else {
                tc.muted_foreground
            };
            let label_x = tab_rect.left() + 12.0;

            // Tab label
            painter.text(
                egui::pos2(label_x, center_y),
                egui::Align2::LEFT_CENTER,
                &label,
                label_font,
                text_color,
            );

            // Close button hit area
            let close_rect = egui::Rect::from_center_size(
                egui::pos2(label_x + text_w + 8.0 + close_w / 2.0, center_y),
                egui::vec2(20.0, 20.0),
            );
            let close_resp = ui.interact(
                close_rect,
                ui.id().with(("tab_close", i)),
                egui::Sense::click(),
            );
            let close_color = if close_resp.hovered() {
                tc.foreground
            } else {
                tc.subtle_foreground
            };
            if close_resp.hovered() {
                painter.rect_filled(close_rect, egui::CornerRadius::same(4u8), tc.accent_active);
            }
            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "×",
                close_font.clone(),
                close_color,
            );

            if close_resp.clicked() {
                close_req = Some(i);
            } else if tab_resp.clicked() {
                activate_req = Some(i);
            }
        }
    });

    // Bottom border across full width
    ui.painter().hline(
        strip_rect.x_range(),
        strip_rect.bottom(),
        egui::Stroke::new(1.0, tc.border),
    );

    (activate_req, close_req)
}
