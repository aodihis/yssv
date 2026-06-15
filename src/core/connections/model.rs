use crate::core::ssh::SshConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub group: String,
    pub engine: DbEngine,
    pub color: ConnColor,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssh: Option<SshConfig>,
    pub is_favorite: bool,
}

impl Connection {
    pub fn new_postgres() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            group: "Local".into(),
            engine: DbEngine::Postgres,
            color: ConnColor::Green,
            host: "127.0.0.1".into(),
            port: 5432,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            ssh: None,
            is_favorite: false,
        }
    }

    pub fn new_mysql() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            group: "Local".into(),
            engine: DbEngine::MySQL,
            color: ConnColor::Blue,
            host: "127.0.0.1".into(),
            port: 3306,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            ssh: None,
            is_favorite: false,
        }
    }

    pub fn display_host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbEngine {
    Postgres,
    MySQL,
}

impl DbEngine {
    pub fn label(&self) -> &'static str {
        match self {
            DbEngine::Postgres => "PostgreSQL",
            DbEngine::MySQL => "MySQL",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            DbEngine::Postgres => "PG",
            DbEngine::MySQL => "My",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DbEngine::Postgres => 5432,
            DbEngine::MySQL => 3306,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnColor {
    Red,
    Amber,
    Green,
    Blue,
    Purple,
    Gray,
}

impl ConnColor {
    pub fn all() -> &'static [ConnColor] {
        &[
            ConnColor::Red,
            ConnColor::Amber,
            ConnColor::Green,
            ConnColor::Blue,
            ConnColor::Purple,
            ConnColor::Gray,
        ]
    }

    pub fn to_color32(self) -> egui::Color32 {
        use crate::theme::colors;
        match self {
            ConnColor::Red => colors::RED,
            ConnColor::Amber => colors::AMBER,
            ConnColor::Green => colors::GREEN,
            ConnColor::Blue => colors::BLUE,
            ConnColor::Purple => colors::PURPLE,
            ConnColor::Gray => colors::GRAY,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConnColor::Red => "Red",
            ConnColor::Amber => "Amber",
            ConnColor::Green => "Green",
            ConnColor::Blue => "Blue",
            ConnColor::Purple => "Purple",
            ConnColor::Gray => "Gray",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_postgres_defaults() {
        let c = Connection::new_postgres();
        assert_eq!(c.engine, DbEngine::Postgres);
        assert_eq!(c.port, 5432);
        assert!(!c.id.is_empty());
    }

    #[test]
    fn new_mysql_defaults() {
        let c = Connection::new_mysql();
        assert_eq!(c.engine, DbEngine::MySQL);
        assert_eq!(c.port, 3306);
    }

    #[test]
    fn display_host_format() {
        let mut c = Connection::new_postgres();
        c.host = "db.example.com".into();
        c.port = 5433;
        assert_eq!(c.display_host(), "db.example.com:5433");
    }

    #[test]
    fn connection_serialization_roundtrip() {
        let mut c = Connection::new_postgres();
        c.name = "Test".into();
        c.database = "mydb".into();
        let json = serde_json::to_string(&c).unwrap();
        let decoded: Connection = serde_json::from_str(&json).unwrap();
        assert_eq!(c, decoded);
    }

    #[test]
    fn db_engine_labels() {
        assert_eq!(DbEngine::Postgres.label(), "PostgreSQL");
        assert_eq!(DbEngine::MySQL.label(), "MySQL");
    }

    #[test]
    fn db_engine_short() {
        assert_eq!(DbEngine::Postgres.short(), "PG");
        assert_eq!(DbEngine::MySQL.short(), "My");
    }

    #[test]
    fn db_engine_default_port() {
        assert_eq!(DbEngine::Postgres.default_port(), 5432);
        assert_eq!(DbEngine::MySQL.default_port(), 3306);
    }

    #[test]
    fn conn_color_all_has_six_variants() {
        assert_eq!(ConnColor::all().len(), 6);
    }

    #[test]
    fn conn_color_labels() {
        assert_eq!(ConnColor::Red.label(), "Red");
        assert_eq!(ConnColor::Amber.label(), "Amber");
        assert_eq!(ConnColor::Green.label(), "Green");
        assert_eq!(ConnColor::Blue.label(), "Blue");
        assert_eq!(ConnColor::Purple.label(), "Purple");
        assert_eq!(ConnColor::Gray.label(), "Gray");
    }

    #[test]
    fn new_postgres_and_mysql_have_unique_ids() {
        let a = Connection::new_postgres();
        let b = Connection::new_mysql();
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_postgres_default_group_and_color() {
        let c = Connection::new_postgres();
        assert_eq!(c.group, "Local");
        assert_eq!(c.color, ConnColor::Green);
        assert!(!c.is_favorite);
        assert!(c.ssh.is_none());
    }

    #[test]
    fn new_mysql_default_group_and_color() {
        let c = Connection::new_mysql();
        assert_eq!(c.group, "Local");
        assert_eq!(c.color, ConnColor::Blue);
        assert!(!c.is_favorite);
        assert!(c.ssh.is_none());
    }

    #[test]
    fn conn_color_to_color32_all_distinct() {
        use std::collections::HashSet;
        let values: HashSet<[u8; 4]> = ConnColor::all()
            .iter()
            .map(|c| c.to_color32().to_array())
            .collect();
        assert_eq!(
            values.len(),
            ConnColor::all().len(),
            "each ConnColor variant must map to a unique Color32"
        );
    }

    #[test]
    fn connection_with_ssh_serializes_and_deserializes() {
        use crate::core::ssh::model::{SshAuth, SshConfig};
        let mut c = Connection::new_postgres();
        c.ssh = Some(SshConfig {
            host: "bastion.io".into(),
            port: 22,
            username: "admin".into(),
            auth: SshAuth::Password("pass".into()),
        });
        let json = serde_json::to_string(&c).unwrap();
        let decoded: Connection = serde_json::from_str(&json).unwrap();
        assert_eq!(c, decoded);
    }

    #[test]
    fn display_host_uses_custom_port() {
        let mut c = Connection::new_postgres();
        c.host = "127.0.0.1".into();
        c.port = 5432;
        assert_eq!(c.display_host(), "127.0.0.1:5432");
    }
}
