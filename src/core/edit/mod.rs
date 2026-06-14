use std::collections::{HashMap, HashSet};

use crate::core::connections::model::DbEngine;
use crate::core::results::model::ColumnDef;

/// Pending, uncommitted edits to a single table's current page of rows.
///
/// Mirrors the DataGrip model: changes accumulate locally and only reach the
/// database when the user commits. Indices are relative to the currently
/// loaded page of `result.rows`, so the edit set is cleared whenever the page
/// reloads.
#[derive(Debug, Clone, Default)]
pub struct TableEdits {
    /// `(original row index, column index)` -> new value (`None` = SQL NULL).
    pub updates: HashMap<(usize, usize), Option<String>>,
    /// Newly added rows, each aligned to the column order. A cell left as
    /// `None` is written as `DEFAULT` so serial / auto-increment keys work.
    pub inserts: Vec<Vec<Option<String>>>,
    /// Original row indices marked for deletion.
    pub deletes: HashSet<usize>,
}

impl TableEdits {
    pub fn clear(&mut self) {
        self.updates.clear();
        self.inserts.clear();
        self.deletes.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty() && self.inserts.is_empty() && self.deletes.is_empty()
    }

    /// Total number of pending operations (updated rows + inserts + deletes).
    pub fn pending_count(&self) -> usize {
        self.updated_rows().len() + self.inserts.len() + self.deletes.len()
    }

    /// Distinct original rows with at least one edited cell, excluding rows
    /// also marked for deletion (a delete supersedes an update).
    pub fn updated_rows(&self) -> Vec<usize> {
        let mut rows: Vec<usize> = self
            .updates
            .keys()
            .map(|(r, _)| *r)
            .filter(|r| !self.deletes.contains(r))
            .collect();
        rows.sort_unstable();
        rows.dedup();
        rows
    }
}

