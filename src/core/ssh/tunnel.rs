use std::sync::{Arc, Mutex};
use std::time::Duration;

use russh::client::{self, Handle, Msg};
use russh::keys::{PrivateKeyWithHashAlg, load_secret_key};
use russh::{Channel, ChannelStream};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::core::drivers::DbError;
use crate::core::ssh::model::{SshAuth, SshConfig};

const RECONNECT_MAX_ATTEMPTS: u32 = 5;

/// Shared, pollable tunnel status: the background accept loop writes it, the UI reads it.
pub type TunnelStatusHandle = Arc<Mutex<TunnelStatus>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelStatus {
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

impl TunnelStatus {
    /// Short label for the tunnel status (used for the UI footer dot + text).
    pub fn label(&self) -> &str {
        match self {
            TunnelStatus::Connecting => "SSH connecting…",
            TunnelStatus::Connected => "SSH tunnel",
            TunnelStatus::Reconnecting => "SSH reconnecting…",
            TunnelStatus::Failed(_) => "SSH failed",
        }
    }
}

/// A live SSH tunnel forwarding `127.0.0.1:local_port` → `remote_host:remote_port`
/// through the configured bastion. The background accept loop is aborted on drop.
pub struct SshTunnel {
    local_port: u16,
    status: TunnelStatusHandle,
    task: JoinHandle<()>,
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        tracing::debug!(
            local_port = self.local_port,
            "ssh tunnel: dropping, aborting accept loop"
        );
        self.task.abort();
    }
}

impl SshTunnel {
    pub async fn open(
        ssh: &SshConfig,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<SshTunnel, DbError> {
        tracing::debug!(
            ssh_host = %ssh.host, ssh_port = ssh.port,
            remote_host = %remote_host, remote_port,
            "ssh tunnel: opening"
        );

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| DbError::new(format!("SSH tunnel: failed to bind local port: {e}")))?;
        let local_port = listener
            .local_addr()
            .map_err(|e| DbError::new(format!("SSH tunnel: failed to read local port: {e}")))?
            .port();

        let status = Arc::new(Mutex::new(TunnelStatus::Connecting));

        // Validate credentials up front so connection failures surface immediately
        // rather than on the first forwarded byte.
        let session = connect_ssh(ssh).await?;
        set_status(&status, TunnelStatus::Connected);
        tracing::info!(
            ssh_host = %ssh.host, local_port,
            remote_host = %remote_host, remote_port,
            "ssh tunnel: established"
        );

        let ssh_cfg = ssh.clone();
        let remote_host = remote_host.to_string();
        let task_status = status.clone();
        let task = tokio::spawn(async move {
            run_accept_loop(
                listener,
                session,
                ssh_cfg,
                remote_host,
                remote_port,
                task_status,
            )
            .await;
        });

        Ok(SshTunnel {
            local_port,
            status,
            task,
        })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn status_handle(&self) -> TunnelStatusHandle {
        self.status.clone()
    }
}

async fn connect_ssh(ssh: &SshConfig) -> Result<Handle<ClientHandler>, DbError> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(config, (ssh.host.as_str(), ssh.port), ClientHandler)
        .await
        .map_err(|e| {
            DbError::new(format!(
                "SSH connect to {}:{} failed: {e}",
                ssh.host, ssh.port
            ))
        })?;

    let result = match &ssh.auth {
        SshAuth::Password(password) => handle
            .authenticate_password(&ssh.username, password)
            .await
            .map_err(|e| DbError::new(format!("SSH password authentication failed: {e}")))?,
        SshAuth::KeyFile(path) => {
            let key = load_secret_key(path, None)
                .map_err(|e| DbError::new(format!("SSH key file load failed ({path}): {e}")))?;
            // RSA hash negotiation only matters for RSA keys; skip the round-trip otherwise.
            let hash = if key.algorithm().is_rsa() {
                handle
                    .best_supported_rsa_hash()
                    .await
                    .ok()
                    .flatten()
                    .flatten()
            } else {
                None
            };
            handle
                .authenticate_publickey(
                    &ssh.username,
                    PrivateKeyWithHashAlg::new(Arc::new(key), hash),
                )
                .await
                .map_err(|e| DbError::new(format!("SSH key authentication failed: {e}")))?
        }
        SshAuth::Agent => {
            return Err(DbError::new(
                "SSH agent authentication is not yet supported; use password or key file.",
            ));
        }
    };

