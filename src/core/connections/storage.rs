use crate::core::connections::model::{ConnColor, Connection, DbEngine};
use crate::core::connections::secrets;
use crate::core::ssh::model::{SshAuth, SshConfig};
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
        tracing::debug!("storage: applying schema migration");
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
                ssh_json    TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0
            );
        ",
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> SqliteResult<Vec<Connection>> {
        // Column indices: 0=id 1=name 2=group_name 3=engine 4=color 5=host
        //                 6=port 7=database 8=username 9=ssh_json 10=is_favorite
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_name, engine, color, host, port, database,
                    username, ssh_json, is_favorite
             FROM connections ORDER BY group_name, name",
        )?;
        tracing::debug!("storage: loading all connections");
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // id
                row.get::<_, String>(1)?,  // name
                row.get::<_, String>(2)?,  // group_name
                row.get::<_, String>(3)?,  // engine
                row.get::<_, String>(4)?,  // color
                row.get::<_, String>(5)?,  // host
                row.get::<_, i64>(6)?,     // port
                row.get::<_, String>(7)?,  // database
                row.get::<_, String>(8)?,  // username
                row.get::<_, Option<String>>(9)?,  // ssh_json
                row.get::<_, i64>(10)?,    // is_favorite
            ))
        })?;

        let mut conns = Vec::new();
        for row in rows {
            let (id, name, group, engine_str, color_str, host, port, database, username, ssh_json, is_fav) = row?;

            let password = secrets::load_db_password(&id);

            let ssh = ssh_json.and_then(|j| {
                let mut cfg: SshConfig = serde_json::from_str(&j).ok()?;
                if let SshAuth::Password(_) = &cfg.auth {
                    cfg.auth = SshAuth::Password(secrets::load_ssh_password(&id));
                }
                Some(cfg)
            });

            conns.push(Connection {
                id,
                name,
                group,
                engine: parse_engine(&engine_str),
                color: parse_color(&color_str),
                host,
                port: port as u16,
                database,
                username,
                password,
                ssh,
                is_favorite: is_fav != 0,
            });
        }
        Ok(conns)
    }

    pub fn save(&self, c: &Connection) -> SqliteResult<()> {
        tracing::debug!(conn_id = %c.id, name = %c.name, "storage: save connection");

        // Persist passwords in keychain; keep column empty
        secrets::save_db_password(&c.id, &c.password);

        // Strip SSH password before serializing, save to keychain separately
        let ssh_for_storage = c.ssh.as_ref().map(|ssh| {
            let mut stripped = ssh.clone();
            if let SshAuth::Password(ref p) = ssh.auth {
                secrets::save_ssh_password(&c.id, p);
                stripped.auth = SshAuth::Password(String::new());
            }
            stripped
        });

        let ssh_json = ssh_for_storage
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        self.conn.execute(
            "INSERT OR REPLACE INTO connections
             (id, name, group_name, engine, color, host, port, database,
              username, ssh_json, is_favorite)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
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
                ssh_json,
                c.is_favorite as i64,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> SqliteResult<()> {
        tracing::debug!(conn_id = %id, "storage: delete connection");
        secrets::delete_all(id);
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

