use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use russh::client::{Handle, Msg};
use russh::{Channel, ChannelMsg};
use tokio_util::sync::CancellationToken;

use crate::ssh::SshClientHandler;

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMsg {
    Data { data: String },
    Resize { cols: u32, rows: u32 },
}

/// Bridges a browser WebSocket to an SSH shell channel. `handle` is held for the
/// life of the session — dropping it would tear down the underlying SSH connection.
pub async fn handle_socket(
    socket: WebSocket,
    handle: Handle<SshClientHandler>,
    channel: Channel<Msg>,
) {
    let _handle = handle;
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (mut read_half, write_half) = channel.split();

    let cancel = CancellationToken::new();

    let cancel_out = cancel.clone();
    let ssh_to_ws = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_out.cancelled() => break,
                msg = read_half.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { data }) | Some(ChannelMsg::ExtendedData { data, .. }) => {
                            if ws_tx.send(Message::Binary(data)).await.is_err() {
                                break;
                            }
                        }
                        Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | Some(ChannelMsg::ExitStatus { .. }) | None => break,
                        _ => {}
                    }
                }
            }
        }
        cancel_out.cancel();
        let _ = ws_tx.close().await;
    });

    let cancel_in = cancel.clone();
    let ws_to_ssh = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_in.cancelled() => break,
                incoming = ws_rx.next() => {
                    let Some(Ok(msg)) = incoming else { break };
                    match msg {
                        Message::Text(text) => match serde_json::from_str::<ClientMsg>(text.as_str()) {
                            Ok(ClientMsg::Data { data }) => {
                                if write_half.data_bytes(data.into_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ClientMsg::Resize { cols, rows }) => {
                                let _ = write_half.window_change(cols, rows, 0, 0).await;
                            }
                            Err(e) => tracing::warn!("bad client frame: {e}"),
                        },
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        }
        cancel_in.cancel();
    });

    let _ = tokio::join!(ssh_to_ws, ws_to_ssh);
}
