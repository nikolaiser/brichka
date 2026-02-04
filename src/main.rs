mod client;
mod cli;
mod commands;
mod config;

use std::sync::OnceLock;

use anyhow::{Result, Context};
use clap::Parser;

use crate::cli::{Cli, ClusterCommands, Commands, ConfigCommands, StatusCommands};

#[derive(Debug)]
pub struct BrichkaContext {
   pub debug: bool,
   pub cwd: String
}

pub static CONTEXT: OnceLock<BrichkaContext> = OnceLock::new();

async fn run(cli: &Cli) -> Result<()> {


    match cli.command.to_owned() {
        Commands::Cluster { command } => match command {
            ClusterCommands::List => commands::cluster::list().await?,
            ClusterCommands::Start => commands::cluster::start().await?
        },
        Commands::Config { command, global } => match command {
            ConfigCommands::Cluster => commands::config::select_cluster(global).await?,
            ConfigCommands::Auth { command } => match command{
                cli::AuthConfigCommands::Token { value, host } => crate::commands::config::configure_token_auth(value, host).await?,
                cli::AuthConfigCommands::Cli { executable: path, profile } => crate::commands::config::configure_cli_auth(path, profile).await?,
            },
        },
        Commands::Init => commands::init::init().await?,
        Commands::Status { command } => match command {
            StatusCommands::Context => commands::status::context().await?,
            StatusCommands::Cluster => commands::status::cluster().await?
        },
        Commands::Run { command, language, init, start } => commands::run::run(command.into_inner(), language, init, start).await?,
        Commands::Lsp => commands::lsp::start().await?,
        Commands::Version => println!("{}", env!("CARGO_PKG_VERSION"))
    };

    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();

    let cwd = match cli.cwd.to_owned() {
        Some(dir) => dir,
        None => std::env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .to_string(),
    };

    CONTEXT.set(BrichkaContext{
        debug: cli.debug,
        cwd: cwd
    }).unwrap();

    let result = run(&cli).await;

    if let Err(e) = result {
        eprintln!("{}", e.to_string());
        if CONTEXT.get().unwrap().debug {
            eprintln!("{}", e.backtrace().to_string());
        }
    }
    
    
    Ok(())
}
