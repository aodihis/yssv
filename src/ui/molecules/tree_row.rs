use crate::theme::ThemeColors;
use crate::ui::atoms::icon::{Icon, icon_image};
use egui::{Color32, FontId, Response, Ui, Vec2};

pub struct TreeRowConfig {
    pub label: String,
    pub level: u32,
    pub is_leaf: bool,
    pub is_open: bool,
    pub is_active: bool,
    pub icon: Icon,
    pub icon_color: Option<Color32>,
    pub count: Option<String>,
    pub pill: Option<String>,
}

pub fn tree_row(ui: &mut Ui, cfg: TreeRowConfig) -> Response {
    let tc = ThemeColors::from_ui(ui);
    let row_h = 26.0;
    let indent = 6.0 + cfg.level as f32 * 15.0;
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return resp;
    }

    let center_y = rect.center().y;
    let mut x = rect.left() + indent;

    // Background + active bar
    let bg = if cfg.is_active {
        tc.selection_bg
    } else if resp.hovered() {
        tc.surface_secondary
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(5u8), bg);
    if cfg.is_active {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + 3.0),
            egui::vec2(2.5, rect.height() - 6.0),
        );
        ui.painter().rect_filled(bar, egui::CornerRadius::same(3u8), tc.button_primary_bg);
    }

    // Chevron SVG — paint_at does not advance the cursor
    if !cfg.is_leaf {
        let chev_icon = if cfg.is_open { Icon::ChevronDown } else { Icon::ChevronRight };
        let chev_rect = egui::Rect::from_center_size(
            egui::pos2(x + 7.0, center_y),
            Vec2::splat(10.0),
        );
        icon_image(chev_icon, 10.0, tc.text_disabled).paint_at(ui, chev_rect);
    }
    x += 14.0;

    // Row icon SVG — paint_at does not advance the cursor
    let icon_color = if cfg.is_active {
        tc.button_primary_bg
    } else {
        cfg.icon_color.unwrap_or(tc.text_secondary)
    };
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(x + 7.0, center_y),
        Vec2::splat(13.0),
    );
    icon_image(cfg.icon, 13.0, icon_color).paint_at(ui, icon_rect);
    x += 16.0;

    // Right decorations + label via painter
    let painter = ui.painter();
    let right_x = rect.right() - 8.0;

    if let Some(pill) = &cfg.pill {
        let pill_color = crate::theme::colors::PURPLE;
        let pill_font = FontId::proportional(9.0);
        let pill_galley = painter.layout_no_wrap(pill.to_string(), pill_font.clone(), pill_color);
        let pill_w = pill_galley.size().x + 8.0;
        let pill_h = 16.0;
        let pill_rect = egui::Rect::from_min_size(
            egui::pos2(right_x - pill_w, center_y - pill_h / 2.0),
            egui::vec2(pill_w, pill_h),
        );
        let pill_bg =
            Color32::from_rgba_unmultiplied(pill_color.r(), pill_color.g(), pill_color.b(), 28);
        painter.rect_filled(pill_rect, egui::CornerRadius::same(4u8), pill_bg);
        painter.rect_stroke(
            pill_rect,
            egui::CornerRadius::same(4u8),
            egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(pill_color.r(), pill_color.g(), pill_color.b(), 70),
            ),
            egui::StrokeKind::Outside,
        );
        painter.text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            pill.as_str(),
            pill_font,
            pill_color,
        );
    } else if let Some(count) = &cfg.count {
        painter.text(
            egui::pos2(right_x, center_y),
            egui::Align2::RIGHT_CENTER,
            count,
            FontId::monospace(10.5),
            tc.text_disabled,
        );
    }

    let label_color = tc.text_primary;
    let label_font = FontId::proportional(12.5);
    let label_max_x = right_x - 28.0;
    let label_galley = painter.layout(
        cfg.label.clone(),
        label_font,
        label_color,
        label_max_x - x,
    );
    painter.galley(
        egui::pos2(x, center_y - label_galley.size().y / 2.0),
        label_galley,
        label_color,
    );

    resp
}
