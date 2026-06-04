use egui::{Color32, RichText, Ui, Vec2};
use crate::pages::explorer::state::TableTab;

/// Returns the index of the tab that was requested to close, if any.
pub fn tab_bar(ui: &mut Ui, tabs: &[TableTab], active: usize) -> (Option<usize>, Option<usize>) {
    let mut close_req: Option<usize> = None;
    let mut activate_req: Option<usize> = None;

    ui.horizontal(|ui| {
        for (i, tab) in tabs.iter().enumerate() {
            let is_active = i == active;
            let label = format!("{}.{}", tab.schema, tab.table);

            let (bg, fg) = if is_active {
                (ui.visuals().selection.bg_fill, ui.visuals().text_color())
            } else {
                (Color32::TRANSPARENT, ui.visuals().weak_text_color())
            };

            let resp = ui
                .horizontal(|ui| {
                    ui.add_space(4.0);
                    let r = ui.add(
                        egui::Button::new(RichText::new(&label).size(12.0).color(fg))
                            .fill(bg)
                            .min_size(Vec2::new(0.0, 26.0)),
                    );
                    let close = ui.add(
                        egui::Button::new(RichText::new("×").size(11.0).color(fg))
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(16.0, 16.0)),
                    );
                    if close.clicked() {
                        close_req = Some(i);
                    }
                    r
                })
                .inner;

            if resp.clicked() {
                activate_req = Some(i);
            }
        }
    });

    (activate_req, close_req)
}
