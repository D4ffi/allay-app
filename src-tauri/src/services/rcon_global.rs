use std::sync::Arc;
use lazy_static::lazy_static;
use super::{rcon_manager::RconManager, heartbeat_manager::HeartbeatManager};

lazy_static! {
    pub static ref GLOBAL_RCON_MANAGER: Arc<RconManager> = Arc::new(RconManager::new());
    pub static ref GLOBAL_HEARTBEAT_MANAGER: HeartbeatManager = HeartbeatManager::new();
}

pub fn get_rcon_manager() -> Arc<RconManager> {
    GLOBAL_RCON_MANAGER.clone()
}

pub fn get_heartbeat_manager() -> &'static HeartbeatManager {
    &GLOBAL_HEARTBEAT_MANAGER
}