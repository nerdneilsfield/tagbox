use crate::commands::*;
use crate::utils::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader};
use tagbox_core::config::AppConfig;

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<serde_json::Value>,
    cmd: String,
    args: serde_json::Value,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Handle stdio mode (JSON-RPC)
pub async fn handle_stdio(config: &AppConfig) -> Result<()> {
    // Silence all logging for stdio mode to avoid interfering with JSON-RPC communication
    log::set_max_level(log::LevelFilter::Off);

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let response = process_request(&line, config).await;

        // Output response as JSON
        let response_json = serde_json::to_string(&response)?;
        println!("{}", response_json);
    }

    Ok(())
}

/// Process a single JSON-RPC request
async fn process_request(request_line: &str, config: &AppConfig) -> JsonRpcResponse {
    // Parse request
    let request: JsonRpcRequest = match serde_json::from_str(request_line) {
        Ok(req) => req,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700, // Parse error
                    message: format!("Parse error: {}", e),
                }),
            };
        }
    };

    // Execute command
    let result = execute_command(&request.cmd, &request.args, config).await;

    match result {
        Ok(result_value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result_value),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603, // Internal error
                message: e.to_string(),
            }),
        },
    }
}

/// Execute a command and return result as JSON value
async fn execute_command(
    cmd: &str,
    args: &serde_json::Value,
    config: &AppConfig,
) -> Result<serde_json::Value> {
    match cmd {
        "search" => {
            let query = get_string_arg(args, "query")?;
            let _json_output = get_bool_arg(args, "json").unwrap_or(true);
            let _columns = get_optional_string_arg(args, "columns");
            let limit = get_optional_usize_arg(args, "limit");
            let offset = get_optional_usize_arg(args, "offset");

            // For stdio mode, we capture the search result instead of printing
            let search_options = Some(tagbox_core::types::SearchOptions {
                offset: offset.unwrap_or(0),
                limit: limit.unwrap_or(50),
                sort_by: None,
                sort_direction: None,
                include_deleted: false,
            });

            let result = tagbox_core::search_files_advanced(&query, search_options, config).await?;
            Ok(serde_json::to_value(result)?)
        }

        "preview" => {
            let id = get_string_arg(args, "id")?;
            let file_entry = tagbox_core::get_file(&id, config).await?;
            Ok(serde_json::to_value(file_entry)?)
        }

        "import" => {
            let path = get_string_arg(args, "path")?;
            let path = std::path::Path::new(&path);

            // Extract other arguments
            let delete = get_bool_arg(args, "delete").unwrap_or(false);
            let category = get_optional_string_arg(args, "category");
            let title = get_optional_string_arg(args, "title");
            let authors = get_optional_string_arg(args, "authors");
            let year = get_optional_i32_arg(args, "year");
            let publisher = get_optional_string_arg(args, "publisher");
            let source = get_optional_string_arg(args, "source");
            let tags = get_optional_string_arg(args, "tags");
            let summary = get_optional_string_arg(args, "summary");

            // For stdio mode, we need to capture the import result
            import::handle_import(
                path, delete, category, title, authors, year, publisher, source, tags, summary,
                None, false, config,
            )
            .await?;

            Ok(serde_json::json!({
                "success": true,
                "message": "Import completed"
            }))
        }

        "link" => {
            let id1 = get_string_arg(args, "id1")?;
            let id2 = get_string_arg(args, "id2")?;
            let relation = get_optional_string_arg(args, "relation");

            tagbox_core::link_files(&id1, &id2, relation, config).await?;

            Ok(serde_json::json!({
                "success": true,
                "message": format!("Linked {} -> {}", id1, id2)
            }))
        }

        "unlink" => {
            let id1 = get_string_arg(args, "id1")?;
            let id2 = get_string_arg(args, "id2")?;

            tagbox_core::unlink_files(&id1, &id2, config).await?;

            Ok(serde_json::json!({
                "success": true,
                "message": format!("Unlinked {} -> {}", id1, id2)
            }))
        }

        "export" => {
            let _json_output = get_bool_arg(args, "json").unwrap_or(true);

            // Get all files for export
            let search_options = Some(tagbox_core::types::SearchOptions {
                offset: 0,
                limit: 1000000,
                sort_by: Some("created_at".to_string()),
                sort_direction: Some("desc".to_string()),
                include_deleted: false,
            });

            let result = tagbox_core::search_files_advanced("*", search_options, config).await?;
            Ok(serde_json::to_value(result.entries)?)
        }

        _ => Err(CliError::InvalidArgument(format!(
            "Unknown command: {}",
            cmd
        ))),
    }
}

/// Helper functions to extract arguments from JSON
fn get_string_arg(args: &serde_json::Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| CliError::InvalidArgument(format!("Missing required argument: {}", key)))
}

fn get_optional_string_arg(args: &serde_json::Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_bool_arg(args: &serde_json::Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}

fn get_optional_usize_arg(args: &serde_json::Value, key: &str) -> Option<usize> {
    args.get(key).and_then(|v| v.as_u64()).map(|u| u as usize)
}

fn get_optional_i32_arg(args: &serde_json::Value, key: &str) -> Option<i32> {
    args.get(key).and_then(|v| v.as_i64()).map(|i| i as i32)
}
