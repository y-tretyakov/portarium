use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "portarium",
    version,
    about = "Port monitoring and management CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List all open ports")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Watch ports in real-time")]
    Watch {
        #[arg(long, default_value = "2", help = "Polling interval in seconds")]
        interval: u64,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Show port event log")]
    Events {
        #[arg(long, help = "Filter by port number")]
        port: Option<u16>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Show connection graph")]
    Graph {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Show traffic for a specific port")]
    Traffic {
        port: u16,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Kill a process by PID")]
    Kill { pid: u32 },
    #[command(about = "Restart a process by PID")]
    Restart {
        pid: u32,
        #[arg(long, help = "Command to run")]
        cmd: String,
        #[arg(long, help = "Working directory")]
        cwd: String,
    },
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let cmd = match cli.command {
        Some(c) => c,
        None => Commands::List { json: false },
    };
    let mut service = portarium_core::PortariumService::default();
    match cmd {
        Commands::List { json } => commands::list(&mut service, json)?,
        Commands::Watch { interval, json } => commands::watch(&mut service, interval, json)?,
        Commands::Events { port, json } => commands::events(&service, port, json)?,
        Commands::Graph { json } => commands::graph(&mut service, json)?,
        Commands::Traffic { port, json } => commands::traffic(&service, port, json)?,
        Commands::Kill { pid } => commands::kill(&service, pid)?,
        Commands::Restart { pid, cmd, cwd } => commands::restart(&service, pid, &cmd, &cwd)?,
    }
    Ok(())
}
