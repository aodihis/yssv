use keyring::Entry;

const SERVICE: &str = "yssv";

fn entry(account: &str) -> keyring::Result<Entry> {
    Entry::new(SERVICE, account)
}

pub fn save_db_password(conn_id: &str, password: &str) {
    if password.is_empty() {
        delete_db_password(conn_id);
        return;
    }
    match entry(conn_id).and_then(|e| e.set_password(password)) {
        Ok(()) => tracing::debug!(conn_id, "secrets: saved db password"),
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to save db password"),
    }
}

pub fn load_db_password(conn_id: &str) -> String {
    match entry(conn_id).and_then(|e| e.get_password()) {
        Ok(p) => p,
        Err(keyring::Error::NoEntry) => String::new(),
        Err(e) => {
            tracing::warn!(conn_id, error = %e, "secrets: failed to load db password");
            String::new()
        }
    }
}

pub fn delete_db_password(conn_id: &str) {
    match entry(conn_id).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to delete db password"),
    }
}

pub fn save_ssh_password(conn_id: &str, password: &str) {
    let account = format!("{conn_id}:ssh");
    if password.is_empty() {
        delete_ssh_password(conn_id);
        return;
    }
    match entry(&account).and_then(|e| e.set_password(password)) {
        Ok(()) => tracing::debug!(conn_id, "secrets: saved ssh password"),
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to save ssh password"),
    }
}

pub fn load_ssh_password(conn_id: &str) -> String {
    let account = format!("{conn_id}:ssh");
    match entry(&account).and_then(|e| e.get_password()) {
        Ok(p) => p,
        Err(keyring::Error::NoEntry) => String::new(),
        Err(e) => {
            tracing::warn!(conn_id, error = %e, "secrets: failed to load ssh password");
            String::new()
        }
    }
}

pub fn delete_ssh_password(conn_id: &str) {
    let account = format!("{conn_id}:ssh");
    match entry(&account).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => tracing::warn!(conn_id, error = %e, "secrets: failed to delete ssh password"),
    }
}

pub fn delete_all(conn_id: &str) {
    delete_db_password(conn_id);
    delete_ssh_password(conn_id);
}
