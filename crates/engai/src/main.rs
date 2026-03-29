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
        target: AddTarget,
    },
    Explain {
        #[command(subcommand)]
        target: ExplainTarget,
    },
    Review { #[arg(long)] all: bool },
    Sync,
    Read { file: String },
    Import { path: String },
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
        action: ConfigAction,
    },
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
    #[command(alias = "-s")]
    Server {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[derive(clap::Subcommand)]
enum AddTarget {
    Word { word: String },
    Phrase { phrase: String },
}

#[derive(clap::Subcommand)]
enum ExplainTarget {
    Word { word: String },
    Phrase { phrase: String },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    Init,
    Set { key: String, value: String },
    Get { key: String },
}

#[derive(clap::Subcommand)]
enum NoteAction {
    Add {
        target_type: String,
        target_id: i64,
        content: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _cli = Cli::parse();
    println!("engai: placeholder");
    Ok(())
}
