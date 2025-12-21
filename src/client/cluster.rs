use serde::{Deserialize, Serialize};
use anyhow::Result;


#[derive(Deserialize)]
pub struct ListClustersResponse {
    pub clusters: Vec<Cluster>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cluster{
    #[serde(rename(deserialize  = "cluster_id" ))]
    pub id: String,
    #[serde(rename(deserialize  = "cluster_name" ))]
    pub name: String,
    pub state: String,
}



pub async fn list() -> Result<ListClustersResponse> {
    let response = crate::client::call_databricks_api::<ListClustersResponse>("get", "/api/2.0/clusters/list?filter_by.cluster_sources=UI", None).await?;
    Ok(response)
}


#[derive(Deserialize)]
pub struct GetClusterInfoResponse {
    pub state: String
}

pub async fn get_info(cluster_id: String) -> Result<GetClusterInfoResponse> {
    let path = format!("/api/2.1/clusters/get?cluster_id={}", cluster_id); 
    let response = crate::client::call_databricks_api::<GetClusterInfoResponse>("get", &path, None).await?;
    Ok(response)
}

pub async fn start(cluster_id: String) -> Result<GetClusterInfoResponse> {
    let request_body = format!("{{\"clusterId\": \"{}\"}}", cluster_id);
    let path = format!("/api/2.1/clusters/start"); 
    let response = crate::client::call_databricks_api::<GetClusterInfoResponse>("get", &path, Some(request_body)).await?;
    Ok(response)
}
