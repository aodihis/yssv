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
}
