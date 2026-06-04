use egui::{Color32, RichText, Ui};
use crate::pages::explorer::state::TableTab;

/// Returns (prev_clicked, next_clicked)
pub fn status_bar(ui: &mut Ui, tab: &TableTab) -> (bool, bool) {
    let mut prev = false;
    let mut next = false;

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        if let Some(result) = &tab.result {
            let total = result.total_rows.unwrap_or(0);
            let from = tab.offset() + 1;
            let to = (tab.offset() + tab.page_size).min(total as u32);
            let pages = tab.total_pages();

            ui.label(
                RichText::new(format!(
                    "Rows {from}–{to} of {total}    Page {} of {pages}",
                    tab.page + 1
                ))
                .size(11.5)
                .color(ui.visuals().weak_text_color()),
            );

            ui.add_space(8.0);
            if ui
                .add_enabled(
                    tab.can_go_prev(),
                    egui::Button::new(RichText::new("◀").size(11.0))
                        .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                prev = true;
            }
            if ui
                .add_enabled(
                    tab.can_go_next(),
                    egui::Button::new(RichText::new("▶").size(11.0))
                        .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                next = true;
            }
        } else if tab.loading {
            ui.label(RichText::new("Loading…").size(11.5).color(ui.visuals().weak_text_color()));
        }
    });

    (prev, next)
}
