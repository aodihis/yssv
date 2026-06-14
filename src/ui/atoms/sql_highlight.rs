use egui::text::{LayoutJob, TextFormat};
use egui::{Margin, Response, Ui};

use crate::theme::{ThemeColors, colors};

const SQL_KEYWORDS: &[&str] = &[
    "ADD", "ALL", "ALTER", "AND", "AS", "ASC", "AVG",
    "BEGIN", "BETWEEN", "BY",
    "CASE", "CAST", "COALESCE", "COLUMN", "COMMIT", "CONSTRAINT", "COUNT", "CREATE", "CROSS",
    "CURRENT",
    "DATABASE", "DEFAULT", "DELETE", "DESC", "DESCRIBE", "DISTINCT", "DROP",
    "ELSE", "END", "EXCEPT", "EXISTS", "EXPLAIN",
    "FALSE", "FILTER", "FOLLOWING", "FOREIGN", "FROM", "FULL",
    "GRANT", "GROUP",
    "HAVING",
    "IF", "ILIKE", "IN", "INDEX", "INNER", "INSERT", "INTERSECT", "INTO", "IS",
    "JOIN",
    "KEY",
    "LATERAL", "LEFT", "LIKE", "LIMIT",
    "MATERIALIZED", "MAX", "MIN",
    "NATURAL", "NOT", "NULL", "NULLIF",
    "OFFSET", "ON", "OR", "ORDER", "OUTER", "OVER",
    "PARTITION", "PRECEDING", "PRIMARY",
    "RANGE", "RECURSIVE", "REFERENCES", "RENAME", "REPLACE", "RETURNING", "REVOKE", "RIGHT",
    "ROLLBACK", "ROW", "ROWS",
    "SAVEPOINT", "SELECT", "SET", "SHOW", "SUM",
    "TABLE", "TEMP", "TEMPORARY", "THEN", "TO", "TRANSACTION", "TRUE",
    "UNBOUNDED", "UNION", "UNIQUE", "UPDATE", "USE", "USING",
    "VALUES", "VIEW",
    "WHEN", "WHERE", "WINDOW", "WITH",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum TKind {
    Keyword,
    StringLit,
    Number,
    Comment,
    Operator,
    Default,
}

struct Span {
    start: usize,
    end: usize,
    kind: TKind,
}

fn tokenize(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < n {
        // Line comment: --
        if i + 1 < n && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            let start = i;
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TKind::Comment });

        // Block comment: /* ... */
        } else if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < n {
                i += 2;
            }
            spans.push(Span { start, end: i, kind: TKind::Comment });

        // Single-quoted string literal
        } else if bytes[i] == b'\'' {
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' {
                    i += (n - i).min(2);
                } else if bytes[i] == b'\'' {
                    i += 1;
                    // SQL '' escape inside string
                    if i < n && bytes[i] == b'\'' {
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            spans.push(Span { start, end: i, kind: TKind::StringLit });

        // Double-quoted identifier
        } else if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < n && bytes[i] != b'"' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TKind::Default });

        // Backtick identifier (MySQL)
        } else if bytes[i] == b'`' {
            let start = i;
            i += 1;
            while i < n && bytes[i] != b'`' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TKind::Default });

        // Numeric literal
        } else if bytes[i].is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TKind::Number });

        // Identifier or SQL keyword
        } else if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &text[start..i];
            let kind = if SQL_KEYWORDS.iter().any(|&kw| kw.eq_ignore_ascii_case(word)) {
                TKind::Keyword
            } else {
                TKind::Default
            };
            spans.push(Span { start, end: i, kind });

        // Operator / punctuation
        } else if matches!(
            bytes[i],
            b'=' | b'<' | b'>' | b'!' | b'+' | b'*' | b'%'
                | b',' | b'(' | b')' | b';' | b'[' | b']'
                | b'|' | b'&' | b'^' | b'~' | b'/' | b'-'
        ) {
            spans.push(Span { start: i, end: i + 1, kind: TKind::Operator });
            i += 1;

        // Whitespace, '.', '@', '$', ':', '?', etc.
        } else {
            let start = i;
            i += 1;
            while i < n
                && !bytes[i].is_ascii_alphanumeric()
                && bytes[i] != b'_'
                && !matches!(
                    bytes[i],
                    b'\'' | b'"' | b'`' | b'-' | b'/'
                        | b'=' | b'<' | b'>' | b'!' | b'+' | b'*' | b'%'
                        | b',' | b'(' | b')' | b';' | b'[' | b']'
                        | b'|' | b'&' | b'^' | b'~'
                )
            {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TKind::Default });
        }
    }

    spans
}

