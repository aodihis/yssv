use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SshAuth {
    Password(String),
    KeyFile(String),
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: String::new(),
            auth: SshAuth::Password(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_config_serialization_roundtrip() {
        let cfg = SshConfig {
            host: "bastion.example.com".into(),
            port: 22,
            username: "admin".into(),
            auth: SshAuth::KeyFile("/home/user/.ssh/id_rsa".into()),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: SshConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, decoded);
    }

    #[test]
    fn ssh_default_port_is_22() {
        let cfg = SshConfig::default();
        assert_eq!(cfg.port, 22);
    }
}
