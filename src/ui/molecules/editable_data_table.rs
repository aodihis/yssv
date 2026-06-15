use crate::core::edit::TableEdits;
use crate::core::results::model::ColumnDef;
use crate::pages::explorer::state::{EditingCell, RowRef};
use crate::theme::{ThemeColors, colors};
use crate::ui::molecules::data_cell::render_cell;
use crate::ui::molecules::data_table::results_header;
use egui::{Color32, RichText};

/// How an in-progress cell edit ended. `CommitNext` (Enter) saves the value and
/// moves the editor to the next column so a row can be filled keyboard-only.
enum Finish {
    Cancel,
    CommitStop,
    CommitNext,
}

/// Editable results grid (DataGrip-style). Renders the loaded page plus any
/// uncommitted insert rows, tinting modified cells, inserted rows, and rows
/// marked for deletion. All interaction mutates `edits` / `editing` / `selected`
/// in place — nothing touches the database here; that happens on commit.
#[allow(clippy::too_many_arguments)]
pub fn editable_data_table(
    ui: &mut egui::Ui,
    columns: &[ColumnDef],
    rows: &[Vec<Option<String>>],
    edits: &mut TableEdits,
    editing: &mut Option<EditingCell>,
    selected: &mut Option<usize>,
    row_height: f32,
) {
    let tc = ThemeColors::from_ui(ui);
    // egui labels are selectable by default; that text-drag interaction swallows
    // cell clicks/double-clicks, making row selection and edit-on-double-click
    // unreliable. Turn it off so a click always lands on the cell.
    ui.style_mut().interaction.selectable_labels = false;

    // Insert rows render first (pinned at the top) so a freshly added row is
    // always visible regardless of which page is loaded.
    let insert_count = edits.inserts.len();
    let total = insert_count + rows.len();

    let modified_bg = tint(colors::WARNING, 30);
    let inserted_bg = tint(tc.success, 26);
    let deleted_bg = tint(tc.error, 26);

    egui::ScrollArea::both().show(ui, |ui| {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            // Cells default to `Sense::hover()`; without click sense the
            // per-cell `Response` never reports `clicked()` / `double_clicked()`,
            // so row selection and double-click-to-edit would do nothing.
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
                body.rows(row_height, total, |mut row| {
                    let display = row.index();
                    let row_ref = if display < insert_count {
                        RowRef::Insert(display)
                    } else {
                        RowRef::Original(display - insert_count)
                    };
                    let is_insert = matches!(row_ref, RowRef::Insert(_));
                    let is_deleted =
                        matches!(row_ref, RowRef::Original(r) if edits.deletes.contains(&r));
                    let is_selected = *selected == Some(display);
                    row.set_selected(is_selected);

                    let row_bg = if is_insert {
                        Some(inserted_bg)
                    } else if is_deleted {
                        Some(deleted_bg)
                    } else {
                        None
                    };

                    row.col(|ui| {
                        if let Some(bg) = row_bg {
                            ui.painter()
                                .rect_filled(ui.max_rect(), egui::CornerRadius::ZERO, bg);
                        }
                        let label = match row_ref {
                            RowRef::Original(r) => (r + 1).to_string(),
                            RowRef::Insert(_) => "＋".to_string(),
                        };
                        ui.label(RichText::new(label).size(11.0).weak());
                    });

                    for (ci, col) in columns.iter().enumerate() {
                        let editing_this =
                            matches!(editing.as_ref(), Some(e) if e.row == row_ref && e.col == ci);
                        let display_val = effective_value(rows, edits, row_ref, ci);
                        let cell_bg = row_bg
                            .or_else(|| is_modified(edits, row_ref, ci).then_some(modified_bg));

                        let mut finish: Option<Finish> = None;
                        let (_, resp) = row.col(|ui| {
                            let rect = ui.max_rect();
                            if editing_this {
                                ui.painter().rect_filled(
                                    rect,
                                    egui::CornerRadius::ZERO,
                                    tc.background,
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::CornerRadius::ZERO,
                                    egui::Stroke::new(1.0, tc.button_primary_bg),
                                    egui::StrokeKind::Inside,
                                );
                            } else if let Some(bg) = cell_bg {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg);
                            }

                            if editing_this {
                                if let Some(e) = editing.as_mut() {
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut e.buffer)
                                            .frame(egui::Frame::NONE)
                                            .margin(egui::Margin::symmetric(3, 0))
                                            .desired_width(f32::INFINITY)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    if e.request_focus {
                                        r.request_focus();
                                        e.request_focus = false;
                                    }
                                    if r.lost_focus() {
                                        let (esc, enter) = ui.input(|i| {
                                            (
                                                i.key_pressed(egui::Key::Escape),
                                                i.key_pressed(egui::Key::Enter),
                                            )
                                        });
                                        finish = Some(if esc {
                                            Finish::Cancel
                                        } else if enter {
                                            Finish::CommitNext
                                        } else {
                                            Finish::CommitStop
                                        });
                                    }
                                }
                            } else if is_deleted {
                                let txt = display_val.clone().unwrap_or_else(|| "NULL".into());
                                ui.label(
                                    RichText::new(txt)
                                        .size(12.0)
                                        .strikethrough()
                                        .color(tc.text_disabled),
                                );
                            } else {
                                render_cell(ui, col, &display_val);
                            }
                        });

                        match finish {
                            Some(Finish::Cancel) => *editing = None,
                            Some(Finish::CommitStop) => {
                                if let Some(e) = editing.take() {
                                    apply_edit(edits, rows, e.row, e.col, e.buffer);
                                }
                            }
                            Some(Finish::CommitNext) => {
                                if let Some(e) = editing.take() {
                                    let next = e.col + 1;
                                    apply_edit(edits, rows, e.row, e.col, e.buffer);
                                    if next < columns.len() {
                                        *editing = Some(EditingCell {
                                            row: e.row,
                                            col: next,
                                            buffer: effective_value(rows, edits, e.row, next)
                                                .unwrap_or_default(),
                                            request_focus: true,
                                        });
                                    }
                                }
                            }
                            None => {}
                        }

                        if !editing_this {
                            if resp.double_clicked() {
                                *editing = Some(EditingCell {
                                    row: row_ref,
                                    col: ci,
                                    buffer: display_val.unwrap_or_default(),
                                    request_focus: true,
                                });
                            } else if resp.clicked() {
                                *selected = if is_selected { None } else { Some(display) };
                            }
                        }
                    }
                });
            });
    });
}

