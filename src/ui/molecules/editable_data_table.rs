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

    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui_extras::TableBuilder::new(ui)
            .striped(false)
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
                results_header(&mut header, columns, &tc);
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

#[cfg(test)]
mod tests {
    use super::*;
    fn base_rows() -> Vec<Vec<Option<String>>> {
        vec![
            vec![Some("1".into()), Some("alice".into())],
            vec![Some("2".into()), None],
        ]
    }

    fn empty_edits() -> TableEdits {
        TableEdits::default()
    }

    // --- tint ---

    #[test]
    fn tint_preserves_rgb_sets_alpha() {
        // Use a fully-opaque source so premultiplied r/g/b == original r/g/b.
        let src = Color32::from_rgb(100, 150, 200);
        let c = tint(src, 255);
        assert_eq!(c.r(), src.r());
        assert_eq!(c.g(), src.g());
        assert_eq!(c.b(), src.b());
        assert_eq!(c.a(), 255);
    }

    #[test]
    fn tint_reduces_alpha() {
        let src = Color32::from_rgb(200, 100, 50);
        let c = tint(src, 30);
        assert_eq!(c.a(), 30);
    }

    #[test]
    fn tint_zero_alpha_is_transparent() {
        let c = tint(Color32::RED, 0);
        assert_eq!(c.a(), 0);
    }

    // --- is_modified ---

    #[test]
    fn is_modified_false_when_no_updates() {
        let edits = empty_edits();
        assert!(!is_modified(&edits, RowRef::Original(0), 1));
    }

    #[test]
    fn is_modified_true_when_update_present() {
        let mut edits = empty_edits();
        edits.updates.insert((0, 1), Some("x".into()));
        assert!(is_modified(&edits, RowRef::Original(0), 1));
    }

    #[test]
    fn is_modified_false_for_insert_row() {
        let mut edits = empty_edits();
        edits.updates.insert((0, 1), Some("x".into()));
        // Insert rows are never tracked in `updates`
        assert!(!is_modified(&edits, RowRef::Insert(0), 1));
    }

    // --- effective_value ---

    #[test]
    fn effective_value_original_no_edit() {
        let rows = base_rows();
        let edits = empty_edits();
        assert_eq!(
            effective_value(&rows, &edits, RowRef::Original(0), 1),
            Some("alice".into())
        );
    }

    #[test]
    fn effective_value_original_null_no_edit() {
        let rows = base_rows();
        let edits = empty_edits();
        assert_eq!(effective_value(&rows, &edits, RowRef::Original(1), 1), None);
    }

    #[test]
    fn effective_value_original_with_update() {
        let rows = base_rows();
        let mut edits = empty_edits();
        edits.updates.insert((0, 1), Some("bob".into()));
        assert_eq!(
            effective_value(&rows, &edits, RowRef::Original(0), 1),
            Some("bob".into())
        );
    }

    #[test]
    fn effective_value_original_update_to_null() {
        let rows = base_rows();
        let mut edits = empty_edits();
        edits.updates.insert((0, 1), None);
        assert_eq!(effective_value(&rows, &edits, RowRef::Original(0), 1), None);
    }

    #[test]
    fn effective_value_insert_row() {
        let rows = base_rows();
        let mut edits = empty_edits();
        edits.inserts.push(vec![Some("99".into()), None]);
        assert_eq!(
            effective_value(&rows, &edits, RowRef::Insert(0), 0),
            Some("99".into())
        );
        assert_eq!(effective_value(&rows, &edits, RowRef::Insert(0), 1), None);
    }

    // --- apply_edit ---

    #[test]
    fn apply_edit_original_changed_value_inserts_update() {
        let rows = base_rows();
        let mut edits = empty_edits();
        apply_edit(&mut edits, &rows, RowRef::Original(0), 1, "bob".into());
        assert_eq!(edits.updates.get(&(0, 1)), Some(&Some("bob".into())));
    }

    #[test]
    fn apply_edit_original_same_value_removes_update() {
        let rows = base_rows();
        let mut edits = empty_edits();
        // Pre-insert a stale edit, then apply the original value back.
        edits.updates.insert((0, 1), Some("stale".into()));
        apply_edit(&mut edits, &rows, RowRef::Original(0), 1, "alice".into());
        assert!(!edits.updates.contains_key(&(0, 1)));
    }

    #[test]
    fn apply_edit_original_null_cell_tracks_new_value() {
        let rows = base_rows();
        let mut edits = empty_edits();
        // row 1 col 1 is NULL — any non-null edit is a change
        apply_edit(&mut edits, &rows, RowRef::Original(1), 1, "filled".into());
        assert_eq!(edits.updates.get(&(1, 1)), Some(&Some("filled".into())));
    }

    #[test]
    fn apply_edit_insert_row_sets_cell() {
        let rows = base_rows();
        let mut edits = empty_edits();
        edits.inserts.push(vec![None, None]);
        apply_edit(&mut edits, &rows, RowRef::Insert(0), 1, "typed".into());
        assert_eq!(edits.inserts[0][1], Some("typed".into()));
    }
}
