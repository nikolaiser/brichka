use std::io::Cursor;

use anyhow::{ Result, Context };
use skim::prelude::*;

use crate::{commands::config, config::ClusterConfig};

fn render_cluster_state(state: &str) -> &str {
    match state {
        "RUNNING" =>  "🟢",
        "PENDING" => "🔵",
        "RESTARTING" => "♻️",
        "RESIZING" => "⚙️",
        "TERMINATING" => "🔻",
        "TERMINATED" => "🔴",
        "ERROR" => "⚠️",
        "UNKNOWN" => "❔",
        _ => "❔"
    }
}

pub async fn select_cluster(global: bool) -> Result<()> {
    let clusters = crate::client::cluster::list().await?.clusters;
    
    let items: String = clusters
        .iter()
        .map(|c| format!("{} {} ({})", render_cluster_state(&c.state), c.name, c.id))
        .collect::<Vec<String>>().join("\n");

    let selected_index = run_skim(&items)?;
    let selected_cluster = clusters.get(selected_index).unwrap();
    let config = ClusterConfig::new(selected_cluster);
    if global {
        config.write_global()?
    } else {
        config.write_local()?
    }
    
    Ok(())
}

fn run_skim(items: &str) -> Result<usize> {
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .prompt("Select cluster: ".to_string())
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let skim_items = item_reader.of_bufread(Cursor::new(items.to_string()));


    let output = Skim::run_with(&options, Some(skim_items)).context("Failed to run skim")?;

    if output.is_abort {
        anyhow::bail!("Selection aborted");
    }

    let selected = output
        .selected_items
        .first()
        .context("No item selected")?;

    Ok(selected.get_index())
}

pub async fn configure_auth(token: Option<String>, cli: Option<String>) -> Result<()> {
    let maybe_token_config = token.map(|token_str|crate::config::AuthConfig::Token { value: token_str });
    let maybe_cli_config = cli.map(|path_str|crate::config::AuthConfig::DatabricksCli { path: path_str } );

    let config = maybe_token_config.or(maybe_cli_config).context("Either CLI or token auth should be specified")?;

    config.write_global()
}
