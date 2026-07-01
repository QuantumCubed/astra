use tokio::process::Command;
use std::collections::HashMap;

use crate::integrations::spotify::spotify_connection::{play, pause, resume, search};
use crate::integrations::home::ha::home_assistant::HaClient;
use crate::integrations::home::ha::types::HaDevice;

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

pub async fn spotify_get_devices(devices: &tokio::sync::Mutex<HashMap<String, String>>) -> Vec<String> {
    devices
        .lock()
        .await
        .keys()
        .cloned()
        .collect()
}

pub async fn spotify_search(client: &reqwest::Client, token: &str, query: &str) -> Result<Vec<(String, String)>, reqwest::Error> {
    search(client, token, query).await
}

pub async fn spotify_play_content(client: &reqwest::Client, token: &str, devices: &tokio::sync::Mutex<HashMap<String, String>>, device_name: &str, spotify_uri: &str) -> Result<(), reqwest::Error> {
    let device_id = devices
        .lock()
        .await
        .get(device_name)
        .cloned();

    play(client, token, device_id.as_deref(), spotify_uri).await?;

    Ok(())
}

pub async fn spotify_pause_content(client: &reqwest::Client, token: &str) -> Result<(), reqwest::Error> {
    pause(client, token, None).await?;

    Ok(())
}

pub async fn spotify_resume_content(client: &reqwest::Client, token: &str) -> Result<(), reqwest::Error> {
    resume(client, token, None).await?;

    Ok(())
}

pub async fn ha_get_devices(ha: &HaClient) -> anyhow::Result<Vec<HaDevice>> {
    ha.get_devices().await
}

pub async fn ha_toggle_device(ha: &HaClient, entity_id: &str) -> anyhow::Result<()> {
    ha.call_service("homeassistant", "toggle", entity_id).await
}

pub async fn ha_reconnect(ha: &HaClient) -> anyhow::Result<()> {
    ha.reconnect().await
}

// pub async fn web_search
