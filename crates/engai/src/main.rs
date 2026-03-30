mod cmd_add;
mod cmd_config;
mod cmd_explain;
mod cmd_export;
mod cmd_import;
mod cmd_note;
mod cmd_read;
mod cmd_review;
mod cmd_stats;
mod cmd_sync;
mod error;
mod routes;
mod server;
mod state;
#[allow(dead_code)] mod tui;

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
        target: cmd_add::AddTarget,
    },
    Explain {
        #[command(subcommand)]
        target: cmd_explain::ExplainTarget,
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
        action: cmd_config::ConfigAction,
    },
    Note {
        #[command(subcommand)]
        action: cmd_note::NoteAction,
    },
    #[command(alias = "-s")]
    Server {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { target }) => cmd_add::run(target).await?,
        Some(Commands::Explain { target }) => cmd_explain::run(target).await?,
        Some(Commands::Review { all }) => cmd_review::run(all).await?,
        Some(Commands::Sync) => cmd_sync::run().await?,
        Some(Commands::Read { file }) => cmd_read::run(&file).await?,
        Some(Commands::Import { path }) => cmd_import::run(&path).await?,
        Some(Commands::Export { word, phrase, all }) => {
            cmd_export::run(word, phrase, all).await?
        }
        Some(Commands::Stats) => cmd_stats::run().await?,
        Some(Commands::Config { action }) => cmd_config::run(action).await?,
        Some(Commands::Note { action }) => cmd_note::run(action).await?,
        Some(Commands::Server { port }) => {
            let config = engai_core::config::Config::load_global()?;
            let db_path = config.db_path();
            let db = engai_core::db::Db::new(&db_path).await?;
            let state = crate::state::AppState::new(std::sync::Arc::new(db), config)?;
            crate::server::run_server(state, port).await?;
        }
        None => println!("Run `engai --help` for available commands"),
    }

    Ok(())
}
