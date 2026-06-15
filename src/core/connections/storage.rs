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
                is_favorite INTEGER NOT NULL DEFAULT 0,
                sort_order  INTEGER NOT NULL DEFAULT 0
            );
        ",
        )?;
        // Idempotent: add sort_order to existing tables that predate the column
        let _ = self.conn.execute_batch(
            "ALTER TABLE connections ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
        );
        Ok(())
    }

    pub fn load_all(&self) -> SqliteResult<Vec<Connection>> {
        // Column indices: 0=id 1=name 2=group_name 3=engine 4=color 5=host
        //                 6=port 7=database 8=username 9=ssh_json 10=is_favorite
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_name, engine, color, host, port, database,
                    username, ssh_json, is_favorite
             FROM connections ORDER BY sort_order, group_name, name",
        )?;
        tracing::debug!("storage: loading all connections");
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,         // id
                row.get::<_, String>(1)?,         // name
                row.get::<_, String>(2)?,         // group_name
                row.get::<_, String>(3)?,         // engine
                row.get::<_, String>(4)?,         // color
                row.get::<_, String>(5)?,         // host
                row.get::<_, i64>(6)?,            // port
                row.get::<_, String>(7)?,         // database
                row.get::<_, String>(8)?,         // username
                row.get::<_, Option<String>>(9)?, // ssh_json
                row.get::<_, i64>(10)?,           // is_favorite
            ))
        })?;

        let mut conns = Vec::new();
        for row in rows {
            let (
                id,
                name,
                group,
                engine_str,
                color_str,
                host,
                port,
                database,
                username,
                ssh_json,
                is_fav,
            ) = row?;

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

    pub fn save_order(&self, ordered_ids: &[String]) -> SqliteResult<()> {
        tracing::debug!(
            count = ordered_ids.len(),
            "storage: persisting connection sort order"
        );
        for (i, id) in ordered_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE connections SET sort_order = ?1 WHERE id = ?2",
                params![i as i64, id],
            )?;
        }
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

    fn conn(name: &str) -> Connection {
        let mut c = Connection::new_postgres();
        c.name = name.into();
        c
    }

    #[test]
    fn engine_str_and_parse_engine_roundtrip() {
        for e in [DbEngine::Postgres, DbEngine::MySQL] {
            assert_eq!(parse_engine(engine_str(&e)), e);
        }
        // Unknown engine string falls back to Postgres.
        assert_eq!(parse_engine("sqlite"), DbEngine::Postgres);
    }

    #[test]
    fn color_str_and_parse_color_roundtrip_all_variants() {
        for &c in ConnColor::all() {
            assert_eq!(parse_color(color_str(&c)), c);
        }
        // Unknown color string falls back to Red.
        assert_eq!(parse_color("chartreuse"), ConnColor::Red);
    }

    #[test]
    fn save_order_reorders_load_all() {
        let s = Storage::open_in_memory().unwrap();
        let a = conn("a");
        let b = conn("b");
        let c = conn("c");
        s.save(&a).unwrap();
        s.save(&b).unwrap();
        s.save(&c).unwrap();

        // Force an explicit order: c, a, b
        s.save_order(&[c.id.clone(), a.id.clone(), b.id.clone()])
            .unwrap();
        let loaded = s.load_all().unwrap();
        let names: Vec<&str> = loaded.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, vec!["c", "a", "b"]);
    }

    #[test]
    fn save_order_ignores_unknown_ids() {
        let s = Storage::open_in_memory().unwrap();
        let a = conn("a");
        s.save(&a).unwrap();
        // Unknown id is simply a no-op UPDATE — must not error.
        assert!(s.save_order(&["ghost".into(), a.id.clone()]).is_ok());
    }

    #[test]
    fn open_file_backed_storage_migrates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conns.db");
        let path_str = path.to_string_lossy().to_string();
        {
            let s = Storage::open(&path_str).unwrap();
            s.save(&conn("persisted")).unwrap();
        }
        // Reopening the same file must find the migrated table + saved row.
        let s2 = Storage::open(&path_str).unwrap();
        assert_eq!(s2.load_all().unwrap().len(), 1);
    }

    #[test]
    fn load_all_empty_when_no_connections_saved() {
        let s = Storage::open_in_memory().unwrap();
        assert!(s.load_all().unwrap().is_empty());
    }

    #[test]
    fn save_and_load_preserves_all_basic_fields() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = Connection::new_mysql();
        c.name = "Production".into();
        c.host = "db.prod.io".into();
        c.port = 3307;
        c.database = "app".into();
        c.username = "root".into();
        c.group = "Prod Group".into();
        c.is_favorite = true;
        let orig_id = c.id.clone();
        s.save(&c).unwrap();

        let loaded = s.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        let l = &loaded[0];
        assert_eq!(l.id, orig_id);
        assert_eq!(l.name, "Production");
        assert_eq!(l.host, "db.prod.io");
        assert_eq!(l.port, 3307);
        assert_eq!(l.database, "app");
        assert_eq!(l.username, "root");
        assert_eq!(l.group, "Prod Group");
        assert_eq!(l.engine, DbEngine::MySQL);
        assert!(l.is_favorite);
    }

    #[test]
    fn save_updates_existing_connection_without_duplication() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = conn("original");
        let id = c.id.clone();
        s.save(&c).unwrap();

        c.name = "updated".into();
        s.save(&c).unwrap();

        let loaded = s.load_all().unwrap();
        assert_eq!(loaded.len(), 1, "upsert must not create a duplicate row");
        assert_eq!(loaded[0].id, id);
        assert_eq!(loaded[0].name, "updated");
    }

    #[test]
    fn delete_removes_connection_from_storage() {
        let s = Storage::open_in_memory().unwrap();
        let a = conn("to-delete");
        let b = conn("keep");
        let del_id = a.id.clone();
        s.save(&a).unwrap();
        s.save(&b).unwrap();
        s.delete(&del_id).unwrap();

        let loaded = s.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "keep");
    }

    #[test]
    fn delete_nonexistent_id_is_ok() {
        let s = Storage::open_in_memory().unwrap();
        assert!(s.delete("ghost-id").is_ok());
    }

    #[test]
    fn list_groups_returns_distinct_group_names() {
        let s = Storage::open_in_memory().unwrap();
        let mut a = conn("a");
        a.group = "GroupA".into();
        let mut b = conn("b");
        b.group = "GroupB".into();
        let mut c = conn("c");
        c.group = "GroupA".into();
        s.save(&a).unwrap();
        s.save(&b).unwrap();
        s.save(&c).unwrap();

        let groups = s.list_groups().unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&"GroupA".to_string()));
        assert!(groups.contains(&"GroupB".to_string()));
    }

    #[test]
    fn list_groups_empty_when_no_connections() {
        let s = Storage::open_in_memory().unwrap();
        assert!(s.list_groups().unwrap().is_empty());
    }

    #[test]
    fn save_connection_with_key_file_ssh_roundtrip() {
        use crate::core::ssh::model::{SshAuth, SshConfig};
        let s = Storage::open_in_memory().unwrap();
        let mut c = conn("with-ssh");
        c.ssh = Some(SshConfig {
            host: "bastion.example.com".into(),
            port: 22,
            username: "admin".into(),
            auth: SshAuth::KeyFile("/home/user/.ssh/id_ed25519".into()),
        });
        s.save(&c).unwrap();

        let loaded = s.load_all().unwrap();
        let ssh = loaded[0].ssh.as_ref().expect("ssh should be present");
        assert_eq!(ssh.host, "bastion.example.com");
        assert_eq!(ssh.port, 22);
        assert_eq!(ssh.username, "admin");
        assert!(
            matches!(&ssh.auth, SshAuth::KeyFile(p) if p == "/home/user/.ssh/id_ed25519"),
            "key path should round-trip through JSON"
        );
    }

    #[test]
    fn save_connection_without_ssh_stores_null_json() {
        let s = Storage::open_in_memory().unwrap();
        let mut c = conn("no-ssh");
        c.ssh = None;
        s.save(&c).unwrap();

        let loaded = s.load_all().unwrap();
        assert!(loaded[0].ssh.is_none());
    }
}