/// Build the ordered list of SQL statements that apply `edits` to `table`.
///
/// Order is UPDATE -> DELETE -> INSERT. Row identity for UPDATE/DELETE uses the
/// primary-key columns when the result has any, otherwise every column's
/// original value (so a table without a PK still produces a precise `WHERE`).
pub fn build_statements(
    engine: DbEngine,
    schema: &str,
    table: &str,
    columns: &[ColumnDef],
    rows: &[Vec<Option<String>>],
    edits: &TableEdits,
) -> Vec<String> {
    let target = qualified(engine, schema, table);
    let ids = identity_indices(columns);
    let mut stmts = Vec::new();

    for r in edits.updated_rows() {
        let Some(orig) = rows.get(r) else { continue };
        let mut sets: Vec<(usize, &Option<String>)> = edits
            .updates
            .iter()
            .filter(|((rr, _), _)| *rr == r)
            .map(|((_, c), v)| (*c, v))
            .collect();
        sets.sort_by_key(|(c, _)| *c);
        let set_clause = sets
            .iter()
            .map(|(c, v)| {
                format!(
                    "{} = {}",
                    quote_ident(engine, &columns[*c].name),
                    quote_value(engine, v.as_deref())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let where_clause = identity_where(engine, columns, orig, &ids);
        stmts.push(format!("UPDATE {target} SET {set_clause} WHERE {where_clause}"));
    }

    let mut dels: Vec<usize> = edits.deletes.iter().copied().collect();
    dels.sort_unstable();
    for r in dels {
        let Some(orig) = rows.get(r) else { continue };
        let where_clause = identity_where(engine, columns, orig, &ids);
        stmts.push(format!("DELETE FROM {target} WHERE {where_clause}"));
    }

    for row in &edits.inserts {
        let col_list = columns
            .iter()
            .map(|c| quote_ident(engine, &c.name))
            .collect::<Vec<_>>()
            .join(", ");
        let val_list = (0..columns.len())
            .map(|i| match row.get(i) {
                Some(Some(v)) => quote_value(engine, Some(v)),
                _ => "DEFAULT".to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        stmts.push(format!("INSERT INTO {target} ({col_list}) VALUES ({val_list})"));
    }

    stmts
}

pub(crate) fn quote_ident(engine: DbEngine, name: &str) -> String {
    match engine {
        DbEngine::Postgres => format!("\"{}\"", name.replace('"', "\"\"")),
        DbEngine::MySQL => format!("`{}`", name.replace('`', "``")),
    }
}

pub(crate) fn quote_value(engine: DbEngine, value: Option<&str>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(v) => match engine {
            DbEngine::Postgres => format!("'{}'", v.replace('\'', "''")),
            DbEngine::MySQL => format!("'{}'", v.replace('\\', "\\\\").replace('\'', "''")),
        },
    }
}

fn identity_indices(columns: &[ColumnDef]) -> Vec<usize> {
    let pks: Vec<usize> = columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_pk)
        .map(|(i, _)| i)
        .collect();
    if pks.is_empty() {
        (0..columns.len()).collect()
    } else {
        pks
    }
}

fn identity_where(
    engine: DbEngine,
    columns: &[ColumnDef],
    row: &[Option<String>],
    ids: &[usize],
) -> String {
    ids.iter()
        .map(|&i| {
            let col = quote_ident(engine, &columns[i].name);
            match row.get(i).and_then(|v| v.as_deref()) {
                Some(v) => format!("{col} = {}", quote_value(engine, Some(v))),
                None => format!("{col} IS NULL"),
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn qualified(engine: DbEngine, schema: &str, table: &str) -> String {
    format!("{}.{}", quote_ident(engine, schema), quote_ident(engine, table))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str, is_pk: bool) -> ColumnDef {
        ColumnDef {
            name: name.into(),
            data_type: "text".into(),
            is_pk,
            is_fk: false,
            nullable: true,
        }
    }

    fn cols() -> Vec<ColumnDef> {
        vec![col("id", true), col("name", false), col("note", false)]
    }

    fn rows() -> Vec<Vec<Option<String>>> {
        vec![
            vec![Some("1".into()), Some("alice".into()), None],
            vec![Some("2".into()), Some("bob".into()), Some("hi".into())],
        ]
    }

    #[test]
    fn quote_ident_postgres_uses_double_quotes() {
        assert_eq!(quote_ident(DbEngine::Postgres, "name"), "\"name\"");
    }

    #[test]
    fn quote_ident_mysql_uses_backticks() {
        assert_eq!(quote_ident(DbEngine::MySQL, "name"), "`name`");
    }

    #[test]
    fn quote_ident_escapes_embedded_quote() {
        assert_eq!(quote_ident(DbEngine::Postgres, "we\"ird"), "\"we\"\"ird\"");
        assert_eq!(quote_ident(DbEngine::MySQL, "we`ird"), "`we``ird`");
    }

    #[test]
    fn quote_value_none_is_null() {
        assert_eq!(quote_value(DbEngine::Postgres, None), "NULL");
        assert_eq!(quote_value(DbEngine::MySQL, None), "NULL");
    }

    #[test]
    fn quote_value_escapes_single_quote() {
        assert_eq!(quote_value(DbEngine::Postgres, Some("it's")), "'it''s'");
        assert_eq!(quote_value(DbEngine::MySQL, Some("it's")), "'it''s'");
    }

    #[test]
    fn quote_value_mysql_escapes_backslash() {
        assert_eq!(quote_value(DbEngine::MySQL, Some("a\\b")), "'a\\\\b'");
    }

    #[test]
    fn quote_value_postgres_keeps_backslash_literal() {
        assert_eq!(quote_value(DbEngine::Postgres, Some("a\\b")), "'a\\b'");
    }

    #[test]
    fn empty_edits_produce_no_statements() {
        let edits = TableEdits::default();
        assert!(build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits).is_empty());
    }

    #[test]
    fn update_uses_pk_in_where_and_changed_columns_in_set() {
        let mut edits = TableEdits::default();
        edits.updates.insert((0, 1), Some("ALICE".into()));
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        assert_eq!(
            stmts,
            vec![r#"UPDATE "public"."users" SET "name" = 'ALICE' WHERE "id" = '1'"#]
        );
    }

    #[test]
    fn update_with_multiple_columns_is_sorted_by_index() {
        let mut edits = TableEdits::default();
        edits.updates.insert((1, 2), Some("yo".into()));
        edits.updates.insert((1, 1), Some("BOB".into()));
        let stmts = build_statements(DbEngine::MySQL, "shop", "users", &cols(), &rows(), &edits);
        assert_eq!(
            stmts,
            vec!["UPDATE `shop`.`users` SET `name` = 'BOB', `note` = 'yo' WHERE `id` = '2'"]
        );
    }

    #[test]
    fn update_to_null_renders_null() {
        let mut edits = TableEdits::default();
        edits.updates.insert((1, 2), None);
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        assert_eq!(
            stmts,
            vec![r#"UPDATE "public"."users" SET "note" = NULL WHERE "id" = '2'"#]
        );
    }

    #[test]
    fn delete_uses_pk_where() {
        let mut edits = TableEdits::default();
        edits.deletes.insert(0);
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        assert_eq!(stmts, vec![r#"DELETE FROM "public"."users" WHERE "id" = '1'"#]);
    }

    #[test]
    fn delete_supersedes_update_for_same_row() {
        let mut edits = TableEdits::default();
        edits.updates.insert((0, 1), Some("x".into()));
        edits.deletes.insert(0);
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        // No UPDATE for row 0 — only the DELETE.
        assert_eq!(stmts, vec![r#"DELETE FROM "public"."users" WHERE "id" = '1'"#]);
    }

    #[test]
    fn insert_emits_default_for_untouched_cells() {
        let mut edits = TableEdits::default();
        edits.inserts.push(vec![None, Some("carol".into()), None]);
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        assert_eq!(
            stmts,
            vec![r#"INSERT INTO "public"."users" ("id", "name", "note") VALUES (DEFAULT, 'carol', DEFAULT)"#]
        );
    }

    #[test]
    fn no_pk_table_uses_all_columns_in_where() {
        let columns = vec![col("a", false), col("b", false)];
        let rows = vec![vec![Some("x".into()), None]];
        let mut edits = TableEdits::default();
        edits.deletes.insert(0);
        let stmts = build_statements(DbEngine::Postgres, "public", "t", &columns, &rows, &edits);
        assert_eq!(
            stmts,
            vec![r#"DELETE FROM "public"."t" WHERE "a" = 'x' AND "b" IS NULL"#]
        );
    }

    #[test]
    fn statements_are_ordered_update_delete_insert() {
        let mut edits = TableEdits::default();
        edits.updates.insert((0, 1), Some("A".into()));
        edits.deletes.insert(1);
        edits.inserts.push(vec![Some("9".into()), Some("z".into()), None]);
        let stmts = build_statements(DbEngine::Postgres, "public", "users", &cols(), &rows(), &edits);
        assert_eq!(stmts.len(), 3);
        assert!(stmts[0].starts_with("UPDATE"));
        assert!(stmts[1].starts_with("DELETE"));
        assert!(stmts[2].starts_with("INSERT"));
    }

    #[test]
    fn pending_count_counts_rows_not_cells() {
        let mut edits = TableEdits::default();
        edits.updates.insert((0, 1), Some("A".into()));
        edits.updates.insert((0, 2), Some("B".into()));
        edits.inserts.push(vec![None, None, None]);
        edits.deletes.insert(1);
        // 1 updated row + 1 insert + 1 delete
        assert_eq!(edits.pending_count(), 3);
    }
}