fn apply_edit(
    edits: &mut TableEdits,
    rows: &[Vec<Option<String>>],
    row_ref: RowRef,
    col: usize,
    buffer: String,
) {
    let new_val = Some(buffer);
    match row_ref {
        RowRef::Original(r) => {
            let original = rows.get(r).and_then(|row| row.get(col)).cloned().flatten();
            if original == new_val {
                edits.updates.remove(&(r, col));
            } else {
                edits.updates.insert((r, col), new_val);
            }
        }
        RowRef::Insert(i) => {
            if let Some(row) = edits.inserts.get_mut(i)
                && let Some(cell) = row.get_mut(col)
            {
                *cell = new_val;
            }
        }
    }
}

fn effective_value(
    rows: &[Vec<Option<String>>],
    edits: &TableEdits,
    row_ref: RowRef,
    col: usize,
) -> Option<String> {
    match row_ref {
        RowRef::Original(r) => match edits.updates.get(&(r, col)) {
            Some(v) => v.clone(),
            None => rows.get(r).and_then(|row| row.get(col)).cloned().flatten(),
        },
        RowRef::Insert(i) => edits
            .inserts
            .get(i)
            .and_then(|row| row.get(col))
            .cloned()
            .flatten(),
    }
}

fn is_modified(edits: &TableEdits, row_ref: RowRef, col: usize) -> bool {
    matches!(row_ref, RowRef::Original(r) if edits.updates.contains_key(&(r, col)))
}

fn tint(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
