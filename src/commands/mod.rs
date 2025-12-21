pub mod cluster;
pub mod config;
pub mod init;
pub mod status;
pub mod run;
pub mod lsp;

use anyhow::Result;
use tokio::time::{ sleep, Duration };

pub async fn await_context(cluster_id: String, context_id: String) -> Result<()> {
    loop {
        let status = crate::client::context::get_status(cluster_id.to_owned(), context_id.to_owned()).await?.status;
        if status == "Running" {
            break;
        } else if status == "Error" {
           anyhow::bail!("Failed creating an execution context"); 
        }
        sleep(Duration::from_secs(2)).await;
    };
    Ok(())
}

