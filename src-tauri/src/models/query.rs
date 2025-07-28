use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub online: bool,
    pub players_online: Option<u32>,
    pub players_max: Option<u32>,
    pub motd: Option<String>,
    pub version: Option<String>,
    pub error: Option<String>,
}

impl QueryResponse {
    pub fn offline(error: String) -> Self {
        QueryResponse {
            online: false,
            players_online: None,
            players_max: None,
            motd: None,
            version: None,
            error: Some(error),
        }
    }
    
    pub fn online(players_online: u32, players_max: u32, motd: String, version: String) -> Self {
        QueryResponse {
            online: true,
            players_online: Some(players_online),
            players_max: Some(players_max),
            motd: Some(motd),
            version: Some(version),
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        QueryConfig {
            host: "127.0.0.1".to_string(),
            port: 25565,
            timeout_ms: 5000,
        }
    }
}