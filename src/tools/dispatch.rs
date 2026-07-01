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
        "ha_get_devices" => {
            match &state.ha_client {
                Some(client) => {
                    match implementations::ha_get_devices(client).await {
                        Ok(devices) => ToolResult::Output(serde_json::to_string(&devices).unwrap_or_default()),
                        Err(e) => ToolResult::Error(e.to_string()),
                    }
                }
                None => ToolResult::Error("Home Assistant is not connected".to_string()),
            }
        }
        "ha_toggle_device" => {
            match &state.ha_client {
                Some(client) => {
                    let entity_id = args["entity_id"].as_str().unwrap_or("");
                    match implementations::ha_toggle_device(client, entity_id).await {
                        Ok(_) => ToolResult::Output("toggled".to_string()),
                        Err(e) => ToolResult::Error(e.to_string()),
                    }
                }
                None => ToolResult::Error("Home Assistant is not connected".to_string()),
            }
        }
        "ha_reconnect" => {
            match &state.ha_client {
                Some(client) => {
                    match implementations::ha_reconnect(client).await {
                        Ok(_) => ToolResult::Output("reconnected to Home Assistant".to_string()),
                        Err(e) => ToolResult::Error(e.to_string()),
                    }
                }
                None => ToolResult::Error("Home Assistant client was never initialized".to_string()),
            }
        }
        _ => ToolResult::Error("unknown tool".to_string()),
    }
}
