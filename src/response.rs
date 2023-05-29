use crate::model::{RaplRecord, ServerInfo};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ServerInfoResponse {
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct RaplResponse {
    pub records: [RaplRecord],
}
