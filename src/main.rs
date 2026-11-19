use std::path::PathBuf;
use std::sync::Arc;

use app::web::AppState;
use opencode_token_dashboard as app;

fn help() {
    println!("opencode-token-dashboard — native Rust dashboard for OpenCode token usage");
    println!();
    println!("Usage: opencode-token-dashboard [options]");
    println!();
    println!("  (no flag)         Open the native GUI window (default)");
    println!("  --web             Run the local web server instead of the GUI");
    println!("  --host <address>  Web server bind address (default 127.0.0.1)");
    println!("  --port <port>     Web server port (default 3000, or PORT env var)");
    println!("  --db-path <path>  OpenCode database (default ~/.local/share/opencode/opencode.db)");
    println!("                    or OPENCODE_DB_PATH env var");
    println!("  -h, --help        Show this help");
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mut host = "127.0.0.1".to_string();
    let mut port: u16 = 3000;
    let mut db_path = app::usage::default_db_path();
    let mut web_mode = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                return;
            }
            "--web" => web_mode = true,
            "--host" => {
                if let Some(value) = args.next() {
                    host = value;
                }
            }
            "--port" => {
                if let Some(value) = args.next().and_then(|v| v.parse().ok()) {
                    port = value;
                }
            }
            "--db-path" => {
                if let Some(value) = args.next() {
                    db_path = PathBuf::from(value);
                }
            }
            other => {
                eprintln!("Unknown option: {other}");
                help();
                std::process::exit(2);
            }
        }
    }

    if let Ok(value) = std::env::var("PORT") {
        if let Ok(parsed) = value.parse() {
            port = parsed;
        }
    }

    if !web_mode {
        if let Err(error) = app::gui::run(db_path) {
            eprintln!("Could not start the GUI: {error}");
            std::process::exit(1);
        }
        return;
    }

    let refresh_seconds = std::env::var("OTD_AUTO_REFRESH_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(15);
    println!("Reading database: {}", db_path.display());
    let state = AppState {
        tera: Arc::new(app::web::templates()),
        db_path,
        refresh_seconds,
        pricing: app::settings::load_pricing_from(&app::settings::pricing_path()),
        config_path: app::settings::config_path(),
    };
    if host != "127.0.0.1" && host != "localhost" && host != "::1" {
        eprintln!("WARNING: binding to {host}. This API has no authentication; do not expose it on an untrusted network.");
    }
    let router = app::web::router(state);
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|error| panic!("Could not bind to {address}: {error}"));

    println!("Token usage web server listening on http://{address}");
    axum::serve(listener, router).await.expect("Server failed");
}
