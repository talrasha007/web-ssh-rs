use std::sync::Arc;

use russh::keys::PrivateKey;

#[derive(Clone)]
pub struct AppState {
    pub identity_key: Option<Arc<PrivateKey>>,
}
