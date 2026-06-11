use crate::core::connections::model::{ConnColor, Connection, DbEngine};
use crate::core::ssh::model::{SshAuth, SshConfig};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SshAuthMethod {
    #[default]
    Password,
    KeyFile,
    Agent,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TestStatus {
    #[default]
    Idle,
    Testing,
    Ok,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SaveStatus {
    #[default]
    Idle,
    Saved,
}

/// Mirrors `Connection` with owned `String` fields for the form.
#[derive(Debug, Clone)]
pub struct ConnectionForm {
    pub id: String,
    pub name: String,
    pub group: String,
    pub engine: DbEngine,
    pub color: ConnColor,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssh_enabled: bool,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_username: String,
    pub ssh_auth_method: SshAuthMethod,
    pub ssh_password: String,
    pub ssh_key_path: String,
    pub is_favorite: bool,
}

impl Default for ConnectionForm {
    fn default() -> Self {
        let c = Connection::new_postgres();
        Self::from_connection(&c)
    }
}

impl ConnectionForm {
    pub fn from_connection(c: &Connection) -> Self {
        let (
            ssh_enabled,
            ssh_host,
            ssh_port,
            ssh_username,
            ssh_auth_method,
            ssh_password,
            ssh_key_path,
        ) = if let Some(ssh) = &c.ssh {
            let (method, pass, key) = match &ssh.auth {
                SshAuth::Password(p) => (SshAuthMethod::Password, p.clone(), String::new()),
                SshAuth::KeyFile(k) => (SshAuthMethod::KeyFile, String::new(), k.clone()),
                SshAuth::Agent => (SshAuthMethod::Agent, String::new(), String::new()),
            };
            (
                true,
                ssh.host.clone(),
                ssh.port.to_string(),
                ssh.username.clone(),
                method,
                pass,
                key,
            )
        } else {
            (
                false,
                String::new(),
                "22".into(),
                String::new(),
                SshAuthMethod::Password,
                String::new(),
                String::new(),
            )
        };

        Self {
            id: c.id.clone(),
            name: c.name.clone(),
            group: c.group.clone(),
            engine: c.engine,
            color: c.color,
            host: c.host.clone(),
            port: c.port.to_string(),
            database: c.database.clone(),
            username: c.username.clone(),
            password: c.password.clone(),
            ssh_enabled,
            ssh_host,
            ssh_port,
            ssh_username,
            ssh_auth_method,
            ssh_password,
            ssh_key_path,
            is_favorite: c.is_favorite,
        }
    }

    pub fn to_connection(&self) -> Connection {
        let port = self
            .port
            .parse::<u16>()
            .unwrap_or(self.engine.default_port());
        let ssh = if self.ssh_enabled {
            let ssh_port = self.ssh_port.parse::<u16>().unwrap_or(22);
            let auth = match self.ssh_auth_method {
                SshAuthMethod::Password => SshAuth::Password(self.ssh_password.clone()),
                SshAuthMethod::KeyFile => SshAuth::KeyFile(self.ssh_key_path.clone()),
                SshAuthMethod::Agent => SshAuth::Agent,
            };
            Some(SshConfig {
                host: self.ssh_host.clone(),
                port: ssh_port,
                username: self.ssh_username.clone(),
                auth,
            })
        } else {
            None
        };

        Connection {
            id: self.id.clone(),
            name: self.name.clone(),
            group: self.group.clone(),
            engine: self.engine,
            color: self.color,
            host: self.host.clone(),
            port,
            database: self.database.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            ssh,
            is_favorite: self.is_favorite,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.host.trim().is_empty()
    }
}

#[derive(Debug)]
pub struct ConnectionsPageState {
    pub connections: Vec<Connection>,
    pub selected_id: Option<String>,
    pub form: ConnectionForm,
    pub test_status: TestStatus,
    pub save_status: SaveStatus,
    pub search_query: String,
    pub collapsed_groups: std::collections::HashSet<String>,
    pub is_new: bool,
}

impl ConnectionsPageState {
    pub fn new(connections: Vec<Connection>) -> Self {
        let selected_id = connections.first().map(|c| c.id.clone());
        let form = selected_id
            .as_ref()
            .and_then(|id| connections.iter().find(|c| &c.id == id))
            .map(ConnectionForm::from_connection)
            .unwrap_or_default();

        let is_new = selected_id.is_none();
        Self {
            connections,
            selected_id,
            form,
            test_status: TestStatus::Idle,
            save_status: SaveStatus::Idle,
            search_query: String::new(),
            collapsed_groups: Default::default(),
            is_new,
        }
    }

    pub fn select(&mut self, id: &str) {
        self.selected_id = Some(id.to_string());
        if let Some(c) = self.connections.iter().find(|c| c.id == id) {
            self.form = ConnectionForm::from_connection(c);
        }
        self.test_status = TestStatus::Idle;
        self.save_status = SaveStatus::Idle;
        self.is_new = false;
    }

    pub fn start_new(&mut self) {
        let c = Connection::new_postgres();
        self.selected_id = Some(c.id.clone());
        self.form = ConnectionForm::from_connection(&c);
        self.test_status = TestStatus::Idle;
        self.save_status = SaveStatus::Idle;
        self.is_new = true;
    }

    pub fn reset_form(&mut self) {
        if let Some(id) = &self.selected_id.clone()
            && let Some(c) = self.connections.iter().find(|c| &c.id == id)
        {
            self.form = ConnectionForm::from_connection(c);
            self.test_status = TestStatus::Idle;
        }
    }

    /// Apply a saved connection to the list (insert or update).
    pub fn apply_saved(&mut self, c: Connection) {
        self.is_new = false;
        if let Some(pos) = self.connections.iter().position(|x| x.id == c.id) {
            self.connections[pos] = c.clone();
        } else {
            self.connections.push(c.clone());
        }
        self.select(&c.id);
    }

    /// Remove a connection from the list.
    pub fn remove(&mut self, id: &str) {
        self.connections.retain(|c| c.id != id);
        if self.selected_id.as_deref() == Some(id) {
            self.selected_id = self.connections.first().map(|c| c.id.clone());
            if let Some(ref sid) = self.selected_id.clone() {
                self.select(sid);
            }
        }
    }

    /// Groups with their connections (filtered by search_query).
    pub fn grouped_connections(&self) -> Vec<(String, Vec<&Connection>)> {
        let q = self.search_query.to_lowercase();
        let filtered: Vec<&Connection> = self
            .connections
            .iter()
            .filter(|c| {
                q.is_empty()
                    || c.name.to_lowercase().contains(&q)
                    || c.host.to_lowercase().contains(&q)
            })
            .collect();

        let groups: Vec<String> = filtered
            .iter()
            .map(|c| c.group.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        groups
            .into_iter()
            .map(|g| {
                let members: Vec<&Connection> =
                    filtered.iter().filter(|c| c.group == g).copied().collect();
                (g, members)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conn(name: &str, group: &str) -> Connection {
        let mut c = Connection::new_postgres();
        c.name = name.into();
        c.group = group.into();
        c
    }

    #[test]
    fn select_updates_form() {
        let c1 = make_conn("Alpha", "Local");
        let c2 = make_conn("Beta", "Local");
        let id2 = c2.id.clone();
        let mut state = ConnectionsPageState::new(vec![c1, c2]);
        state.select(&id2);
        assert_eq!(state.form.name, "Beta");
        assert_eq!(state.selected_id.as_deref(), Some(id2.as_str()));
    }

    #[test]
    fn start_new_creates_empty_form() {
        let mut state = ConnectionsPageState::new(vec![]);
        state.start_new();
        assert!(state.is_new);
        assert!(state.form.name.is_empty());
    }

    #[test]
    fn apply_saved_inserts_new_connection() {
        let mut state = ConnectionsPageState::new(vec![]);
        state.start_new();
        let mut c = state.form.to_connection();
        c.name = "New Conn".into();
        state.apply_saved(c);
        assert_eq!(state.connections.len(), 1);
        assert_eq!(state.connections[0].name, "New Conn");
    }

    #[test]
    fn apply_saved_updates_existing() {
        let c = make_conn("Original", "Local");
        let id = c.id.clone();
        let mut state = ConnectionsPageState::new(vec![c]);
        let mut updated = state.connections[0].clone();
        updated.name = "Updated".into();
        state.apply_saved(updated);
        assert_eq!(state.connections.len(), 1);
        assert_eq!(state.connections[0].name, "Updated");
    }

    #[test]
    fn remove_delects_and_selects_next() {
        let c1 = make_conn("A", "Local");
        let c2 = make_conn("B", "Local");
        let id1 = c1.id.clone();
        let mut state = ConnectionsPageState::new(vec![c1, c2]);
        state.remove(&id1);
        assert_eq!(state.connections.len(), 1);
        assert_eq!(state.connections[0].name, "B");
    }

    #[test]
    fn search_filters_by_name() {
        let c1 = make_conn("alpha", "Local");
        let c2 = make_conn("beta", "Local");
        let mut state = ConnectionsPageState::new(vec![c1, c2]);
        state.search_query = "alp".into();
        let groups = state.grouped_connections();
        assert_eq!(groups[0].1.len(), 1);
        assert_eq!(groups[0].1[0].name, "alpha");
    }

    #[test]
    fn grouped_connections_deduplicates_groups() {
        let c1 = make_conn("A", "Production");
        let c2 = make_conn("B", "Local");
        let c3 = make_conn("C", "Local");
        let state = ConnectionsPageState::new(vec![c1, c2, c3]);
        let groups = state.grouped_connections();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn form_to_connection_roundtrip() {
        let c = make_conn("Test", "Local");
        let form = ConnectionForm::from_connection(&c);
        let rebuilt = form.to_connection();
        assert_eq!(rebuilt.name, c.name);
        assert_eq!(rebuilt.engine, c.engine);
        assert_eq!(rebuilt.port, c.port);
    }

    #[test]
    fn form_is_valid_requires_name_and_host() {
        let mut form = ConnectionForm::default();
        assert!(!form.is_valid());
        form.name = "Test".into();
        assert!(form.is_valid()); // host already "127.0.0.1"
    }
}
