use anyhow::Result;
use tokio::time::{ sleep, Duration };

use crate::{client::cluster::ListClustersResponse, config::ClusterConfig};


pub async fn list() -> Result<()> {
    let response = crate::client::cluster::list().await?;
    let result_json = serde_json::to_string(&response.clusters)?;
    println!("{}", result_json);
    Ok(())
}

pub async fn start() -> Result<()> {
    let cluster = ClusterConfig::read_local().or(ClusterConfig::read_global())?;
    let cluster_id = cluster.id;
    crate::client::cluster::start(cluster_id.to_owned()).await?;
    
    
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
