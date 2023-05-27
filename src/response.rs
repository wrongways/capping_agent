use crate::model::{RaplRecord, ServerInfo};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ServerInfoResponse {
    pub status: String,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct RaplResponse {
    pub status: String,
    pub records: [RaplRecord],
}
