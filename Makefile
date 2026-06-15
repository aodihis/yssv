# YSSV — cross-platform Makefile
# Requires: GNU make, Rust toolchain (cargo, rustfmt, clippy)
# Windows: install make via `winget install GnuWin32.Make` or scoop/choco

ifeq ($(OS),Windows_NT)
    BINARY  = target\release\yssv.exe
    RUN_BIN = target\release\yssv.exe
else
    BINARY  = target/release/yssv
    RUN_BIN = ./target/release/yssv
endif

.PHONY: all build release run run-release run-log watch watch-log test-all test-lib test-integration \
        test-e2e lint fmt fmt-check check clean help

# ── Default ────────────────────────────────────────────────────────────────────
all: build

# ── Build ──────────────────────────────────────────────────────────────────────
build:
	cargo build

release:
	cargo build --release

# ── Run ────────────────────────────────────────────────────────────────────────
run:
	cargo run

run-release: release
	$(RUN_BIN)

# ── Watch ──────────────────────────────────────────────────────────────────────
# Requires: cargo install cargo-watch
watch:
	cargo watch -x run

# Watch with a specific log level: make watch-log LEVEL=debug
ifeq ($(OS),Windows_NT)
watch-log:
	powershell -NoProfile -Command "[Environment]::SetEnvironmentVariable('YSSV_LOG','$(LEVEL)','Process'); cargo watch -x run"
else
watch-log:
	YSSV_LOG=$(LEVEL) cargo watch -x run
endif

# Run with a specific log level: make run-log LEVEL=debug
# Uses cmd /C on Windows so the env var is scoped to that command.
ifeq ($(OS),Windows_NT)
run-log:
	powershell -NoProfile -Command "[Environment]::SetEnvironmentVariable('YSSV_LOG','$(LEVEL)','Process'); cargo run"
else
run-log:
	YSSV_LOG=$(LEVEL) cargo run
endif

# ── Test ───────────────────────────────────────────────────────────────────────
# All tests including #[ignore] — requires live DB servers and Docker.
# Set YSSV_TEST_PG_URL and/or YSSV_TEST_MYSQL_URL first.
test-all:
	cargo test -- --include-ignored

# Unit tests only — fastest, no DB required
test-lib:
	cargo test --lib

# Integration suite only (tests/integration/)
test-integration:
	cargo test --test integration

# E2E suite only (tests/e2e/) — requires Docker
test-e2e:
	cargo test --test e2e --features e2e

# ── Code quality ───────────────────────────────────────────────────────────────
check:
	cargo check

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

fix:
	cargo fmt && cargo clippy --fix --allow-dirty


# ── Clean ──────────────────────────────────────────────────────────────────────
clean:
	cargo clean

# ── Help ───────────────────────────────────────────────────────────────────────
help:
	@echo YSSV - available targets:
	@echo.
	@echo   build                 Debug build
	@echo   release               Optimised release build
	@echo   run                   Run in development mode (debug log to stderr + file)
	@echo   run-release           Build release then run it
	@echo   run-log LEVEL=X       Run with YSSV_LOG=X  (e.g. make run-log LEVEL=debug)
	@echo   watch                 Auto-rebuild and rerun on file changes (requires cargo-watch)
	@echo   watch-log LEVEL=X     Watch with YSSV_LOG=X  (e.g. make watch-log LEVEL=debug)
	@echo.
	@echo   test-all              All tests including #[ignore] (needs live DB + Docker)
	@echo   test-lib              Unit tests only - no database required
	@echo   test-integration      Integration suite only (tests/integration/)
	@echo   test-e2e              E2E suite only (tests/e2e/) - requires Docker
	@echo.
	@echo   check                 Type-check without building
	@echo   lint                  Clippy with -D warnings
	@echo   fmt                   Auto-format with rustfmt
	@echo   fmt-check             Check formatting (CI use)
	@echo.
	@echo   clean                 Remove build artifacts
	@echo   help                  Show this message
	@echo.
	@echo Environment variables:
	@echo   YSSV_LOG              Log filter (default: debug in dev, info in release)
	@echo   YSSV_LOG_KEEP_DAYS    Days of log files to keep (default: 7)
	@echo   YSSV_RENDERER         Renderer: wgpu (default) or glow (OpenGL fallback)
	@echo   YSSV_TEST_PG_URL      PostgreSQL URL for integration tests
	@echo   YSSV_TEST_MYSQL_URL   MySQL URL for integration tests
