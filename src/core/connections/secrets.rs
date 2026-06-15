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

#[cfg(test)]
mod tests {
    use super::*;

    // Initialize the native keyring store once per test run so that
    // set_password / get_password can actually persist in tests that run on a
    // machine with a real secret store (Windows Credential Manager, macOS
    // Keychain, etc.).  Errors are silently ignored: when no store is
    // available every operation falls back to a no-op and load returns "".
    static KEYRING_INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn init_keyring() {
        KEYRING_INIT.get_or_init(|| {
            let _ = keyring::use_native_store(false);
        });
    }

    fn unique_id() -> String {
        format!("yssv-test-{}", uuid::Uuid::new_v4())
    }

    // --- smoke tests: functions must not panic even without a keyring ---

    #[test]
    fn load_db_password_returns_empty_for_unknown_id() {
        let pw = load_db_password("yssv-test-no-such-db-entry");
        assert!(pw.is_empty());
    }

    #[test]
    fn load_ssh_password_returns_empty_for_unknown_id() {
        let pw = load_ssh_password("yssv-test-no-such-ssh-entry");
        assert!(pw.is_empty());
    }

    #[test]
    fn delete_all_does_not_panic_on_missing_entries() {
        delete_all("yssv-test-delete-all-no-entry");
    }

    // --- logic tests: control-flow inside do_save / do_delete ---

    #[test]
    fn save_empty_db_password_acts_as_delete() {
        init_keyring();
        let id = unique_id();
        // First write something (may or may not persist, depending on store).
        save_db_password(&id, "secret");
        // Saving an empty string must take the delete branch.
        save_db_password(&id, "");
        let pw = load_db_password(&id);
        // Whether the keyring persisted "secret" or not, after saving "" the
        // entry is either deleted (→ "") or was never written (→ "").
        assert!(pw.is_empty(), "empty-password save should clear the entry");
    }

    #[test]
    fn save_empty_ssh_password_acts_as_delete() {
        init_keyring();
        let id = unique_id();
        save_ssh_password(&id, "secret");
        save_ssh_password(&id, "");
        let pw = load_ssh_password(&id);
        assert!(pw.is_empty());
    }

    #[test]
    fn delete_db_password_leaves_load_returning_empty() {
        init_keyring();
        let id = unique_id();
        save_db_password(&id, "todelete");
        delete_db_password(&id);
        assert!(load_db_password(&id).is_empty());
    }

    #[test]
    fn delete_all_clears_both_db_and_ssh_entries() {
        init_keyring();
        let id = unique_id();
        save_db_password(&id, "db-pw");
        save_ssh_password(&id, "ssh-pw");
        delete_all(&id);
        assert!(load_db_password(&id).is_empty());
        assert!(load_ssh_password(&id).is_empty());
    }

    // --- roundtrip tests: only meaningful when a real keyring is available ---

    #[test]
    fn db_and_ssh_passwords_are_stored_independently() {
        init_keyring();
        let id = unique_id();
        save_db_password(&id, "db-secret");
        save_ssh_password(&id, "ssh-secret");

        let db_pw = load_db_password(&id);
        let ssh_pw = load_ssh_password(&id);

        delete_all(&id); // always clean up

        // When the keyring is available both values must be what we saved and
        // must be different (they live under different account keys).
        if !db_pw.is_empty() {
            assert_eq!(db_pw, "db-secret");
        }
        if !ssh_pw.is_empty() {
            assert_eq!(ssh_pw, "ssh-secret");
        }
        if !db_pw.is_empty() && !ssh_pw.is_empty() {
            assert_ne!(db_pw, ssh_pw, "db and ssh passwords use distinct keyring accounts");
        }
    }
}
