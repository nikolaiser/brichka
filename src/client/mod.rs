pub mod cluster;
pub mod context;
pub mod command;
pub mod uc;

use std::fmt::format;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use crate::{CONTEXT, config::AuthConfig};

#[derive(Deserialize)]
struct DatabricksTokenCreateResponse {
    token_value: String
}

async fn get_token_cli(path: &str) -> Result<String> {
    let params = vec!["tokens", "create", "--comment", "Brichka temporary token", "--lifetime-seconds", "60"];

    let output = Command::new(path).args(params).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create a temporary token: {}", stderr);
    }

    let stdout_str = String::from_utf8(output.stdout).context("Failed to parse Databricks CLI output as UTF-8")?;

    serde_json::from_str::<DatabricksTokenCreateResponse>(&stdout_str).context("Failed to parse databricks CLI output").map(|response|response.token_value)
}


#[derive(Deserialize)]
struct DatabricksDescribeProfileResponse {
    details: Details
}

#[derive(Deserialize)]
struct Details {
    host: String
}

async fn get_host_cli(path: &str, profile: &str) -> Result<String> {
    let params = vec!["auth", "describe", "--profile", profile, "--output", "json"];

    let output = Command::new(path).args(params).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create a temporary token: {}", stderr);
    }

    let stdout_str = String::from_utf8(output.stdout).context("Failed to parse Databricks CLI output as UTF-8")?;

    serde_json::from_str::<DatabricksDescribeProfileResponse>(&stdout_str).context("Failed to parse databricks CLI output").map(|response|response.details.host)
}

struct DatabricksAuthConfig {
    token: String,
    host: String
}

async fn get_auth_config() -> Result<DatabricksAuthConfig> {
    let config = AuthConfig::read_global().await.context("Failed to read authentication config")?;

    match config {
        AuthConfig::DatabricksCli { path, profile } => 
            { 
                let token = get_token_cli(&path).await?;
                let host = get_host_cli(&path, &profile).await?;
                Ok(DatabricksAuthConfig{token: token, host: host})
            },
        AuthConfig::Token { value, host } => Ok(DatabricksAuthConfig{token: value, host: host}),
    }
}

async fn call_databricks_api<T>(method: reqwest::Method, path: &str, body: Option<String>) -> Result<T>
where T: for<'de> Deserialize<'de>
{
    let debug = CONTEXT.get().unwrap().debug;

    let auth_config = get_auth_config().await?;
    let client = reqwest::Client::builder().build()?;

    let token_header = format!("Bearer {}", auth_config.token);

    let url = format!("{}{}",auth_config.host, path);

    let base_request = client.request(method, url).header("Authorization", token_header);

    let request = if let Some(body_str) = body {
        base_request.body(body_str)
    } else {
        base_request
    };

    let response = request.send().await?;

    let response_status = response.status();
    let response_text = response.text().await?;


    if debug {
        println!("{}", response_status.to_owned());
        println!("{}", response_text.to_owned());
    }

    serde_json::from_str::<T>(&response_text).context("Failed to parse Databricks API output")
}
