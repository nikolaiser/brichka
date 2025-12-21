use serde::Deserialize;
use anyhow::Result;


#[derive(Deserialize)]
pub struct CreateContextResponse {
    pub id: String
}

pub async fn create(cluster_id: String, language: String) -> Result<CreateContextResponse> {
    let request_body = format!("{{\"clusterId\": \"{}\", \"language\": \"{}\"}}", cluster_id, language);

    let response = crate::client::call_databricks_api::<CreateContextResponse>("post", "/api/1.2/contexts/create", Some(request_body)).await?;
    Ok(response)
}

#[derive(Deserialize)]
pub struct GetContextStatusResponse {
    pub status: String
}

pub async fn get_status(cluster_id: String, context_id: String) -> Result<GetContextStatusResponse> {
    let path = format!("/api/1.2/contexts/status?clusterId={}&contextId={}", cluster_id, context_id); 
    let response = crate::client::call_databricks_api::<GetContextStatusResponse>("get", &path, None).await?;
    Ok(response)
}