fn highlight_sql(ui: &Ui, text: &str) -> LayoutJob {
    let tc = ThemeColors::from_ui(ui);
    let mono = egui::FontId::new(12.5, egui::FontFamily::Monospace);

    let fmt = |color: egui::Color32| TextFormat {
        font_id: mono.clone(),
        color,
        ..Default::default()
    };

    let kw_fmt = fmt(tc.button_primary_bg);
    let str_fmt = fmt(tc.success);
    let num_fmt = fmt(colors::AMBER);
    let cmt_fmt = fmt(tc.text_disabled);
    let op_fmt = fmt(tc.text_secondary);
    let def_fmt = fmt(tc.text_primary);

    let mut job = LayoutJob::default();
    for span in tokenize(text) {
        let s = &text[span.start..span.end];
        let f = match span.kind {
            TKind::Keyword => kw_fmt.clone(),
            TKind::StringLit => str_fmt.clone(),
            TKind::Number => num_fmt.clone(),
            TKind::Comment => cmt_fmt.clone(),
            TKind::Operator => op_fmt.clone(),
            TKind::Default => def_fmt.clone(),
        };
        job.append(s, 0.0, f);
    }

    job
}

pub fn sql_area(ui: &mut Ui, value: &mut String, placeholder: &str, min_rows: usize) -> Response {
    let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut job = highlight_sql(ui, text.as_str());
        job.wrap.max_width = wrap_width;
        ui.ctx().fonts_mut(|f| f.layout_job(job))
    };

    ui.add(
        egui::TextEdit::multiline(value)
            .hint_text(placeholder)
            .desired_width(f32::INFINITY)
            .desired_rows(min_rows)
            .margin(Margin { left: 11, right: 11, top: 8, bottom: 8 })
            .font(egui::TextStyle::Monospace)
            .layouter(&mut layouter),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(String, TKind)> {
        tokenize(text)
            .into_iter()
            .map(|s| (text[s.start..s.end].to_string(), s.kind))
            .collect()
    }

    #[test]
    fn keywords_are_highlighted() {
        let spans = kinds("SELECT");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].1, TKind::Keyword);
    }

    #[test]
    fn keywords_are_case_insensitive() {
        let spans = kinds("select");
        assert_eq!(spans[0].1, TKind::Keyword);
        let spans = kinds("Select");
        assert_eq!(spans[0].1, TKind::Keyword);
    }

    #[test]
    fn identifiers_are_default() {
        let spans = kinds("my_table");
        assert_eq!(spans[0].1, TKind::Default);
    }

    #[test]
    fn single_quoted_string_is_string_lit() {
        let spans = kinds("'hello world'");
        assert_eq!(spans[0].1, TKind::StringLit);
        assert_eq!(spans[0].0, "'hello world'");
    }

    #[test]
    fn escaped_quote_inside_string() {
        let spans = kinds("'it''s'");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].1, TKind::StringLit);
        assert_eq!(spans[0].0, "'it''s'");
    }

    #[test]
    fn line_comment_is_comment() {
        let spans = kinds("-- this is a comment");
        assert_eq!(spans[0].1, TKind::Comment);
    }

    #[test]
    fn block_comment_is_comment() {
        let spans = kinds("/* block */");
        assert_eq!(spans[0].1, TKind::Comment);
        assert_eq!(spans[0].0, "/* block */");
    }

    #[test]
    fn numbers_are_number() {
        let spans = kinds("42");
        assert_eq!(spans[0].1, TKind::Number);
        let spans = kinds("3.14");
        assert_eq!(spans[0].1, TKind::Number);
    }

    #[test]
    fn semicolon_is_operator() {
        let spans = kinds(";");
        assert_eq!(spans[0].1, TKind::Operator);
    }

    #[test]
    fn mixed_query_produces_correct_kinds() {
        let spans = kinds("SELECT id FROM users WHERE id = 1");
        let kw: Vec<_> = spans.iter().filter(|(_, k)| *k == TKind::Keyword).collect();
        let num: Vec<_> = spans.iter().filter(|(_, k)| *k == TKind::Number).collect();
        assert!(kw.iter().any(|(t, _)| t == "SELECT"));
        assert!(kw.iter().any(|(t, _)| t == "FROM"));
        assert!(kw.iter().any(|(t, _)| t == "WHERE"));
        assert!(num.iter().any(|(t, _)| t == "1"));
    }
}
