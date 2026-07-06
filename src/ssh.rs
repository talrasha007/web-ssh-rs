use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use russh::client::{self, Handle, Msg};
use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, PublicKey};
use russh::Channel;

use crate::protocol::ClientMsg;
use crate::target::SshTarget;

const MAX_PASSWORD_ATTEMPTS: u32 = 3;

pub struct SshClientHandler;

impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    /// Accept any server host key. There is no TLS/auth layer in front of this
    /// service and no known_hosts store to check against — this is an explicit
    /// simplification for the "core functionality only" scope of this project.
    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn connect_and_shell(
    target: &SshTarget,
    identity_key: Option<Arc<PrivateKey>>,
    socket: &mut WebSocket,
) -> anyhow::Result<(Handle<SshClientHandler>, Channel<Msg>)> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(
        config,
        (target.host.as_str(), target.port),
        SshClientHandler,
    )
    .await?;

    // Tracks the terminal size reported by the browser while it may still be
    // arriving during an interactive password prompt, so the PTY is sized
    // correctly from the start instead of always defaulting to 80x24.
    let mut term_size = (80u32, 24u32);

    match identity_key {
        Some(key) => {
            let hash_alg = handle.best_supported_rsa_hash().await?.flatten();
            let auth_result = handle
                .authenticate_publickey(&target.user, PrivateKeyWithHashAlg::new(key, hash_alg))
                .await?;
            if !auth_result.success() {
                anyhow::bail!(
                    "SSH publickey authentication failed for {}@{}:{}",
                    target.user,
                    target.host,
                    target.port
                );
            }
        }
        None => authenticate_with_password(&mut handle, target, socket, &mut term_size).await?,
    }

    let channel = handle.channel_open_session().await?;
    channel
        .request_pty(false, "xterm-256color", term_size.0, term_size.1, 0, 0, &[])
        .await?;
    channel.request_shell(false).await?;

    Ok((handle, channel))
}

async fn authenticate_with_password(
    handle: &mut Handle<SshClientHandler>,
    target: &SshTarget,
    socket: &mut WebSocket,
    term_size: &mut (u32, u32),
) -> anyhow::Result<()> {
    for attempt in 0..MAX_PASSWORD_ATTEMPTS {
        let prompt = format!("{}@{}'s password: ", target.user, target.host);
        socket
            .send(Message::Binary(prompt.into_bytes().into()))
            .await?;

        let Some(password) = read_password_line(socket, term_size).await? else {
            anyhow::bail!("client disconnected during authentication");
        };
        socket
            .send(Message::Binary(b"\r\n".to_vec().into()))
            .await?;

        let auth_result = handle
            .authenticate_password(&target.user, &password)
            .await?;
        if auth_result.success() {
            return Ok(());
        }

        if attempt + 1 < MAX_PASSWORD_ATTEMPTS {
            socket
                .send(Message::Binary(
                    b"Permission denied, please try again.\r\n".to_vec().into(),
                ))
                .await?;
        }
    }

    socket
        .send(Message::Binary(
            b"Too many authentication failures\r\n".to_vec().into(),
        ))
        .await?;
    anyhow::bail!(
        "password authentication failed for {}@{}:{}",
        target.user,
        target.host,
        target.port
    );
}

/// Reads keystrokes from the browser until Enter, without echoing them back
/// (nothing is written to the terminal for typed characters, matching a
/// normal SSH client's blind password entry). Also keeps `term_size` updated
/// so a resize sent by the browser while this prompt is showing isn't lost.
async fn read_password_line(
    socket: &mut WebSocket,
    term_size: &mut (u32, u32),
) -> anyhow::Result<Option<String>> {
    let mut buf = String::new();
    while let Some(msg) = socket.next().await {
        match msg? {
            Message::Text(text) => match serde_json::from_str::<ClientMsg>(text.as_str()) {
                Ok(ClientMsg::Data { data }) => {
                    for ch in data.chars() {
                        match ch {
                            '\r' | '\n' => return Ok(Some(buf)),
                            '\x7f' | '\u{8}' => {
                                buf.pop();
                            }
                            '\x03' => anyhow::bail!("authentication interrupted"),
                            c => buf.push(c),
                        }
                    }
                }
                Ok(ClientMsg::Resize { cols, rows }) => *term_size = (cols, rows),
                Err(e) => tracing::warn!("bad client frame: {e}"),
            },
            Message::Close(_) => return Ok(None),
            _ => {}
        }
    }
    Ok(None)
}
