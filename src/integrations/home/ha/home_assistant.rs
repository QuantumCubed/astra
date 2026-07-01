use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicU32, atomic::Ordering};
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use serde_json::Value;
use tokio::sync::{Mutex, oneshot};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::backend::config::load_conf;
use crate::integrations::home::ha::config::{ha_token, ha_url};
use crate::integrations::home::ha::types::HaDevice;

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct HaClient {
    sink: Arc<Mutex<WsSink>>,
    msg_id: AtomicU32,
    pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>>,
}

impl HaClient {
    async fn establish() -> anyhow::Result<(WsSink, WsStream)> {
        let config = load_conf();
        let url = ha_url(&config).expect("expected ha endpoint");
        let token = ha_token(&config).expect("expected ha token");

        let (mut wsc, _) = tokio_tungstenite::connect_async(url).await?;

        let msg = wsc
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("connection closed before auth_required"))??;

        let text = match msg {
            Message::Text(t) => t,
            _ => anyhow::bail!("expected text frame for auth_required"),
        };

        let parsed: Value = serde_json::from_str(&text)?;

        if parsed["type"] != "auth_required" {
            anyhow::bail!("expected auth_required, got {}", parsed["type"]);
        }

        let auth = serde_json::json!({"type": "auth", "access_token": token});
        wsc.send(Message::Text(auth.to_string().into())).await?;

        let msg = wsc
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("auth_ok not received"))??;

        let text = match msg {
            Message::Text(t) => t,
            _ => anyhow::bail!("expected text frame for auth_ok"),
        };

        let parsed: Value = serde_json::from_str(&text)?;

        if parsed["type"] != "auth_ok" {
            anyhow::bail!("expected auth_ok, got {}", parsed["type"]);
        }

        Ok(wsc.split())
    }

    pub async fn connect() -> anyhow::Result<Self> {
        let (sink, stream) = Self::establish().await?;
        let pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>> = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(Self::event_loop(stream, Arc::clone(&pending)));

        tracing::info!("connected to Home Assistant");

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            msg_id: AtomicU32::new(1),
            pending,
        })
    }

    pub async fn reconnect(&self) -> anyhow::Result<()> {
        let (sink, stream) = Self::establish().await?;

        *self.sink.lock().await = sink;
        self.msg_id.store(1, Ordering::SeqCst);
        self.pending.lock().await.clear();

        tokio::spawn(Self::event_loop(stream, Arc::clone(&self.pending)));

        tracing::info!("reconnected to Home Assistant");

        Ok(())
    }

    async fn event_loop(
        mut stream: WsStream,
        pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>>,
    ) {
        while let Some(msg) = stream.next().await {
            let Ok(Message::Text(text)) = msg else { continue };
            let Ok(parsed) = serde_json::from_str::<Value>(&text) else { continue };

            if let Some(id) = parsed["id"].as_u64() {
                let mut map = pending.lock().await;
                if let Some(tx) = map.remove(&(id as u32)) {
                    let _ = tx.send(parsed);
                }
            } else {
                tracing::debug!("ha event: {}", parsed["type"]);
            }
        }
    }

    async fn send_command(&self, mut payload: Value) -> anyhow::Result<Value> {
        let id = self.msg_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        self.pending.lock().await.insert(id, tx);

        payload["id"] = Value::from(id);

        {
            let mut sink = self.sink.lock().await;
            sink.send(Message::Text(payload.to_string().into())).await?;
        }

        let res = rx.await?;

        if res["success"] != true {
            anyhow::bail!("HA command failed: {}", res["error"]);
        }

        Ok(res["result"].clone())
    }

    pub async fn get_states(&self) -> anyhow::Result<Value> {
        self.send_command(serde_json::json!({"type": "get_states"})).await
    }

    pub async fn get_entity_registry(&self) -> anyhow::Result<Value> {
        self.send_command(serde_json::json!({"type": "config/entity_registry/list"})).await
    }

    pub async fn get_area_registry(&self) -> anyhow::Result<Value> {
        self.send_command(serde_json::json!({"type": "config/area_registry/list"})).await
    }

    pub async fn get_devices(&self) -> anyhow::Result<Vec<HaDevice>> {
        let (states_res, entity_res, area_res) = tokio::join!(
            self.get_states(),
            self.get_entity_registry(),
            self.get_area_registry(),
        );
        let states = states_res?;
        let entities = entity_res?;
        let areas = area_res?;

        let area_map: HashMap<String, String> = areas
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|a| {
                let id = a["area_id"].as_str()?;
                let name = a["name"].as_str()?;
                Some((id.to_string(), name.to_string()))
            })
            .collect();

        let state_map: HashMap<String, String> = states
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|s| {
                let id = s["entity_id"].as_str()?;
                let state = s["state"].as_str()?;
                Some((id.to_string(), state.to_string()))
            })
            .collect();

        let devices: Vec<HaDevice> = entities
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|e| {
                if !e["disabled_by"].is_null() {
                    return None;
                }

                if e["options"]["conversation"]["should_expose"] != true {
                    return None;
                }

                let entity_id = e["entity_id"].as_str()?;
                let friendly_name = e["name"].as_str()
                    .or_else(|| e["original_name"].as_str())
                    .unwrap_or(entity_id)
                    .to_string();
                let aliases = e["aliases"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|a| a.as_str().map(String::from))
                    .collect();
                let area = e["area_id"].as_str()
                    .and_then(|id| area_map.get(id))
                    .cloned();
                let state = state_map.get(entity_id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                Some(HaDevice { entity_id: entity_id.to_string(), friendly_name, aliases, area, state })
            })
            .collect();

        Ok(devices)
    }

    pub async fn call_service(&self, domain: &str, service: &str, entity_id: &str) -> anyhow::Result<()> {
        self.send_command(serde_json::json!({
            "type": "call_service",
            "domain": domain,
            "service": service,
            "target": { "entity_id": entity_id }
        })).await?;

        Ok(())
    }
}
