use crate::models::query::{QueryResponse, QueryConfig};
use crate::services::query_service::QueryService;
use crate::services::rcon_manager::RconManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServerMonitorStatus {
    Offline,
    Starting,
    Online,
    Stopping,
}

#[derive(Clone, Serialize)]
pub struct ServerStatusEvent {
    pub server_name: String,
    pub old_status: ServerMonitorStatus,
    pub new_status: ServerMonitorStatus,
    pub timestamp: u64, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone)]
pub struct ServerMonitorState {
    pub status: ServerMonitorStatus,
    pub last_query_success: Option<Instant>,
    pub last_rcon_success: Option<Instant>,
    pub last_check: Instant,
    pub last_status_change: Instant,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub port: u16,
}

impl ServerMonitorState {
    pub fn new(port: u16) -> Self {
        Self {
            status: ServerMonitorStatus::Offline,
            last_query_success: None,
            last_rcon_success: None,
            last_check: Instant::now(),
            last_status_change: Instant::now(),
            consecutive_failures: 0,
            consecutive_successes: 0,
            port,
        }
    }

    pub fn is_online(&self) -> bool {
        self.status == ServerMonitorStatus::Online
    }

    pub fn is_transitioning(&self) -> bool {
        matches!(self.status, ServerMonitorStatus::Starting | ServerMonitorStatus::Stopping)
    }
}

pub struct ServerMonitor {
    servers: Arc<RwLock<HashMap<String, ServerMonitorState>>>,
    rcon_manager: Arc<Mutex<RconManager>>,
    monitoring_task: Option<tokio::task::JoinHandle<()>>,
    app_handle: Option<AppHandle>,
}

impl ServerMonitor {
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
    pub async fn start_monitoring(&self, server_name: String, port: u16) {
        let mut servers = self.servers.write().await;
        servers.insert(server_name.clone(), ServerMonitorState::new(port));
        println!("Started monitoring server: {} on port {}", server_name, port);
    }

    /// Stop monitoring a server
    pub async fn stop_monitoring(&self, server_name: &str) {
        let mut servers = self.servers.write().await;
        servers.remove(server_name);
        println!("Stopped monitoring server: {}", server_name);
    }

    /// Update server status manually (for start/stop commands)
    pub async fn update_server_status(&self, server_name: &str, status: ServerMonitorStatus) {
        let mut servers = self.servers.write().await;
        if let Some(server_state) = servers.get_mut(server_name) {
            let old_status = server_state.status;
            if old_status != status {
                server_state.status = status;
                server_state.last_status_change = Instant::now();
                server_state.consecutive_failures = 0;
                server_state.consecutive_successes = 0;
                println!("🔄 Manual update server {} status: {:?} → {:?}", server_name, old_status, status);
            }
        } else {
            println!("⚠️  Tried to update status for unknown server: {}", server_name);
        }
    }

    /// Get current server status
    pub async fn get_server_status(&self, server_name: &str) -> ServerMonitorStatus {
        let servers = self.servers.read().await;
        servers.get(server_name)
            .map(|state| state.status.clone())
            .unwrap_or(ServerMonitorStatus::Offline)
    }

    /// Get all monitored servers and their statuses
    pub async fn get_all_statuses(&self) -> HashMap<String, ServerMonitorStatus> {
        let servers = self.servers.read().await;
        let statuses: HashMap<String, ServerMonitorStatus> = servers.iter()
            .map(|(name, state)| (name.clone(), state.status.clone()))
            .collect();
        
        if !statuses.is_empty() {
            println!("📊 Event-driven status summary: {} servers monitored", statuses.len());
            for (name, status) in &statuses {
                println!("• {}: {:?}", name, status);
            }
        }
        
        statuses
    }
    
    /// Diagnostic method to verify event system health
    pub async fn diagnose_event_system(&self) -> String {
        let servers = self.servers.read().await;
        let server_count = servers.len();
        let has_app_handle = self.app_handle.is_some();
        
        let diagnostic = format!(
            "🔍 Event System Diagnostic:\n\
             • Servers monitored: {}\n\
             • App handle available: {}\n\
             • Monitoring task active: {}\n\
             • Mode: Pure event-driven (no polling)",
            server_count,
            has_app_handle,
            self.monitoring_task.is_some()
        );
        
        println!("{}", diagnostic);
        diagnostic
    }

    /// Start the background monitoring task
    pub fn start_background_monitoring(&mut self) {
        // Prevent multiple monitoring tasks
        if self.monitoring_task.is_some() {
            println!("⚠️ Monitoring task already running, skipping duplicate start");
            return;
        }
        
        println!("🚀 ✅ Starting PURE EVENT-DRIVEN monitoring (no frontend polling)");

        let servers = Arc::clone(&self.servers);
        let rcon_manager = Arc::clone(&self.rcon_manager);
        let app_handle = self.app_handle.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(20)); // Check every 20 seconds (less aggressive)

