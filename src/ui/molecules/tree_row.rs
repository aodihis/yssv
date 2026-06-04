use egui::{Color32, FontId, Response, Ui, Vec2};
use crate::theme::ThemeColors;

pub struct TreeRowConfig<'a> {
    pub label:     &'a str,
    pub level:     u32,
    pub is_leaf:   bool,
    pub is_open:   bool,
    pub is_active: bool,
    pub icon:      &'a str,
    pub icon_color: Option<Color32>,
    pub count:     Option<String>,
    pub pill:      Option<&'a str>,
}

pub fn tree_row(ui: &mut Ui, cfg: TreeRowConfig<'_>) -> Response {
    let tc = ThemeColors::from_ui(ui);
    let row_h = 26.0;
    // Indent: 6px base + 15px per level (design spec)
    let indent = 6.0 + cfg.level as f32 * 15.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), row_h),
        egui::Sense::click(),
    );
    if !ui.is_rect_visible(rect) { return resp; }

    let painter = ui.painter();

    // Background
    let bg = if cfg.is_active {
        tc.bg_selected
    } else if resp.hovered() {
        tc.bg_hover
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, egui::CornerRadius::same(5u8), bg);

    // Active left bar — 2.5px, accent color
    if cfg.is_active {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + 3.0),
            egui::vec2(2.5, rect.height() - 6.0),
        );
        painter.rect_filled(bar, egui::CornerRadius::same(3u8), tc.accent);
    }

    let center_y = rect.center().y;
    let mut x = rect.left() + indent;

    // Chevron — 14px wide
    if !cfg.is_leaf {
        let chev = if cfg.is_open { "▾" } else { "▸" };
        painter.text(
            egui::pos2(x + 7.0, center_y),
            egui::Align2::CENTER_CENTER,
            chev,
            FontId::proportional(11.0),
            tc.text_faint,
        );
    }
    x += 14.0;

    // Icon — 14px wide
    let icon_color = if cfg.is_active {
        tc.accent
    } else {
        cfg.icon_color.unwrap_or(tc.text_muted)
    };
    painter.text(
        egui::pos2(x + 7.0, center_y),
        egui::Align2::CENTER_CENTER,
        cfg.icon,
        FontId::proportional(13.0),
        icon_color,
    );
    x += 16.0;

    // Label — 12.5px
    let label_color = tc.text;
    let label_font = FontId::proportional(12.5);

    // Right side: count or pill — compute before drawing label
    let right_x = rect.right() - 8.0;

    if let Some(pill) = cfg.pill {
        let pill_color = crate::theme::colors::PURPLE;
        let pill_font = FontId::proportional(9.0);
        let pill_galley = painter.layout_no_wrap(pill.to_string(), pill_font.clone(), pill_color);
        let pill_w = pill_galley.size().x + 8.0;
        let pill_h = 16.0;
        let pill_rect = egui::Rect::from_min_size(
            egui::pos2(right_x - pill_w, center_y - pill_h / 2.0),
            egui::vec2(pill_w, pill_h),
        );
        let pill_bg = Color32::from_rgba_unmultiplied(
            pill_color.r(), pill_color.g(), pill_color.b(), 28,
        );
        painter.rect_filled(pill_rect, egui::CornerRadius::same(4u8), pill_bg);
        painter.rect_stroke(pill_rect, egui::CornerRadius::same(4u8), egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(pill_color.r(), pill_color.g(), pill_color.b(), 70)), egui::StrokeKind::Outside);
        painter.text(pill_rect.center(), egui::Align2::CENTER_CENTER, pill, pill_font, pill_color);
    } else if let Some(count) = &cfg.count {
        painter.text(
            egui::pos2(right_x, center_y),
            egui::Align2::RIGHT_CENTER,
            count,
            FontId::monospace(10.5),
            tc.text_faint,
        );
    }

    // Label (clip before right decorations)
    let label_max_x = right_x - 28.0;
    let label_galley = painter.layout(
        cfg.label.to_string(),
        label_font,
        label_color,
        label_max_x - x,
    );
    painter.galley(egui::pos2(x, center_y - label_galley.size().y / 2.0), label_galley, label_color);

    resp
}
