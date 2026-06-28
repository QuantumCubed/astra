use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicU32};
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use serde_json::Value;
use tokio::sync::{Mutex, oneshot};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::backend::config::load_conf;
use crate::integrations::home::ha::config::{ha_token, ha_url};

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct HaClient {
    sink: Arc<Mutex<WsSink>>,
    msg_id: AtomicU32,
    pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>>,
}

impl HaClient {
    pub async fn connect() -> anyhow::Result<Self> {
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
            anyhow::bail!("expected auth_required, got {}",
                parsed["type"]);
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
            anyhow::bail!("expected auth_ok, got {}",
                parsed["type"]);
        }

        tracing::info!("connected to Home Assistant");

        let (sink, stream) = wsc.split();
        let pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>> = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(Self::event_loop(stream, Arc::clone(&pending)));

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            msg_id: AtomicU32::new(1),
            pending,
        })
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
}
