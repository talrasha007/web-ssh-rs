#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMsg {
    Data { data: String },
    Resize { cols: u32, rows: u32 },
}
