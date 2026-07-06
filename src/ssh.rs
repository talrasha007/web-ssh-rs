use std::sync::Arc;

use russh::client::{self, Handle, Msg};
use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, PublicKey};
use russh::Channel;

use crate::target::SshTarget;

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
    key: Arc<PrivateKey>,
) -> anyhow::Result<(Handle<SshClientHandler>, Channel<Msg>)> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(
        config,
        (target.host.as_str(), target.port),
        SshClientHandler,
    )
    .await?;

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

    let channel = handle.channel_open_session().await?;
    channel
        .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
        .await?;
    channel.request_shell(false).await?;

    Ok((handle, channel))
}
