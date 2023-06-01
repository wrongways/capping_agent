use crate::model::{RaplRecord, ServerInfo};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaplResponse {
    pub records: Vec<RaplRecord>,
}
