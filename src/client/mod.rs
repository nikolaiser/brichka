pub mod cluster;
pub mod context;
pub mod command;
pub mod uc;

use std::process::Command;
use anyhow::{Context, Result};
use serde::Deserialize;

use crate::CONTEXT;


async fn call_databricks_api<T>(method: &str, path: &str, body: Option<String>) -> Result<T>
where T: for<'de> Deserialize<'de>
{
    let debug = CONTEXT.get().unwrap().debug;

    let mut params = vec!["api", method, path, "--output", "json"];
    if let Some(ref s) = body {
        params.push("--json");
        params.push(s);
    }

    let output = Command::new("databricks").args(&params).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to call databricks API: {}", stderr);
    }

    let stdout_str = String::from_utf8(output.stdout).context("Failed to parse Databricks CLI output as UTF-8")?;

    if debug {
        println!("{}", stdout_str.to_owned());
    }

    serde_json::from_str::<T>(&stdout_str).context("Failed to parse Databricks API output")
}
