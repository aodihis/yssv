use yssv::core::connections::{
    model::{ConnColor, Connection, DbEngine},
    storage::Storage,
};
use yssv::core::ssh::model::{SshAuth, SshConfig};

fn make_conn(name: &str, group: &str) -> Connection {
    let mut c = Connection::new_postgres();
    c.name = name.into();
    c.group = group.into();
    c.database = "testdb".into();
    c.username = "postgres".into();
    c
}

#[test]
fn crud_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let c = make_conn("Test Conn", "Local");
    s.save(&c).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "Test Conn");
    assert_eq!(all[0].engine, DbEngine::Postgres);
}

#[test]
fn persist_and_load_multiple_connections() {
    let s = Storage::open_in_memory().unwrap();
    let c1 = make_conn("prod-pg", "Production");
    let c2 = make_conn("local-pg", "Local");
    let c3 = {
        let mut c = Connection::new_mysql();
        c.name = "local-mysql".into();
        c.group = "Local".into();
        c
    };
    s.save(&c1).unwrap();
    s.save(&c2).unwrap();
    s.save(&c3).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(all.len(), 3);
    let names: Vec<&str> = all.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"prod-pg"));
    assert!(names.contains(&"local-pg"));
    assert!(names.contains(&"local-mysql"));
}

#[test]
fn save_updates_existing() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("Original", "Local");
    s.save(&c).unwrap();
    c.name = "Updated".into();
    s.save(&c).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "Updated");
}

#[test]
fn delete_removes_record() {
    let s = Storage::open_in_memory().unwrap();
    let c = make_conn("To Delete", "Local");
    s.save(&c).unwrap();
    s.delete(&c.id).unwrap();
    assert!(s.load_all().unwrap().is_empty());
}

#[test]
fn delete_nonexistent_is_ok() {
    let s = Storage::open_in_memory().unwrap();
    assert!(s.delete("nonexistent-id").is_ok());
}

#[test]
fn groups_list_deduplicates() {
    let s = Storage::open_in_memory().unwrap();
    s.save(&make_conn("A", "Production")).unwrap();
    s.save(&make_conn("B", "Local")).unwrap();
    s.save(&make_conn("C", "Local")).unwrap();
    s.save(&make_conn("D", "Staging")).unwrap();
    let groups = s.list_groups().unwrap();
    assert_eq!(groups.len(), 3);
    assert!(groups.contains(&"Local".to_string()));
    assert!(groups.contains(&"Production".to_string()));
    assert!(groups.contains(&"Staging".to_string()));
}

#[test]
fn ssh_config_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("ssh-conn", "Production");
    c.ssh = Some(SshConfig {
        host: "bastion.example.com".into(),
        port: 22,
        username: "deploy".into(),
        auth: SshAuth::KeyFile("/home/deploy/.ssh/id_rsa".into()),
    });
    s.save(&c).unwrap();
    let all = s.load_all().unwrap();
    let loaded = &all[0];
    assert!(loaded.ssh.is_some());
    let ssh = loaded.ssh.as_ref().unwrap();
    assert_eq!(ssh.host, "bastion.example.com");
    assert_eq!(ssh.username, "deploy");
    assert!(matches!(ssh.auth, SshAuth::KeyFile(_)));
}

#[test]
fn ssh_password_auth_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("ssh-pw", "Local");
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
fn favorite_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("fav", "Local");
    c.is_favorite = true;
    s.save(&c).unwrap();
    assert!(s.load_all().unwrap()[0].is_favorite);
}

#[test]
fn color_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("col", "Local");
    c.color = ConnColor::Purple;
    s.save(&c).unwrap();
    assert_eq!(s.load_all().unwrap()[0].color, ConnColor::Purple);
}
