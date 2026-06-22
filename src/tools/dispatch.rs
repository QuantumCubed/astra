use crate::tools::implementations;
use serde_json::Value;
use serde::Serialize;

#[derive(Serialize)]
pub enum ToolResult {
    Exit { success: bool, code: Option<i32> },
    Error(String),
}

pub async fn dispatch_tool(tool_name: &str, _args: Value) -> ToolResult {
    match tool_name {
        "echo_hello_world" => {
            match implementations::echo_hello_world().await {
                Ok(status) => ToolResult::Exit {
                    success: status.success(),
                    code: status.code(),
                },
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        "list_contents" => {
            match implementations::list_contents().await {
                Ok(status) => ToolResult::Exit {
                    success: status.success(),
                    code: status.code(),
                },
                Err(e) => ToolResult::Error(e.to_string()),
            }
        }
        _ => ToolResult::Error("unknown tool".to_string()),
    }
}
