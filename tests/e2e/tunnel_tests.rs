use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};
use testcontainers::core::{IntoContainerPort, WaitFor};
use tokio::io::AsyncReadExt;
use yssv::core::ssh::{
    SshAuth, SshConfig, SshTunnel, TunnelStatus,
};

async fn start_ssh_container() -> (ContainerAsync<GenericImage>, u16) {
    let container = GenericImage::new("linuxserver/openssh-server", "latest")
        .with_exposed_port(2222.tcp())
        .with_wait_for(WaitFor::seconds(5))
        .with_env_var("USER_NAME", "testuser")
        .with_env_var("USER_PASSWORD", "testpass")
        .with_env_var("PASSWORD_ACCESS", "true")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(2222).await.unwrap();
    (container, port)
}

fn ssh_config(port: u16) -> SshConfig {
    SshConfig {
        host: "127.0.0.1".into(),
        port,
        username: "testuser".into(),
        auth: SshAuth::Password("testpass".into()),
    }
}

#[tokio::test]
async fn ssh_tunnel_opens_with_password_auth() {
    let (_container, port) = start_ssh_container().await;
    let tunnel = SshTunnel::open(&ssh_config(port), "127.0.0.1", 2222)
        .await
        .expect("tunnel should open with valid credentials");
    assert!(tunnel.local_port() > 0);
}

#[tokio::test]
async fn ssh_tunnel_local_port_is_nonzero() {
    let (_container, port) = start_ssh_container().await;
    let tunnel = SshTunnel::open(&ssh_config(port), "127.0.0.1", 2222)
        .await
        .unwrap();
    assert!(tunnel.local_port() > 0, "local_port should be assigned a non-zero OS port");
}

#[tokio::test]
async fn ssh_tunnel_status_is_connected_after_open() {
    let (_container, port) = start_ssh_container().await;
    let tunnel = SshTunnel::open(&ssh_config(port), "127.0.0.1", 2222)
        .await
        .unwrap();
    let status = tunnel.status_handle().lock().unwrap().clone();
    assert_eq!(status, TunnelStatus::Connected);
}

#[tokio::test]
async fn ssh_tunnel_bad_password_returns_error() {
    let (_container, port) = start_ssh_container().await;
    let ssh = SshConfig {
        host: "127.0.0.1".into(),
        port,
        username: "testuser".into(),
        auth: SshAuth::Password("wrongpassword".into()),
    };
    let err = SshTunnel::open(&ssh, "127.0.0.1", 2222).await.unwrap_err();
    assert!(!err.message.is_empty(), "error message should not be empty");
}

#[tokio::test]
async fn ssh_tunnel_agent_auth_returns_unsupported_error() {
    let (_container, port) = start_ssh_container().await;
    let ssh = SshConfig {
        host: "127.0.0.1".into(),
        port,
        username: "testuser".into(),
        auth: SshAuth::Agent,
    };
    let err = SshTunnel::open(&ssh, "127.0.0.1", 2222).await.unwrap_err();
    assert!(
        err.message.contains("not yet supported"),
        "expected unsupported-agent message, got: {}",
        err.message
    );
}

#[tokio::test]
async fn ssh_tunnel_forwards_tcp_connection() {
    let (_container, port) = start_ssh_container().await;
    // Forward tunnel → the container's own SSH daemon (127.0.0.1:2222 from the container's view).
    // Connecting through the tunnel and reading 4 bytes should yield the SSH banner "SSH-".
    let tunnel = SshTunnel::open(&ssh_config(port), "127.0.0.1", 2222)
        .await
        .unwrap();

    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", tunnel.local_port()))
        .await
        .expect("should connect to local tunnel port");

    let mut buf = [0u8; 4];
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        stream.read_exact(&mut buf),
    )
    .await
    .expect("read should not time out")
    .expect("read should succeed");

    assert_eq!(&buf, b"SSH-", "expected SSH banner through the forwarded tunnel");
}
