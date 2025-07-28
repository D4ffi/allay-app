use crate::services::rcon_manager::RconManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServerStatus {
    Offline,
    Online,
}

#[derive(Clone, Serialize)]
pub struct ServerStatusEvent {
    pub server_name: String,
    pub old_status: ServerStatus,
    pub new_status: ServerStatus,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ServerState {
    pub status: ServerStatus,
    pub is_connecting: bool,
    pub last_connection_attempt: Option<std::time::Instant>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            status: ServerStatus::Offline,
            is_connecting: false,
            last_connection_attempt: None,
        }
    }
}

pub struct SimpleRconMonitor {
    servers: Arc<RwLock<HashMap<String, ServerState>>>,
    rcon_manager: Arc<Mutex<RconManager>>,
    monitoring_task: Option<tokio::task::JoinHandle<()>>,
    app_handle: Option<AppHandle>,
}

impl SimpleRconMonitor {
    pub fn new(rcon_manager: Arc<Mutex<RconManager>>) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            rcon_manager,
            monitoring_task: None,
            app_handle: None,
        }
    }

    /// Set the Tauri app handle for event emission
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    /// Start monitoring a server
    pub async fn start_monitoring(&self, server_name: String) {
        let mut servers = self.servers.write().await;
        servers.insert(server_name.clone(), ServerState::new());
        println!("🎯 Started RCON monitoring for server: {}", server_name);
    }

    /// Stop monitoring a server
    pub async fn stop_monitoring(&self, server_name: &str) {
        let mut servers = self.servers.write().await;
        servers.remove(server_name);
        
        // Disconnect RCON if connected
        let rcon = self.rcon_manager.lock().await;
        if rcon.is_connected(server_name) {
            drop(rcon);
            let _ = self.disconnect_rcon(server_name).await;
        }
        
        println!("🛑 Stopped RCON monitoring for server: {}", server_name);
    }

    /// Update server status manually (when user starts/stops server)
    pub async fn update_server_status(&self, server_name: &str, status: ServerStatus) {
        let mut servers = self.servers.write().await;
        if let Some(server_state) = servers.get_mut(server_name) {
            let old_status = server_state.status;
            if old_status != status {
                server_state.status = status;
                self.emit_status_change(server_name, old_status, status).await;
            }
        }
    }

    /// Get current server status
    pub async fn get_server_status(&self, server_name: &str) -> ServerStatus {
        let servers = self.servers.read().await;
        servers.get(server_name)
            .map(|state| state.status)
            .unwrap_or(ServerStatus::Offline)
    }

    /// Get all monitored servers and their statuses
    pub async fn get_all_statuses(&self) -> HashMap<String, ServerStatus> {
        let servers = self.servers.read().await;
        servers.iter()
            .map(|(name, state)| (name.clone(), state.status))
            .collect()
    }

    /// Start the background RCON monitoring task
    pub fn start_background_monitoring(&mut self) {
        if self.monitoring_task.is_some() {
            return;
        }

        println!("🚀 Starting simple RCON-based monitoring (15s intervals)");

        let servers = Arc::clone(&self.servers);
        let rcon_manager = Arc::clone(&self.rcon_manager);
        let app_handle = self.app_handle.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            
            loop {
                interval.tick().await;
                Self::monitor_cycle(Arc::clone(&servers), Arc::clone(&rcon_manager), app_handle.clone()).await;
            }
        });

        self.monitoring_task = Some(task);
    }

    /// Stop the background monitoring task
    pub fn stop_background_monitoring(&mut self) {
        if let Some(task) = self.monitoring_task.take() {
            task.abort();
        }
    }

    /// Single monitoring cycle - tries to connect via RCON
    async fn monitor_cycle(
        servers: Arc<RwLock<HashMap<String, ServerState>>>,
        rcon_manager: Arc<Mutex<RconManager>>,
        app_handle: Option<AppHandle>,
    ) {
        let server_list: Vec<String> = {
            let servers_read = servers.read().await;
            servers_read.keys().cloned().collect()
        };

        for server_name in server_list {
            let should_attempt_connection = {
                let servers_read = servers.read().await;
                if let Some(state) = servers_read.get(&server_name) {
                    // Only try to connect if:
                    // 1. Currently offline AND not already connecting
                    // 2. OR last attempt was more than 15 seconds ago (in case of failure)
                    state.status == ServerStatus::Offline && 
                    !state.is_connecting &&
                    (state.last_connection_attempt.is_none() || 
                     state.last_connection_attempt.unwrap().elapsed() >= Duration::from_secs(15))
                } else {
                    false
                }
            };

            if should_attempt_connection {
                // Try to connect via RCON
                {
                    let mut servers_write = servers.write().await;
                    if let Some(state) = servers_write.get_mut(&server_name) {
                        state.is_connecting = true;
                        state.last_connection_attempt = Some(std::time::Instant::now());
                    }
                }

                let connection_result = Self::attempt_rcon_connection(&server_name, &rcon_manager).await;

                // Update status based on connection result
                let mut servers_write = servers.write().await;
                if let Some(state) = servers_write.get_mut(&server_name) {
                    state.is_connecting = false;
                    
                    match connection_result {
                        Ok(()) => {
                            // Successfully connected
                            if state.status != ServerStatus::Online {
                                let old_status = state.status;
                                state.status = ServerStatus::Online;
                                
                                // Emit event
                                if let Some(ref app) = app_handle {
                                    let event = ServerStatusEvent {
                                        server_name: server_name.clone(),
                                        old_status,
                                        new_status: ServerStatus::Online,
                                        timestamp: SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis() as u64,
                                    };
                                    
                                    if let Err(e) = app.emit("server-status-changed", &event) {
                                        println!("⚠️ Failed to emit status event: {}", e);
                                    }
                                }
                                
                                println!("✅ {} now online via RCON", server_name);
                            }
                        },
                        Err(_) => {
                            // Connection failed, stay offline
                            // No need to log every failure - too spammy
                        }
                    }
                }
            } else {
                // Check if currently online server is still connected and perform heartbeat
                let is_connected = {
                    let rcon = rcon_manager.lock().await;
                    rcon.is_connected(&server_name)
                };

                if is_connected {
                    // Passive heartbeat - handle Keep Alive messages without sending commands
                    let rcon = rcon_manager.lock().await;
                    rcon.heartbeat_all();
                    drop(rcon);
                }

                let mut servers_write = servers.write().await;
                if let Some(state) = servers_write.get_mut(&server_name) {
                    if state.status == ServerStatus::Online && !is_connected {
                        // Server was online but RCON disconnected
                        let old_status = state.status;
                        state.status = ServerStatus::Offline;
                        
                        // Emit event
                        if let Some(ref app) = app_handle {
                            let event = ServerStatusEvent {
                                server_name: server_name.clone(),
                                old_status,
                                new_status: ServerStatus::Offline,
                                timestamp: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64,
                            };
                            
                            if let Err(e) = app.emit("server-status-changed", &event) {
                                println!("⚠️ Failed to emit status event: {}", e);
                            }
                        }
                        
                        println!("❌ {} went offline (RCON disconnected)", server_name);
                    }
                }
            }
        }
    }

    /// Attempt to connect to a server via RCON
    async fn attempt_rcon_connection(
        server_name: &str,
        rcon_manager: &Arc<Mutex<RconManager>>,
    ) -> Result<(), String> {
        let rcon = rcon_manager.lock().await;
        
        // Setup RCON with default settings (need to import RconConfig)
        use crate::services::rcon_manager::RconConfig;
        let config = RconConfig {
            host: "127.0.0.1".to_string(),
            port: 25575,
            password: "minecraft".to_string(),
        };
        
        rcon.add_server(server_name.to_string(), config);
        
        // Try to connect
        match rcon.connect(server_name) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("RCON connection failed: {}", e)),
        }
    }

    /// Disconnect RCON for a server
    async fn disconnect_rcon(&self, server_name: &str) -> Result<(), String> {
        let rcon = self.rcon_manager.lock().await;
        rcon.disconnect(server_name);
        Ok(())
    }

    /// Emit status change event
    async fn emit_status_change(&self, server_name: &str, old_status: ServerStatus, new_status: ServerStatus) {
        if let Some(ref app) = self.app_handle {
            let event = ServerStatusEvent {
                server_name: server_name.to_string(),
                old_status,
                new_status,
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            };
            
            if let Err(e) = app.emit("server-status-changed", &event) {
                println!("⚠️ Failed to emit status event: {}", e);
            }
        }
    }
}

impl Drop for SimpleRconMonitor {
    fn drop(&mut self) {
        self.stop_background_monitoring();
    }
}