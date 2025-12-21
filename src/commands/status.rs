use crate::config::{ClusterConfig, ContextConfig};

use anyhow::Result;

pub async fn context() -> Result<()> {
    let cluster = ClusterConfig::read_local().or(ClusterConfig::read_global())?;
    let context = ContextConfig::read_local()?;
    let status = crate::client::context::get_status(cluster.id, context.id).await?.status;
    println!("{}", status);
    Ok(())
}

pub async fn cluster() -> Result<()> {
    let cluster = ClusterConfig::read_local().or(ClusterConfig::read_global())?;
    let state = crate::client::cluster::get_info(cluster.id).await?.state;
    println!("{}", state);
    Ok(())
}
