use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: Option<u64>,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            total_rows: Some(0),
        }
    }

    pub fn page_count(&self, page_size: u32) -> u64 {
        match self.total_rows {
            Some(total) if page_size > 0 => total.div_ceil(page_size as u64),
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_pk: bool,
    pub is_fk: bool,
    pub nullable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        let r = QueryResult::empty();
        assert!(r.columns.is_empty());
        assert!(r.rows.is_empty());
        assert_eq!(r.total_rows, Some(0));
    }

    #[test]
    fn page_count_exact_multiple() {
        let r = QueryResult {
            columns: vec![],
            rows: vec![],
            total_rows: Some(200),
        };
        assert_eq!(r.page_count(100), 2);
    }

    #[test]
    fn page_count_remainder() {
        let r = QueryResult {
            columns: vec![],
            rows: vec![],
            total_rows: Some(201),
        };
        assert_eq!(r.page_count(100), 3);
    }

    #[test]
    fn page_count_unknown_total() {
        let r = QueryResult {
            columns: vec![],
            rows: vec![],
            total_rows: None,
        };
        assert_eq!(r.page_count(100), 1);
    }

    #[test]
    fn page_count_zero_page_size_returns_one() {
        let r = QueryResult {
            columns: vec![],
            rows: vec![],
            total_rows: Some(500),
        };
        assert_eq!(r.page_count(0), 1);
    }

    #[test]
    fn column_def_serialization_roundtrip() {
        let col = ColumnDef {
            name: "id".into(),
            data_type: "integer".into(),
            is_pk: true,
            is_fk: false,
            nullable: false,
        };
        let json = serde_json::to_string(&col).unwrap();
        let decoded: ColumnDef = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "id");
        assert!(decoded.is_pk);
        assert!(!decoded.nullable);
    }
}
