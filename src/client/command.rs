use reqwest::Method;
use serde::Deserialize;
use anyhow::Result;


#[derive(Deserialize)]
pub struct RunCommandResponse {
    pub id: String
}

pub async fn run(command: String, cluster_id: String, context_id: String, language: String) -> Result<RunCommandResponse> {
    let request_body = serde_json::json!({
        "clusterId": cluster_id,
        "contextId": context_id,
        "command": command,
        "language": language
    });

    let response = crate::client::call_databricks_api::<RunCommandResponse>(Method::POST, "/api/1.2/commands/execute", Some(request_body.to_string())).await?;
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct GetCommandInfoResponse{
    pub id: String,
    pub status: String,
    pub results: Option<CommandResults>,
}

#[derive(Debug, Deserialize)]
pub struct CommandResults {
    pub data: Option<serde_json::Value>,
    #[serde(rename = "resultType")]
    pub result_type: String,
    pub schema: Option<Vec<Schema>>,
    pub cause: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub name: String,
    #[serde(rename = "type")]
    pub tpe: String,
}

pub async fn get_info(command_id: String, cluster_id: String, context_id: String) -> Result<GetCommandInfoResponse> {
    let path = format!("/api/1.2/commands/status?clusterId={}&contextId={}&commandId={}", cluster_id, context_id, command_id); 
    let response = crate::client::call_databricks_api::<GetCommandInfoResponse>(Method::GET, &path, None).await?;
    Ok(response)
}
