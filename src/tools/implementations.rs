use std::process::Command;
use std::process::ExitStatus;

pub async fn echo_hello_world() -> Result<ExitStatus, std::io::Error> {
    let status = Command::new("sh")
        .arg("/Users/anishkurani/Documents/Coding-Stuff/astra/src/integrations/script/echo.sh")
        .status()?;
    Ok(status)
}

pub async fn list_contents() -> Result<ExitStatus, std::io::Error> {
    let status = Command::new("ls")
        .status()?;
    Ok(status)
}