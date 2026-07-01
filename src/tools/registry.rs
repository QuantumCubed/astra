use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Serialize, Clone, Debug)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl Tool {
    pub fn new(name: &str, description: &str, parameters: Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            },
        }
    }
}

pub fn register_tools() -> Vec<Tool> {
    let no_args = serde_json::json!({ "type": "object", "properties": {} });
    vec![
        Tool::new(
            "echo_hello_world",
            "calls a .sh script to echo hello world",
            no_args.clone(),
        ),
        Tool::new(
            "list_contents",
            "lists the contents of the pwd",
            no_args.clone(),
        ),
        Tool::new(
            "spotify_get_devices",
            "gets active spotify client devices",
            no_args.clone(),
        ),
        Tool::new(
            "spotify_search",
            "searches Spotify for tracks, albums, and playlists — returns a list of (name, uri) results; call spotify_play_content with the chosen uri",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "the search term e.g. artist name, track name, album name"
                    }
                },
                "required": ["query"]
            }),
        ),
        Tool::new(
            "spotify_play_content",
            "plays spotify content by uri on target or active client",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "the spotify uri of the content to play e.g. spotify:track:..."
                    },
                    "device_name": {
                        "type": "string",
                        "description": "the name of the device to play on — omit to use the active device"
                    }
                },
                "required": ["uri"]
            }),
        ),
        Tool::new(
            "spotify_pause_content",
            "pauses spotify on active client",
            no_args.clone(),
        ),
        Tool::new(
            "spotify_resume_content",
            "resumes spotify on active client",
            no_args.clone(),
        ),
        Tool::new(
            "ha_get_devices",
            "loads current device information",
            no_args.clone()
        ),
        Tool::new(
            "ha_reconnect",
            "reconnects to Home Assistant if the connection was lost",
            no_args.clone()
        ),
        Tool::new(
            "ha_toggle_device",
            "toggles a home assistant entity on or off",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "the entity_id to toggle e.g. switch.standup_lamp"
                    }
                },
                "required": ["entity_id"]
            }),
        )
    ]
}
