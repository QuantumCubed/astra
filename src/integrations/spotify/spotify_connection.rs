use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
}

#[derive (Deserialize)]
struct SpotifyDevice {
    id: String,
    is_active: bool,
    is_private_session: bool,
    is_restricted: bool, 
    name: String,
    supports_volume: bool,
    #[serde(rename = "type")]
    device_type: String,
    volume_percent: Option<u8>
}

#[derive(Deserialize)]
struct SpotifyGetDevicesResponse {
    devices: Vec<SpotifyDevice>
}

#[derive(Serialize)]
struct SpotifyPlayOffset {
    uri: Option<String>,
    position: Option<u32>
}

#[derive(Serialize)]
struct SpotifyPlayRequest {
    context_uri: Option<String>,
    uris: Option<Vec<String>>,
    offset: Option<SpotifyPlayOffset>,
    position_ms: u32
}

#[derive(Deserialize)]
struct SpotifySearchItem {
    name: String,
    uri: String,
}

#[derive(Deserialize)]
struct SpotifySearchPage {
    items: Vec<Option<SpotifySearchItem>>,
}

#[derive(Deserialize)]
struct SpotifySearchResponse {
    tracks: Option<SpotifySearchPage>,
    albums: Option<SpotifySearchPage>,
    playlists: Option<SpotifySearchPage>,
}

pub async fn search(client: &reqwest::Client, token: &str, query: &str) -> Result<Vec<(String, String)>, reqwest::Error> {
    let response = client
        .get("https://api.spotify.com/v1/search")
        .bearer_auth(token)
        .query(&[("q", query), ("type", "track,album,playlist"), ("limit", "5")])
        .send()
        .await?
        .json::<SpotifySearchResponse>()
        .await?;

    let mut results = Vec::new();

    if let Some(page) = response.tracks {
        for item in page.items.into_iter().flatten() {
            results.push((item.name, item.uri));
        }
    }
    if let Some(page) = response.albums {
        for item in page.items.into_iter().flatten() {
            results.push((item.name, item.uri));
        }
    }
    if let Some(page) = response.playlists {
        for item in page.items.into_iter().flatten() {
            results.push((item.name, item.uri));
        }
    }

    Ok(results)
}

pub async fn refresh_access_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str
) -> Result<String, reqwest::Error> {
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?
        .json::<SpotifyTokenResponse>()
        .await?;

    Ok(response.access_token)
}

pub async fn get_devices(client: &reqwest::Client, token: &str) -> Result<HashMap<String, String>, reqwest::Error> {
    let response = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(token)
        .send()
        .await?
        .json::<SpotifyGetDevicesResponse>()
        .await?;
    
    Ok(response.devices
        .into_iter()
        .map(|sd| (sd.name, sd.id))
        .collect())
}

pub async fn play(client: &reqwest::Client, token: &str, device_id: Option<&str>, uri: &str) -> Result<(), reqwest::Error> {
    
    let mut play_request = SpotifyPlayRequest {
        context_uri: None,
        uris: None,
        offset: None,
        position_ms: 0
    };

    if uri.starts_with("spotify:track") {
        play_request.uris = Some(vec![uri.to_string()])
    } else {
        play_request.context_uri = Some(uri.to_string())
    }

    let mut req = client
        .put("https://api.spotify.com/v1/me/player/play")
        .bearer_auth(token)
        .json(&play_request);
        if let Some(id) = device_id {
            req = req.query(&[("device_id", id)]) 
        }
         req.send()
        .await?;

        Ok(())
}

pub async fn pause(client: &reqwest::Client, token: &str, device_id: Option<&str>) -> Result<(), reqwest::Error> {
    let mut req = client
        .put("https://api.spotify.com/v1/me/player/pause")
        .bearer_auth(token);
        if let Some(id) = device_id {
            req = req.query(&[("device_id", id)]) 
        }
        req.send()
        .await?;

        Ok(())
}

pub async fn resume(client: &reqwest::Client, token: &str, device_id: Option<&str>) -> Result<(), reqwest::Error> {
    let mut req = client
        .put("https://api.spotify.com/v1/me/player/play")
        .bearer_auth(token);
        if let Some(id) = device_id {
            req = req.query(&[("device_id", id)]) 
        }
        req.send()
        .await?;

        Ok(())
}

