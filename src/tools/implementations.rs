use tokio::process::Command;
use std::process::ExitStatus;

pub async fn echo_hello_world() -> Result<ExitStatus, std::io::Error> {
    Command::new("sh")
        .arg("src/integrations/script/echo.sh")
        .status()
        .await
}

pub async fn list_contents() -> Result<ExitStatus, std::io::Error> {
    Command::new("ls")
        .status()
        .await
}
