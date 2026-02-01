use clap::{Parser, Subcommand};
use clap_stdin::MaybeStdin;


#[derive(Parser, Debug)]
#[command(name = "brichka")]
#[command(about = "Databricks cli tools", long_about = None)]
pub struct Cli {
    /// Override the current working dirrectory
    #[arg(long, global = true)]
    pub cwd: Option<String>,

    /// Print debug logs
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,

}


#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Cluster commands
    Cluster {
        #[command(subcommand)]
        command: ClusterCommands,
    },
    /// Config commands
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
        /// Save config globally (to `~/.config/brichka`) instead of saving it to the current working directory
        #[arg(long, short)]
        global: bool,
    },
    /// Initialize a new execution context in the current working directory
    Init,
    /// Status commands
    Status {
        #[command(subcommand)]
        command: StatusCommands,
    },
    /// Run code on the interactive cluster
    Run {
        /// Code that will be executed on the interactive cluster. Pass `-` to read from stdin
        command: MaybeStdin<String>,
        /// `sql`, `scala`, `python` or `r`
        #[arg(short, long)]
        language: String,
        /// If set brichka will automatically initialize a new shared execution context if the existing one does not exist or is not available anymore. If not set and no shared execution context can be found brichka will create a temporary one-off one
        #[arg(long, short)]
        init: bool,
        /// Automatically start a terminated cluster
        #[arg(long, short)]
        start: bool,
    },
    /// Start LSP server for Unity Catalog completion
    Lsp
}

#[derive(Subcommand, Debug, Clone)]
pub enum ClusterCommands {
    /// Prints a list of clusters with their id and current state
    List,
    /// Start a terminated cluster
    Start,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Run fuzzy finder to select a cluser that will be used to run the code
    Cluster,
    /// Configure Databricks authentication
    Auth {
        #[command(subcommand)]
        command: AuthConfigCommands,
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum StatusCommands {
    /// Show the current state of the shared execution context
    Context,
    /// Show the current state of the selected cluster
    Cluster
}

#[derive(Subcommand, Debug, Clone)]
pub enum AuthConfigCommands {
    Token {
        /// Databricks token
        #[arg(long, short)]
        value: String,
        /// Databricks base URL
        #[arg(long, short)]
        host: String
    },
    Cli {
        /// Path to Databricks CLI executable
        #[arg(long, short, default_value="databricks")]
        executable: String,
        /// Databricks profile to use
        #[arg(long, short, default_value="DEFAULT")]
        profile: String,
    }
}
