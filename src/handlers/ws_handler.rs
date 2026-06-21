use axum::{extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame}, response::Response};
use crate::protocol::{Envelope, Message};

use crate::state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            WsFrame::Text(text) => {
                match serde_json::from_str::<Envelope>(&text) {
                    Ok(envelope) => {
                        // match on envelop.message
                        
                    }
                    Err(_) => {}
                }
            }

            WsFrame::Close(_) => return,
            _ => {} // ignore ping/pong binary frames
        }
    }
}