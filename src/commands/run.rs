
use std::env::{self};

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::{fs, time::{ Duration, sleep }};

use crate::{client::{command::{GetCommandInfoResponse, Schema}, context::GetContextStatusResponse}, config::{ClusterConfig, ContextConfig}};

async fn create_temporary_context(cluster_id: String) -> Result<String> {
    let context_id = crate::client::context::create(cluster_id.clone(), "sql".to_string()).await?.id;
    crate::commands::await_context(cluster_id.to_owned(), context_id.to_owned()).await?;
    Ok(context_id)
}

async fn get_or_create_context(cluster_id: String, init: bool) -> Result<String> {
    let existing_context = ContextConfig::read_local();
    match existing_context {
        Err(_) => {
            if init {
                crate::commands::init::init().await?;
                let context = ContextConfig::read_local()?;
                Ok(context.id)
            } else {
                create_temporary_context(cluster_id).await
            }
        },
        Ok(context) =>  {
            let response = crate::client::context::get_status(cluster_id, context.id.to_owned()).await;
            if let Ok(GetContextStatusResponse { ref status }) = response 
                && status == "Running" 
            {
                Ok(context.id)
            } else {
                if init {
                    crate::commands::init::init().await?;
                    let context = ContextConfig::read_local()?;
                    Ok(context.id)
                } else {
                    anyhow::bail!("Execution context does not exist anymore. Recreate it with `brichka init`")
                }
            }
        }
    }
}

fn filter_excluded_sections(command: &str) -> String {
    let mut result = Vec::new();
    let mut excluding = false;
    for line in command.lines() {
        let trimmed = line.trim();

        match trimmed {
            "// brichka: exclude" => {
                excluding = true;
                continue;
            },
            "// brichka: include" => {
                excluding = false;
                continue;
            },
            _ => {
                if !excluding {
                    result.push(line);
                };
            }
        }

    }
    result.join("\n")
}



async fn await_command_result(cluster_id: String, context_id: String, command_id: String) -> Result<GetCommandInfoResponse> {
    loop {
        let result = crate::client::command::get_info(command_id.to_owned(), cluster_id.to_owned(), context_id.to_owned()).await?;


        if result.status == "Finished" || result.status == "Error" || result.status == "Cancelled" {
            return Ok(result);
        } 
        sleep(Duration::from_secs(2)).await;
    };

}


#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum CommandRunResult {
    Text {
        value: String,
    },
    Error {
        message: String,
        cause: Option< String>
    },
    Table {
        path: String
    }
}

fn is_complex_type(field_type: &str) -> bool {
    let trimmed_type = field_type.trim();
    trimmed_type.starts_with("{}") || trimmed_type.starts_with("[")
}

fn format_array_value(schema: &serde_json::Value, value: &serde_json::Value) -> Result<String> {
    if let Some(element_type) = schema.get("elementType") {
        if let Some(array_values) = value.as_array() {
            let element_type_str = serde_json::to_string(element_type)?;

            if is_complex_type(&element_type_str) {
                let formatted_elements: Result<Vec<serde_json::Value>> = array_values
                    .iter()
                    .map(|elem| {
                        let formatted_str = format_complex_value(&element_type_str, elem)?;
                        Ok(serde_json::from_str(&formatted_str)?)
                    })
                    .collect();

                return Ok(serde_json::to_string(&formatted_elements?)?);
            }
        }
    }

    Ok(serde_json::to_string(value)?)
}

fn format_struct_value(schema: &serde_json::Value, value: &serde_json::Value) -> Result<String> {
    if let Some(fields) = schema.get("fields").and_then(|f| f.as_array()) {
        if let Some(array_values) = value.as_array() {
            let mut obj = serde_json::Map::new();
            for (i, field) in fields.iter().enumerate() {
                if let Some(field_name) = field.get("name").and_then(|n| n.as_str()) {
                    if i < array_values.len() {
                        let field_value = &array_values[i];

                        if let Some(field_type) = field.get("type") {
                            let field_type_str = serde_json::to_string(field_type)?;
                            let formatted_value = if is_complex_type(&field_type_str) {
                                let formatted_str = format_complex_value(&field_type_str, field_value)?;
                                serde_json::from_str(&formatted_str)?
                            } else {
                                field_value.clone()
                            };
                            obj.insert(field_name.to_string(), formatted_value);
                        } else {
                            obj.insert(field_name.to_string(), field_value.clone());
                        }
                    }
                }
            }

            let json_str = serde_json::to_string(&obj)?;
            // Replace ":" with ": " but only for keys (before values)
            let formatted = json_str.replace("\":", "\": ");
            return Ok(formatted);
        }
    }

    Ok(serde_json::to_string(value)?)
}

