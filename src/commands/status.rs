use crate::config::{ClusterConfig, ContextConfig};

use anyhow::Result;

pub async fn context() -> Result<()> {
    let cluster = ClusterConfig::read_local().await.or(ClusterConfig::read_global().await)?;
    let context = ContextConfig::read_local().await?;
    let status = crate::client::context::get_status(cluster.id, context.id).await?.status;
    println!("{}", status);
    Ok(())
}

pub async fn cluster() -> Result<()> {
    let cluster = ClusterConfig::read_local().await.or(ClusterConfig::read_global().await)?;
    let state = crate::client::cluster::get_info(cluster.id).await?.state;
    println!("{}", state);
    Ok(())
}
