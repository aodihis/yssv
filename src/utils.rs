pub fn data_dir_path() -> String {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        format!("{}\\yssv\\connections.db", base)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/.config/yssv/connections.db", home)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_path_ends_with_connections_db() {
        let path = data_dir_path();
        assert!(path.ends_with("connections.db"), "got: {path}");
    }

    #[test]
    fn data_dir_path_contains_yssv() {
        let path = data_dir_path();
        assert!(path.contains("yssv"), "got: {path}");
    }
}
