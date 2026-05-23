use std::collections::HashMap;
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

#[derive(Serialize)]
pub struct NoArgs {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub properties: serde_json::Value,
}

impl NoArgs {
    pub fn new() -> Self {
        Self {
            type_: "object",
            properties: serde_json::json!({}),
        }
    }
}

pub fn register_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "echo_hello_world",
            "calls a .sh script to echo hello world",
            serde_json::to_value(NoArgs::new()).unwrap(),
        ),
        Tool::new(
            "list_contents",
            "lists the contents of the pwd",
            serde_json::to_value(NoArgs::new()).unwrap(),
        )
    ]
}