    if !result.success() {
        return Err(DbError::new(
            "SSH authentication failed: server rejected credentials.",
        ));
    }
    tracing::debug!(user = %ssh.username, host = %ssh.host, "ssh tunnel: authenticated");
    Ok(handle)
}

async fn open_channel(
    session: &Handle<ClientHandler>,
    remote_host: &str,
    remote_port: u16,
) -> Result<Channel<Msg>, DbError> {
    session
        .channel_open_direct_tcpip(remote_host.to_string(), remote_port as u32, "127.0.0.1", 0)
        .await
        .map_err(|e| DbError::new(format!("SSH tunnel: open channel failed: {e}")))
}

async fn reconnect(
    ssh: &SshConfig,
    status: &TunnelStatusHandle,
) -> Result<Handle<ClientHandler>, DbError> {
    set_status(status, TunnelStatus::Reconnecting);
    for attempt in 1..=RECONNECT_MAX_ATTEMPTS {
        tracing::info!(attempt, host = %ssh.host, "ssh tunnel: reconnect attempt");
        match connect_ssh(ssh).await {
            Ok(handle) => {
                tracing::info!(attempt, host = %ssh.host, "ssh tunnel: reconnected");
                return Ok(handle);
            }
            Err(e) => {
                tracing::warn!(attempt, error = %e.message, "ssh tunnel: reconnect attempt failed");
                tokio::time::sleep(Duration::from_secs(2 * attempt as u64)).await;
            }
        }
    }
    Err(DbError::new(format!(
        "SSH tunnel: reconnect failed after {RECONNECT_MAX_ATTEMPTS} attempts"
    )))
}

async fn run_accept_loop(
    listener: TcpListener,
    mut session: Handle<ClientHandler>,
    ssh: SshConfig,
    remote_host: String,
    remote_port: u16,
    status: TunnelStatusHandle,
) {
    loop {
        let (mut inbound, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "ssh tunnel: accept failed");
                continue;
            }
        };
        tracing::debug!(peer = %peer, "ssh tunnel: inbound connection accepted");

        let channel = match open_channel(&session, &remote_host, remote_port).await {
            Ok(channel) => Some(channel),
            Err(_) => {
                tracing::warn!("ssh tunnel: session lost, attempting reconnect");
                match reconnect(&ssh, &status).await {
                    Ok(new_session) => {
                        session = new_session;
                        set_status(&status, TunnelStatus::Connected);
                        open_channel(&session, &remote_host, remote_port).await.ok()
                    }
                    Err(e) => {
                        tracing::error!(error = %e.message, "ssh tunnel: reconnect failed, dropping connection");
                        set_status(&status, TunnelStatus::Failed(e.message));
                        None
                    }
                }
            }
        };

        let Some(channel) = channel else {
            continue;
        };

        let host = remote_host.clone();
        tokio::spawn(async move {
            let mut stream: ChannelStream<Msg> = channel.into_stream();
            match tokio::io::copy_bidirectional(&mut inbound, &mut stream).await {
                Ok((to_remote, to_local)) => tracing::debug!(
                    to_remote, to_local, host = %host,
                    "ssh tunnel: forwarded stream closed"
                ),
                Err(e) => tracing::debug!(error = %e, host = %host, "ssh tunnel: stream error"),
            }
        });
    }
}

fn set_status(status: &TunnelStatusHandle, new: TunnelStatus) {
    if let Ok(mut s) = status.lock() {
        *s = new;
    }
}

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels() {
        assert_eq!(TunnelStatus::Connecting.label(), "SSH connecting…");
        assert_eq!(TunnelStatus::Connected.label(), "SSH tunnel");
        assert_eq!(TunnelStatus::Reconnecting.label(), "SSH reconnecting…");
        assert_eq!(TunnelStatus::Failed("x".into()).label(), "SSH failed");
    }

    #[test]
    fn set_status_updates_shared_state() {
        let status = Arc::new(Mutex::new(TunnelStatus::Connecting));
        set_status(&status, TunnelStatus::Connected);
        assert_eq!(*status.lock().unwrap(), TunnelStatus::Connected);
    }
}
