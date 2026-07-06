mod cli;
mod protocol;
mod routes;
mod ssh;
mod state;
mod target;
mod ws;

use std::sync::Arc;

use clap::Parser;
use tower_http::trace::TraceLayer;

use cli::Cli;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    let identity_key = cli
        .identity_file
        .as_ref()
        .map(|path| russh::keys::load_secret_key(path, cli.key_passphrase.as_deref()))
        .transpose()?
        .map(Arc::new);
    let state = AppState { identity_key };

    let app = routes::router(state).layer(TraceLayer::new_for_http());

    tracing::info!("listening on {}", cli.bind_addr);
    let listener = tokio::net::TcpListener::bind(cli.bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
