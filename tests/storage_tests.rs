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
fn storage_persist_and_load_multiple_connections() {
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
fn storage_groups_list() {
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
fn storage_delete_nonexistent_is_ok() {
    let s = Storage::open_in_memory().unwrap();
    let result = s.delete("nonexistent-id");
    assert!(result.is_ok());
}

#[test]
fn storage_ssh_config_roundtrip() {
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
    matches!(ssh.auth, SshAuth::KeyFile(_));
}

#[test]
fn storage_favorite_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("fav", "Local");
    c.is_favorite = true;
    s.save(&c).unwrap();
    let loaded = &s.load_all().unwrap()[0];
    assert!(loaded.is_favorite);
}

#[test]
fn storage_color_roundtrip() {
    let s = Storage::open_in_memory().unwrap();
    let mut c = make_conn("col", "Local");
    c.color = ConnColor::Purple;
    s.save(&c).unwrap();
    let loaded = &s.load_all().unwrap()[0];
    assert_eq!(loaded.color, ConnColor::Purple);
}
