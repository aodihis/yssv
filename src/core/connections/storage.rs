use crate::core::connections::model::{ConnColor, Connection, DbEngine};
use rusqlite::{Connection as SqliteConn, Result as SqliteResult, params};

pub struct Storage {
    conn: SqliteConn,
}

impl Storage {
    pub fn open(path: &str) -> SqliteResult<Self> {
        let conn = SqliteConn::open(path)?;
        let s = Self { conn };
        s.migrate()?;
        Ok(s)
    }

    pub fn open_in_memory() -> SqliteResult<Self> {
        let conn = SqliteConn::open_in_memory()?;
        let s = Self { conn };
        s.migrate()?;
        Ok(s)
    }

    fn migrate(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS connections (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                group_name  TEXT NOT NULL DEFAULT '',
                engine      TEXT NOT NULL,
                color       TEXT NOT NULL,
                host        TEXT NOT NULL,
                port        INTEGER NOT NULL,
                database    TEXT NOT NULL DEFAULT '',
                username    TEXT NOT NULL DEFAULT '',
                password    TEXT NOT NULL DEFAULT '',
                ssh_json    TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0
            );
        ",
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> SqliteResult<Vec<Connection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_name, engine, color, host, port, database,
                    username, password, ssh_json, is_favorite
             FROM connections ORDER BY group_name, name",
        )?;
        let rows = stmt.query_map([], |row| {
            let engine_str: String = row.get(3)?;
            let color_str: String = row.get(4)?;
            let ssh_json: Option<String> = row.get(10)?;
            Ok(Connection {
                id: row.get(0)?,
                name: row.get(1)?,
                group: row.get(2)?,
                engine: parse_engine(&engine_str),
                color: parse_color(&color_str),
                host: row.get(5)?,
                port: row.get::<_, i64>(6)? as u16,
                database: row.get(7)?,
                username: row.get(8)?,
                password: row.get(9)?,
                ssh: ssh_json.and_then(|j| serde_json::from_str(&j).ok()),
                is_favorite: row.get::<_, i64>(11)? != 0,
            })
        })?;
        rows.collect()
    }

    pub fn save(&self, c: &Connection) -> SqliteResult<()> {
        tracing::debug!(conn_id = %c.id, name = %c.name, "storage: save connection");
        let ssh_json = c.ssh.as_ref().and_then(|s| serde_json::to_string(s).ok());
        self.conn.execute(
            "INSERT OR REPLACE INTO connections
             (id, name, group_name, engine, color, host, port, database,
              username, password, ssh_json, is_favorite)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                c.id,
                c.name,
                c.group,
                engine_str(&c.engine),
                color_str(&c.color),
                c.host,
                c.port as i64,
                c.database,
                c.username,
                c.password,
                ssh_json,
                c.is_favorite as i64,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> SqliteResult<()> {
        tracing::debug!(conn_id = %id, "storage: delete connection");
        self.conn
            .execute("DELETE FROM connections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_groups(&self) -> SqliteResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT group_name FROM connections ORDER BY group_name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }
}

fn engine_str(e: &DbEngine) -> &'static str {
    match e {
        DbEngine::Postgres => "postgres",
        DbEngine::MySQL => "mysql",
    }
}

fn parse_engine(s: &str) -> DbEngine {
    match s {
        "mysql" => DbEngine::MySQL,
        _ => DbEngine::Postgres,
    }
}

fn color_str(c: &ConnColor) -> &'static str {
    match c {
        ConnColor::Red => "red",
        ConnColor::Amber => "amber",
        ConnColor::Green => "green",
        ConnColor::Blue => "blue",
        ConnColor::Purple => "purple",
        ConnColor::Gray => "gray",
    }
}

fn parse_color(s: &str) -> ConnColor {
    match s {
        "amber" => ConnColor::Amber,
        "green" => ConnColor::Green,
        "blue" => ConnColor::Blue,
        "purple" => ConnColor::Purple,
        "gray" => ConnColor::Gray,
        _ => ConnColor::Red,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::connections::model::Connection;
    use crate::core::ssh::model::{SshAuth, SshConfig};

    fn make_conn(name: &str) -> Connection {
        let mut c = Connection::new_postgres();
        c.name = name.into();
        c.group = "Local".into();
        c.database = "testdb".into();
        c.username = "user".into();
        c
    }

    #[test]
    fn crud_roundtrip() {
        let s = Storage::open_in_memory().unwrap();
        let c = make_conn("Test Conn");
        s.save(&c).unwrap();

        let all = s.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Test Conn");
        assert_eq!(all[0].engine, DbEngine::Postgres);
    }

    #[test]
    fn delete_removes_record() {
        let s = Storage::open_in_memory().unwrap();
        let c = make_conn("To Delete");
        s.save(&c).unwrap();
        s.delete(&c.id).unwrap();
        assert!(s.load_all().unwrap().is_empty());
    }

    #[test]
    fn save_updates_existing() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = make_conn("Original");
        s.save(&c).unwrap();
        c.name = "Updated".into();
        s.save(&c).unwrap();

        let all = s.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Updated");
    }

    #[test]
    fn list_groups_deduplicates() {
        let s = Storage::open_in_memory().unwrap();
        let mut c1 = make_conn("A");
        c1.group = "Production".into();
        let mut c2 = make_conn("B");
        c2.group = "Local".into();
        let mut c3 = make_conn("C");
        c3.group = "Local".into();
        s.save(&c1).unwrap();
        s.save(&c2).unwrap();
        s.save(&c3).unwrap();

        let groups = s.list_groups().unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn ssh_config_persisted() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = make_conn("With SSH");
        c.ssh = Some(SshConfig {
            host: "bastion.example.com".into(),
            port: 22,
            username: "admin".into(),
            auth: SshAuth::Password("secret".into()),
        });
        s.save(&c).unwrap();

        let all = s.load_all().unwrap();
        assert!(all[0].ssh.is_some());
        assert_eq!(all[0].ssh.as_ref().unwrap().host, "bastion.example.com");
    }

    #[test]
    fn favorite_flag_persisted() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = make_conn("Fav");
        c.is_favorite = true;
        s.save(&c).unwrap();
        assert!(s.load_all().unwrap()[0].is_favorite);
    }
}
