use anyhow::Result;
use tokio::time::{ sleep, Duration };

use crate::config::ClusterConfig;


pub async fn list() -> Result<()> {
    let response = crate::client::cluster::list().await?;
    let result_json = serde_json::to_string(&response.clusters)?;
    println!("{}", result_json);
    Ok(())
}

pub async fn status() -> Result<()> {
    let cluster = ClusterConfig::read_local().await.or(ClusterConfig::read_global().await)?;
    let state = crate::client::cluster::get_info(cluster.id).await?.state;
    println!("{}", state);
    Ok(())
}

pub async fn start() -> Result<()> {
    let cluster = ClusterConfig::read_local().await.or(ClusterConfig::read_global().await)?;
    let cluster_id = cluster.id;

    let state = crate::client::cluster::get_info(cluster_id.to_owned()).await?.state;
    match state.as_str() {
        "RUNNING" => return Ok(()),
        "TERMINATED" => {
            crate::client::cluster::start(cluster_id.to_owned()).await?;
        }
        _ => {}
    }

    loop {
        let state = crate::client::cluster::get_info(cluster_id.to_owned()).await?.state;
        if state == "RUNNING" {
            break;
        } else if state == "ERROR" {
           anyhow::bail!("Failed to start the cluster");
        }
        sleep(Duration::from_secs(2)).await;
    };

    Ok(())
}
