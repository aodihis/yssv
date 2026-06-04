use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn log_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        format!("{}\\yssv\\logs", base)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/.config/yssv/logs", home)
    }
}

/// Initialize structured logging.
///
/// Outputs:
/// - **Stderr**: human-readable, colored. Active in debug builds; in release
///   builds only when `YSSV_LOG` is set.
/// - **Rolling file**: `{data_dir}/yssv/logs/yssv.YYYY-MM-DD.log`. Always
///   active. No ANSI codes.
///
/// Override the log level at runtime:
/// ```
/// YSSV_LOG=trace cargo run
/// YSSV_LOG=yssv=debug,sqlx=warn cargo run
/// ```
fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let dir = log_dir();
    let _ = std::fs::create_dir_all(&dir);
    let file_appender = tracing_appender::rolling::daily(&dir, "yssv.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Default level: DEBUG in debug builds, INFO in release.
    let default_level = if cfg!(debug_assertions) { "debug" } else { "info" };
    let env_filter = EnvFilter::try_from_env("YSSV_LOG")
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    // File layer — compact, no color, always on.
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .compact();

    // Stderr layer — pretty, colored, only in debug OR when YSSV_LOG is set.
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(true)
        .pretty();

    if cfg!(debug_assertions) || std::env::var("YSSV_LOG").is_ok() {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stderr_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    guard
}

fn main() -> eframe::Result<()> {
    let _log_guard = init_logging();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "YSSV starting");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("YSSV")
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "YSSV",
        options,
        Box::new(|cc| Ok(Box::new(yssv::app::YssvApp::new(cc, rt)))),
    )
}