pub fn format_complex_value(type_str: &str, value: &serde_json::Value) -> Result<String> {
    if let Ok(schema) = serde_json::from_str::<serde_json::Value>(type_str) {
        if let Some(type_name) = schema.get("type").and_then(|t| t.as_str()) {
            match type_name {
                "struct" => {
                    return format_struct_value(&schema, value);
                }
                "array" => {
                    return format_array_value(&schema, value);
                }
                _ => {}
            }
        }
    }

    Ok(serde_json::to_string(value)?)
}

fn convert_field_value(field_type: &str, value: &serde_json::Value) -> Result<serde_json::Value> {
    if value.is_null() {
        return Ok(serde_json::Value::Null);
    }

    if is_complex_type(field_type) {
        if let Ok(formatted_json_str) = format_complex_value(field_type, value) {
            Ok(serde_json::from_str(&formatted_json_str)?)
        } else {
            Ok(value.clone())
        }
    } else {
        Ok(value.clone())
    }
}

fn build_row_object(
    schema: &[Schema],
    row_array: &[serde_json::Value],
) -> Result<serde_json::Map<String, serde_json::Value>> {
    let mut json_obj = serde_json::Map::new();

    for (i, value) in row_array.iter().enumerate() {
        let field_name = &schema[i].name;
        let field_type = &schema[i].tpe;

        let field_value = convert_field_value(field_type, value)?;
        json_obj.insert(field_name.clone(), field_value);
    }

    Ok(json_obj)
}

fn format_table_result(schema: &[Schema], data: &serde_json::Value) -> Result<Vec<serde_json::Value>> {
    let rows = data.as_array().context("Data is not an array")?;

    let mut result = Vec::new();

    for row in rows {
        let row_array = row.as_array().context("Row is not an array")?;

        if row_array.len() != schema.len() {
            anyhow::bail!(
                "Row length {} doesn't match schema length {}",
                row_array.len(),
                schema.len()
            );
        }

        let json_obj = build_row_object(schema, row_array)?;
        result.push(serde_json::Value::Object(json_obj));
    }

    Ok(result)
}

async fn write_table_result(command_id: String, result: Vec<serde_json::Value>) -> Result<String> {
    let temp_dir = env::temp_dir();
    let path = temp_dir.join("brichka").join("results").join(format!("{}.jsonl", command_id));
    if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).await?;
    }
    
    let result_string = serde_json::to_string(&result).context("Failed to serialize table data")?;

    fs::write(path.to_owned(), result_string).await?;

    Ok(path.to_string_lossy().to_string())
}

async fn format_command_result(result: GetCommandInfoResponse) -> Result<CommandRunResult> {
    let results = result.results.unwrap();
    match results.result_type.as_str() {
        "error" => {
            let summary = results.summary.context("Error result is missing summary")?;
            Ok(CommandRunResult::Error { message: summary, cause: results.cause })
        },
        "text" => {
            let data = results.data.context("Text result is missing data")?;
            Ok(CommandRunResult::Text { value: data.to_string() })
        },
        "table" => {
            let data = results.data.context("Table result is missing data")?;
            let schema = results.schema.context("Missing schema for tabular data")?;
            let formatted = format_table_result(&schema, &data)?;
            let path = write_table_result(result.id, formatted).await?; 
            Ok(CommandRunResult::Table { path })
        },
        _ => anyhow::bail!("Failed to format command results. Unexpected result type {}", results.result_type) 
    }
}


pub async fn check_cluster_state(cluster_id: String, start: bool) -> Result<()> {
    let state = crate::client::cluster::get_info(cluster_id.to_owned()).await?.state;

    if state == "RUNNING" || state == "RESIZING" {
        Ok(())
    } else if state == "TERMINATED" {
        if start {
           crate::commands::cluster::start().await
        } else {
            anyhow::bail!("Cluster is terminated. Use the --start flag if you want to start it");
        }
    } else {
        anyhow::bail!("Can not run command, cluster state is `{}`", state);
    }
}

pub async fn run(command: String, language: String, init: bool, start: bool) -> Result<()> {
    let cluster_id = ClusterConfig::read_local().or(ClusterConfig::read_global())?.id;
    check_cluster_state(cluster_id.to_owned(), start).await?;
    let context_id = get_or_create_context(cluster_id.to_owned(), init).await?;

    let filtered_command = filter_excluded_sections(&command);

    let command_id = crate::client::command::run(filtered_command, cluster_id.to_owned(), context_id.to_owned(), language).await?.id;

    let raw_result = await_command_result(cluster_id, context_id, command_id).await?;
    let formatted_result = format_command_result(raw_result).await?;
    let formatted_result_str = serde_json::to_string(&formatted_result)?;
    println!("{}", formatted_result_str);

    Ok(())

}