            println!("🔍 Background monitoring thread started (20s intervals)");
            loop {
                interval.tick().await;
                
                let server_count = {
                    let servers_read = servers.read().await;
                    servers_read.len()
                };
                
                // Declare the static variable outside both blocks
                static mut CYCLE_COUNT: usize = 0;
                
                if server_count > 0 {
                    // Only log every 3rd cycle to reduce spam (every 60 seconds)
                    unsafe {
                        CYCLE_COUNT += 1;
                        if CYCLE_COUNT % 3 == 1 {
                            println!("🔍 Monitoring cycle #{} for {} servers (20s interval)", CYCLE_COUNT, server_count);
                        }
                    }
                    Self::monitor_cycle(Arc::clone(&servers), Arc::clone(&rcon_manager), app_handle.clone()).await;
                } else {
                    // Only log this occasionally to avoid spam
                    unsafe {
                        CYCLE_COUNT += 1;
                        if CYCLE_COUNT % 6 == 0 { // Every 2 minutes when no servers
                            println!("⏳ No servers to monitor yet");
                        }
                    }
                }
            }
        });

        self.monitoring_task = Some(task);
        println!("🎯 Background server monitoring started");
    }

    /// Stop the background monitoring task
    pub fn stop_background_monitoring(&mut self) {
        if let Some(task) = self.monitoring_task.take() {
            task.abort();
            println!("Background server monitoring stopped");
        }
    }

    /// Single monitoring cycle - checks all servers
    async fn monitor_cycle(
        servers: Arc<RwLock<HashMap<String, ServerMonitorState>>>,
        rcon_manager: Arc<Mutex<RconManager>>,
        app_handle: Option<AppHandle>,
    ) {
        let server_list: Vec<(String, u16, ServerMonitorStatus)> = {
            let servers_read = servers.read().await;
            servers_read.iter()
                .map(|(name, state)| (name.clone(), state.port, state.status.clone()))
                .collect()
        };

        for (server_name, port, current_status) in server_list {
            // Get additional state data for smart transitions
            let (consecutive_failures, consecutive_successes) = {
                let servers_read = servers.read().await;
                if let Some(state) = servers_read.get(&server_name) {
                    (state.consecutive_failures, state.consecutive_successes)
                } else {
                    (0, 0)
                }
            };
            
            let (new_status, is_success) = Self::check_server_status_with_counters(
                &server_name,
                port,
                current_status,
                consecutive_failures,
                consecutive_successes,
                Arc::clone(&rcon_manager),
            ).await;

            // Update status and counters
            let mut servers_write = servers.write().await;
            if let Some(server_state) = servers_write.get_mut(&server_name) {
                let now = Instant::now();
                let time_since_last_change = now.duration_since(server_state.last_status_change);
                let should_change_status = server_state.status != new_status;
                
                // Update counters based on success/failure
                if is_success {
                    server_state.consecutive_successes += 1;
                    server_state.consecutive_failures = 0;
                    server_state.last_query_success = Some(now);
                } else {
                    server_state.consecutive_failures += 1;
                    server_state.consecutive_successes = 0;
                }
                
                if should_change_status {
                    // Prevent rapid status changes - require more time between changes for stability
                    // unless it's a manual command (starting/stopping)
                    let min_change_interval = if matches!(new_status, ServerMonitorStatus::Starting | ServerMonitorStatus::Stopping) {
                        Duration::from_secs(10)  // Allow transitions for manual commands but still require some stability
                    } else {
                        Duration::from_secs(30) // Require 30 seconds of stability for automatic transitions (was 15s)
                    };
                    
                    if time_since_last_change >= min_change_interval {
                        let old_status = server_state.status;
                        println!("🚀 Monitor detected stable change for {}: {:?} → {:?} (after {:.1}s, failures: {}, successes: {})", 
                                 server_name, old_status, new_status, time_since_last_change.as_secs_f32(),
                                 server_state.consecutive_failures, server_state.consecutive_successes);
                        
                        // Emit event to frontend
                        if let Some(ref app) = app_handle {
                            let event = ServerStatusEvent {
                                server_name: server_name.clone(),
                                old_status,
                                new_status,
                                timestamp: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64,
                            };
                            
                            match app.emit("server-status-changed", &event) {
                                Ok(_) => {
                                    println!("📡 ✅ Event SUCCESS: {} {:?} → {:?} (pure event-driven)", 
                                           server_name, old_status, new_status);
                                },
                                Err(e) => {
                                    println!("⚠️ ❌ CRITICAL EVENT FAILURE: {} - {}", server_name, e);
                                    println!("🛠️ Frontend will NOT update without events! server={}, old={:?}, new={:?}", 
                                           server_name, old_status, new_status);
                                }
                            }
                        }
                        
                        // Update status and reset counters on successful change
                        server_state.status = new_status;
                        server_state.last_status_change = now;
                        server_state.consecutive_failures = 0;
                        server_state.consecutive_successes = 0;
                    } else {
                        let time_remaining = min_change_interval.saturating_sub(time_since_last_change);
                        println!("🕰️ Suppressing status change for {} ({}s remaining for stability)", 
                                server_name, time_remaining.as_secs());
                    }
                }
                
                server_state.last_check = now;
            }
        }
    }

    /// Check individual server status using RCON and Query (with improved stability)
    async fn check_server_status_with_counters(
        server_name: &str,
        port: u16,
        current_status: ServerMonitorStatus,
        consecutive_failures: u32,
        consecutive_successes: u32,
        rcon_manager: Arc<Mutex<RconManager>>,
    ) -> (ServerMonitorStatus, bool) { // Returns (new_status, is_success)
        // Priority 1: Check RCON connection
        let rcon_connected = {
            let rcon = rcon_manager.lock().await;
            rcon.is_connected(server_name)
        };

        if rcon_connected {
            // RCON is connected = server is very likely online (RCON is more reliable than Query)
            if current_status != ServerMonitorStatus::Online {
                // RCON is reliable enough to immediately mark as online
                println!("✅ {} confirmed online via RCON", server_name);
            }
            return (ServerMonitorStatus::Online, true); // Success - RCON is trustworthy
        }

        // Priority 2: Check Query protocol
        let query_config = QueryConfig {
            host: "127.0.0.1".to_string(),
            port,
            timeout_ms: 2000, // 2 second timeout for monitoring
        };

        let query_service = QueryService::new(query_config);
        let query_response = query_service.ping_server().await;

        if query_response.online {
            // Query successful = server is potentially online
            // For conservative approach: require 2 consecutive successes to mark as online
            if current_status != ServerMonitorStatus::Online {
                if consecutive_successes >= 1 { // Require 2 total successes (previous + this one)
                    println!("✅ {} confirmed online via Query (after {} successes)", server_name, consecutive_successes + 1);
                    return (ServerMonitorStatus::Online, true);
                } else {
                    println!("🔍 {} responding to Query, but waiting for confirmation", server_name);
                    return (current_status, true); // Success but don't change state yet
                }
            } else {
                // Already online, just confirm it's still working
                return (ServerMonitorStatus::Online, true);
            }
        }

        // Both RCON and Query failed - this is a failure
        // Only log offline status when it changes to avoid spam
        if current_status == ServerMonitorStatus::Online || current_status == ServerMonitorStatus::Starting {
            println!("❌ {} connection failed - RCON: {}, Query: {} (failures: {})", 
                     server_name, rcon_connected, query_response.online, consecutive_failures + 1);
        }
        
        // Use very conservative transitions based on failure count and current state
        let new_status = match current_status {
            ServerMonitorStatus::Starting => {
                // Keep as starting - servers take time to fully initialize
                // Only transition to offline after many consecutive failures (60+ seconds)
                if consecutive_failures >= 3 { // 3 * 20s cycles = 60 seconds
                    println!("⏰ {} failed to start after {} attempts (60s), marking offline", server_name, consecutive_failures + 1);
                    ServerMonitorStatus::Offline
                } else {
                    ServerMonitorStatus::Starting
                }
            },
            ServerMonitorStatus::Stopping => {
                // If stopping and can't connect, it's now offline
                ServerMonitorStatus::Offline
            },
            ServerMonitorStatus::Online => {
                // VERY CONSERVATIVE: Require multiple consecutive failures before marking offline
                // This prevents flapping due to temporary network issues
                if consecutive_failures >= 3 { // 60+ seconds of failures (3 * 20s cycles)
                    println!("⬇️ {} going offline after {} consecutive failures (60s)", server_name, consecutive_failures + 1);
                    ServerMonitorStatus::Offline
                } else {
                    // Stay online, this might just be a temporary hiccup
                    ServerMonitorStatus::Online
                }
            },
            ServerMonitorStatus::Offline => {
                // Already offline, stay offline
                ServerMonitorStatus::Offline
            }
        };
        
        (new_status, false) // Failure
    }
}

impl Drop for ServerMonitor {
    fn drop(&mut self) {
        self.stop_background_monitoring();
    }
}