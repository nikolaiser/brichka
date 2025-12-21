use clap::{Parser, Subcommand};
use clap_stdin::MaybeStdin;


#[derive(Parser, Debug)]
#[command(name = "brichka")]
#[command(about = "Databricks cli tools", long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub cwd: Option<String>,

    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,

}


#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Cluster {
        #[command(subcommand)]
        command: ClusterCommands,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
        #[arg(long, short)]
        global: bool,
    },
    Init,
    Status {
        #[command(subcommand)]
        command: StatusCommands,
    },
    Run {
        command: MaybeStdin<String>,
        #[arg(short, long)]
        language: String,
        #[arg(long, short)]
        init: bool,
        #[arg(long, short)]
        start: bool,
    },
    Lsp
}

#[derive(Subcommand, Debug, Clone)]
pub enum ClusterCommands {
    List,
    Start,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    Cluster,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StatusCommands {
    Context,
    Cluster
}
