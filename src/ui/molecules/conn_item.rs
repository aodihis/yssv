use crate::core::connections::model::Connection;
use crate::theme::ThemeColors;
use crate::ui::atoms::icon::{icon_image, Icon};
use egui::{Color32, FontId, Response, Ui, Vec2};

pub fn conn_item(ui: &mut Ui, conn: &Connection, selected: bool) -> Response {
    let tc = ThemeColors::from_ui(ui);
    let item_h = 48.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), item_h),
        egui::Sense::click(),
    );
    if !ui.is_rect_visible(rect) {
        return resp;
    }

    let painter = ui.painter();

    // Background
    let bg = if selected {
        tc.primary_subtle
    } else if resp.hovered() {
        tc.accent
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, egui::CornerRadius::same(5u8), bg);

    // Active left bar
    if selected {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + 6.0),
            egui::vec2(3.0, rect.height() - 12.0),
        );
        painter.rect_filled(bar, egui::CornerRadius::same(3u8), tc.primary);
    }

    // Layout: left-pad 12 (accounts for the bar), dot, gap, meta block, engine badge right
    let left_x = rect.left() + 14.0;
    let center_y = rect.center().y;

    // Color dot — 9px + glow ring
    let dot_c = conn.color.to_color32();
    let dot_pos = egui::pos2(left_x + 4.5, center_y);
    // Glow ring (larger, semi-transparent circle)
    painter.circle_filled(
        dot_pos,
        7.0,
        Color32::from_rgba_unmultiplied(dot_c.r(), dot_c.g(), dot_c.b(), 30),
    );
    painter.circle_filled(dot_pos, 4.5, dot_c);

    // Name + host text block
    let text_x = left_x + 18.0;
    let name_y = center_y - 9.0;
    let host_y = center_y + 5.5;

    painter.text(
        egui::pos2(text_x, name_y),
        egui::Align2::LEFT_CENTER,
        &conn.name,
        FontId::proportional(13.0),
        tc.foreground,
    );
    painter.text(
        egui::pos2(text_x, host_y),
        egui::Align2::LEFT_CENTER,
        conn.display_host(),
        FontId::monospace(10.5),
        tc.subtle_foreground,
    );

    let has_ssh = conn.ssh.is_some();

    // Engine badge — right side, 9.5px mono uppercase in a small rounded pill
    let badge_text = conn.engine.label().to_uppercase();
    let badge_font = FontId::monospace(9.5);
    let badge_galley =
        painter.layout_no_wrap(badge_text.clone(), badge_font.clone(), tc.muted_foreground);
    let badge_w = badge_galley.size().x + 10.0; // 5px padding each side
    let badge_h = 18.0;
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.right() - badge_w - 10.0, center_y - badge_h / 2.0),
        egui::vec2(badge_w, badge_h),
    );
    painter.rect_filled(badge_rect, egui::CornerRadius::same(4u8), tc.accent_active);
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        &badge_text,
        badge_font,
        tc.muted_foreground,
    );

    if has_ssh {
        let icon_size = 12.0;
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(badge_rect.left() - 18.0, center_y - icon_size / 2.0),
            egui::vec2(icon_size, icon_size),
        );
        ui.put(icon_rect, icon_image(Icon::Terminal, icon_size, tc.subtle_foreground));
    }

    resp
}
