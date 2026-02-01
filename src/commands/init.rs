

use crate::config::{ClusterConfig, ContextConfig};

use anyhow::Result;

pub async fn init() -> Result<()> {
    let cluster = ClusterConfig::read_local().await.or(ClusterConfig::read_global().await)?;
    let cluster_id = cluster.id;
    let context_id = crate::client::context::create(cluster_id.clone(), "sql".to_string()).await?.id;

    ContextConfig::new(context_id.to_owned()).write_local().await?;
    
    crate::commands::await_context(cluster_id.to_owned(), context_id.to_owned()).await?;

    Ok(())
}
