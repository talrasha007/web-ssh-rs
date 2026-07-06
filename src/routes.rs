use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

use crate::ssh::connect_and_shell;
use crate::state::AppState;
use crate::target::{SshQuery, SshTarget, TargetParseError};
use crate::ws::handle_socket;

const INDEX_HTML: &str = include_str!("../static/index.html");

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ssh/{target}", get(ssh_page_path))
        .route("/ssh", get(ssh_page_query))
        .route("/ws/{target}", get(ws_handler_path))
        .route("/ws", get(ws_handler_query))
        .with_state(state)
}

fn bad_target(e: TargetParseError) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.to_string())
}

async fn ssh_page_path(
    Path(target): Path<String>,
) -> Result<Html<&'static str>, (StatusCode, String)> {
    let _target: SshTarget = target.parse().map_err(bad_target)?;
    Ok(Html(INDEX_HTML))
}

async fn ssh_page_query(Query(_query): Query<SshQuery>) -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn ws_handler_path(
    Path(target): Path<String>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let target: SshTarget = target.parse().map_err(bad_target)?;
    Ok(upgrade(ws, state, target))
}

async fn ws_handler_query(
    Query(query): Query<SshQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade(ws, state, query.into())
}

fn upgrade(ws: WebSocketUpgrade, state: AppState, target: SshTarget) -> axum::response::Response {
    ws.on_upgrade(move |socket| async move {
        match connect_and_shell(&target, state.identity_key.clone()).await {
            Ok((handle, channel)) => handle_socket(socket, handle, channel).await,
            Err(e) => tracing::error!(
                "ssh connect to {}@{}:{} failed: {e}",
                target.user,
                target.host,
                target.port
            ),
        }
    })
}
