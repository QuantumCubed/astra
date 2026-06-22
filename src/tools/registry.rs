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
            no_args,
        ),
    ]
}
