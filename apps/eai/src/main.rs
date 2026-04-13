mod ai;
mod cli {
    pub mod cmd_add;
    pub mod cmd_config;
    pub mod cmd_explain;
    pub mod cmd_export;
    pub mod cmd_import;
    pub mod cmd_note;
    pub mod cmd_read;
    pub mod cmd_review;
    pub mod cmd_stats;
    pub mod cmd_sync;
}
mod config;
mod db;
mod error;
mod handlers {
    pub mod chat;
    pub mod notes;
    pub mod phrases;
    pub mod readings;
    pub mod reviews;
    pub mod stats;
    pub mod sync;
    pub mod words;
}
mod models;
mod prompt;
mod review;
mod server;
mod services;
mod state;
mod sync_db_adapter;

use clap::Parser;

#[derive(Parser)]
#[command(name = "engai", about = "AI English Learning System")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Add {
        #[command(subcommand)]
        target: cli::cmd_add::AddTarget,
    },
    Explain {
        #[command(subcommand)]
        target: cli::cmd_explain::ExplainTarget,
    },
    Review {
        #[arg(long)]
        all: bool,
    },
    Sync,
    Read {
        file: String,
    },
    Import {
        path: String,
    },
    Export {
        #[arg(long)]
        word: Option<String>,
        #[arg(long)]
        phrase: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Stats,
    Config {
        #[command(subcommand)]
        action: cli::cmd_config::ConfigAction,
    },
    Note {
        #[command(subcommand)]
        action: cli::cmd_note::NoteAction,
    },
    #[command(alias = "-s")]
    Server {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "engai=info");
    }
    tracing_subscriber::fmt::init();

    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { target }) => cli::cmd_add::run(target).await?,
        Some(Commands::Explain { target }) => cli::cmd_explain::run(target).await?,
        Some(Commands::Review { all }) => cli::cmd_review::run(all).await?,
        Some(Commands::Sync) => cli::cmd_sync::run().await?,
        Some(Commands::Read { file }) => cli::cmd_read::run(&file).await?,
        Some(Commands::Import { path }) => cli::cmd_import::run(&path).await?,
        Some(Commands::Export { word, phrase, all }) => {
            cli::cmd_export::run(word, phrase, all).await?
        }
        Some(Commands::Stats) => cli::cmd_stats::run().await?,
        Some(Commands::Config { action }) => cli::cmd_config::run(action).await?,
        Some(Commands::Note { action }) => cli::cmd_note::run(action).await?,
        Some(Commands::Server { port }) => {
            let config = crate::config::Config::load_global()?;
            let db_path = config.db_path();
            let db = crate::db::Db::new(&db_path).await?;
            let state = crate::state::AppState::new(std::sync::Arc::new(db), config);
            crate::server::run_server(state, port).await?;
        }
        None => {
            let config = crate::config::Config::load_global()?;
            let db_path = config.db_path();
            let db = crate::db::Db::new(&db_path).await?;
            let state = crate::state::AppState::new(std::sync::Arc::new(db), config.clone());

            let port = config.server.port;
            let server_url = format!("http://127.0.0.1:{}", port);
            
            let server_handle = tokio::spawn(async move {
                if let Err(e) = crate::server::run_server(state, port).await {
                    tracing::error!("Server error: {}", e);
                }
            });

            let tui_url = server_url.clone();
            let tui_handle = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                if let Err(e) = etui::run_tui(&tui_url).await {
                    tracing::error!("TUI error: {}", e);
                }
            });

            tokio::select! {
                _ = tui_handle => {
                    tracing::info!("TUI exited, shutting down server");
                }
                _ = server_handle => {
                    tracing::info!("Server exited, shutting down TUI");
                }
            }
        }
    }

    Ok(())
}
