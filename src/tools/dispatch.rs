use crate::tools::implementations;
use serde_json::Value;
use serde::Serialize;
use crate::backend::state::AppState;

#[derive(Serialize)]
pub enum ToolResult {
    Output(String),
    Error(String),
}

pub async fn dispatch_tool(tool_name: &str, args: Value, state: &AppState) -> ToolResult {
    match tool_name {
        "echo_hello_world" => {
            match implementations::echo_hello_world().await {
                Ok(output) => ToolResult::Output(output),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "list_contents" => {
            match implementations::list_contents().await {
                Ok(output) => ToolResult::Output(output),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "spotify_get_devices" => {
            let devices = implementations::spotify_get_devices(&state.spotify_devices).await;
            ToolResult::Output(serde_json::to_string(&devices).unwrap_or_default())
        }
        "spotify_search" => {
            let query = args["query"].as_str().unwrap_or("");
            match implementations::spotify_search(
                &state.client,
                &state.spotify_token.lock().await,
                query,
            ).await {
                Ok(results) => ToolResult::Output(serde_json::to_string(&results).unwrap_or_default()),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "spotify_play_content" => {
            let device_name = args["device_name"].as_str().unwrap_or("");
            let uri = args["uri"].as_str().unwrap_or("");
            match implementations::spotify_play_content(
                &state.client,
                &state.spotify_token.lock().await,
                &state.spotify_devices,
                device_name,
                uri,
            ).await {
                Ok(_) => ToolResult::Output("Playback started".to_string()),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "spotify_pause_content" => {
            match implementations::spotify_pause_content(
                &state.client,
                &state.spotify_token.lock().await,
            ).await {
                Ok(_) => ToolResult::Output("Paused".to_string()),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "spotify_resume_content" => {
            match implementations::spotify_resume_content(
                &state.client,
                &state.spotify_token.lock().await,
            ).await {
                Ok(_) => ToolResult::Output("Resumed".to_string()),
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        _ => ToolResult::Error("unknown tool".to_string()),
    }
}
