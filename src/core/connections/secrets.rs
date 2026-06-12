use keyring_core::{Entry, Error};

const SERVICE: &str = "yssv";

fn entry(account: &str) -> keyring_core::Result<Entry> {
    Entry::new(SERVICE, account)
}

fn do_save(conn_id: &str, account: &str, label: &str, password: &str) {
    if password.is_empty() {
        do_delete(conn_id, account, label);
        return;
    }
    match entry(account).and_then(|e| e.set_password(password)) {
        Ok(()) => tracing::debug!(conn_id, "secrets: saved {label} password"),
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to save {label} password"),
    }
}

fn do_load(conn_id: &str, account: &str, label: &str) -> String {
    match entry(account).and_then(|e| e.get_password()) {
        Ok(p) => p,
        Err(Error::NoEntry) => String::new(),
        Err(e) => {
            tracing::warn!(conn_id, error = %e, "secrets: failed to load {label} password");
            String::new()
        }
    }
}

fn do_delete(conn_id: &str, account: &str, label: &str) {
    match entry(account).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(Error::NoEntry) => {}
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to delete {label} password"),
    }
}

pub fn save_db_password(conn_id: &str, password: &str) {
    do_save(conn_id, conn_id, "db", password);
}

pub fn load_db_password(conn_id: &str) -> String {
    do_load(conn_id, conn_id, "db")
}

pub fn delete_db_password(conn_id: &str) {
    do_delete(conn_id, conn_id, "db");
}

pub fn save_ssh_password(conn_id: &str, password: &str) {
    let account = format!("{conn_id}:ssh");
    do_save(conn_id, &account, "ssh", password);
}

pub fn load_ssh_password(conn_id: &str) -> String {
    let account = format!("{conn_id}:ssh");
    do_load(conn_id, &account, "ssh")
}

pub fn delete_ssh_password(conn_id: &str) {
    let account = format!("{conn_id}:ssh");
    do_delete(conn_id, &account, "ssh");
}

pub fn delete_all(conn_id: &str) {
    delete_db_password(conn_id);
    delete_ssh_password(conn_id);
}
