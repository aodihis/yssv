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

/// Delete log files older than `keep_days` days.
/// Only touches files whose names start with "yssv.log".
fn cleanup_old_logs(dir: &str, keep_days: u64) {
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(keep_days * 86_400))
        .unwrap_or(std::time::UNIX_EPOCH);

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_log = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("yssv.log"))
            .unwrap_or(false);
        if !is_log {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if modified < cutoff {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

/// Initialize structured logging.
///
/// ## Environment variables
///
/// | Variable            | Default      | Description                                  |
/// |---------------------|--------------|----------------------------------------------|
/// | `YSSV_LOG`          | `info`/`debug` | Log filter. Supports level and per-crate directives (see below). |
/// | `YSSV_LOG_KEEP_DAYS`| `7`          | How many days of rolling log files to keep.  |
///
/// ## Log levels (lowest → highest severity)
///
/// `trace` → `debug` → `info` → `warn` → `error`
///
/// Setting a level includes all levels above it. Examples:
/// ```
/// YSSV_LOG=debug                     # everything at debug and above
/// YSSV_LOG=trace                     # everything, very verbose
/// YSSV_LOG=yssv=debug,sqlx=warn      # app at debug, sqlx quiet
/// YSSV_LOG=yssv::core::drivers=trace # single module at trace
/// ```
fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let dir = log_dir();
    let _ = std::fs::create_dir_all(&dir);

    // Retention: YSSV_LOG_KEEP_DAYS (default 7).
    let keep_days = std::env::var("YSSV_LOG_KEEP_DAYS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(7);
    cleanup_old_logs(&dir, keep_days);

    let file_appender = tracing_appender::rolling::daily(&dir, "yssv.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Default level: DEBUG in debug builds, INFO in release.
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let env_filter =
        EnvFilter::try_from_env("YSSV_LOG").unwrap_or_else(|_| EnvFilter::new(default_level));

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

fn pick_renderer(settings: &yssv::pages::settings::SettingsState) -> eframe::Renderer {
    use yssv::pages::settings::RendererPreference;

    // Priority: --renderer CLI flag > YSSV_RENDERER env var > settings file > wgpu default.
    let from_args = std::env::args().skip_while(|a| a != "--renderer").nth(1);
    let from_env = std::env::var("YSSV_RENDERER").ok();

    match from_args.as_deref().or(from_env.as_deref()) {
        Some("glow") => eframe::Renderer::Glow,
        Some("wgpu") => eframe::Renderer::Wgpu,
        Some(other) => {
            tracing::warn!(
                value = other,
                "unknown --renderer value, falling back to settings"
            );
            match settings.renderer {
                RendererPreference::Glow => eframe::Renderer::Glow,
                RendererPreference::Wgpu => eframe::Renderer::Wgpu,
            }
        }
        None => match settings.renderer {
            RendererPreference::Glow => eframe::Renderer::Glow,
            RendererPreference::Wgpu => eframe::Renderer::Wgpu,
        },
    }
}

fn native_options(renderer: eframe::Renderer) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("YSSV")
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([900.0, 600.0]),
        renderer,
        ..Default::default()
    }
}

fn main() -> eframe::Result<()> {
    let _log_guard = init_logging();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "YSSV starting");

    let rt = std::sync::Arc::new(
        tokio::runtime::Runtime::new().expect("failed to create tokio runtime"),
    );

    let settings = yssv::pages::settings::SettingsState::load();
    tracing::info!(
        theme = ?settings.theme,
        renderer = ?settings.renderer,
        log_keep_days = settings.log_keep_days,
        "loaded settings"
    );

    let renderer = pick_renderer(&settings);
    tracing::info!(?renderer, "selected renderer");

    let rt1 = rt.clone();
    let result = eframe::run_native(
        "YSSV",
        native_options(renderer),
        Box::new(move |cc| Ok(Box::new(yssv::app::YssvApp::new(cc, rt1)))),
    );

    // If the chosen renderer failed (common with wgpu on some GPU drivers),
    // automatically retry with glow so the user sees the app instead of nothing.
    if result.is_err() && renderer != eframe::Renderer::Glow {
        tracing::warn!(error = ?result, "renderer failed, retrying with glow");
        eframe::run_native(
            "YSSV",
            native_options(eframe::Renderer::Glow),
            Box::new(move |cc| Ok(Box::new(yssv::app::YssvApp::new(cc, rt)))),
        )
    } else {
        result
    }
}
