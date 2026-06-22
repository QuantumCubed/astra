use tokio::process::Command;

pub async fn echo_hello_world() -> Result<String, std::io::Error> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", "echo Hello, World!"])
        .output()
        .await?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", "echo 'Hello, World!'"])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn list_contents() -> Result<String, std::io::Error> {
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args(["-Command", "Get-ChildItem | Select-Object -ExpandProperty Name"])
        .output()
        .await?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", "ls"])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
