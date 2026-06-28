use crate::core::results::model::ColumnDef;
use crate::ui::molecules::data_cell::render_cell;
use egui::RichText;

/// Shared results-grid header row: a `#` index column followed by one column
/// per `ColumnDef` (PK marker, semibold name, weak type). Used by both the
/// read-only `data_table` and the editable grid so the two never drift.
pub(crate) fn results_header(header: &mut egui_extras::TableRow<'_, '_>, columns: &[ColumnDef]) {
    header.col(|ui| {
        ui.label(RichText::new("#").size(11.0).weak());
    });
    for col in columns {
        header.col(|ui| {
            ui.horizontal(|ui| {
                if col.is_pk {
                    ui.label(RichText::new("🔑").size(10.0));
                }
                ui.label(
                    RichText::new(&col.name)
                        .size(12.0)
                        .family(egui::FontFamily::Name("SemiBold".into())),
                );
            });
        });
    }
}

pub fn data_table(
    ui: &mut egui::Ui,
    columns: &[ColumnDef],
    rows: &[Vec<Option<String>>],
    row_height: f32,
    selected: Option<usize>,
) -> Option<usize> {
    let mut new_selected = selected;
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui_extras::TableBuilder::new(ui)
            .striped(false)
            .resizable(true)
            // Cells default to `Sense::hover()`; click sense is required for the
            // per-cell `Response` to report `clicked()` (row selection / Ctrl+C).
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto().at_least(36.0))
            .columns(
                egui_extras::Column::auto().at_least(80.0).resizable(true),
                columns.len(),
            )
            .header(row_height, |mut header| {
                results_header(&mut header, columns);
            })
            .body(|body| {
                body.rows(row_height, rows.len(), |mut row| {
                    let ri = row.index();
                    let is_selected = new_selected == Some(ri);
                    row.set_selected(is_selected);
                    row.col(|ui| {
                        ui.label(RichText::new((ri + 1).to_string()).size(11.0).weak());
                    });
                    for (ci, col) in columns.iter().enumerate() {
                        let (_, resp) = row.col(|ui| {
                            render_cell(ui, col, &rows[ri].get(ci).cloned().flatten());
                        });
                        if resp.clicked() {
                            new_selected = if is_selected { None } else { Some(ri) };
                        }
                    }
                });
            });
    });
    new_selected
}
