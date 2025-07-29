use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use super::rcon_service::{RconConnection, RconError};
use crate::util::{ServerPropertiesManager, RconLogger};

#[derive(Debug, Clone)]
pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Clone)]
struct FailureTracker {
    consecutive_failures: u32,
    last_failure_time: Option<Instant>,
    total_failures: u32,
}

impl Default for RconConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 25575,
            password: "".to_string(),
        }
    }
}

impl Default for FailureTracker {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            last_failure_time: None,
            total_failures: 0,
        }
    }
}

pub struct RconManager {
    connections: Arc<Mutex<HashMap<String, RconConnection>>>,
    configs: Arc<Mutex<HashMap<String, RconConfig>>>,
    last_connect_attempts: Arc<Mutex<HashMap<String, Instant>>>,
    failure_trackers: Arc<Mutex<HashMap<String, FailureTracker>>>,
    loggers: Arc<Mutex<HashMap<String, RconLogger>>>,
}

impl RconManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            configs: Arc::new(Mutex::new(HashMap::new())),
            last_connect_attempts: Arc::new(Mutex::new(HashMap::new())),
            failure_trackers: Arc::new(Mutex::new(HashMap::new())),
            loggers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_server(&self, server_name: String, config: RconConfig) {
        let mut configs = self.configs.lock().unwrap();
        configs.insert(server_name, config);
    }

    fn get_logger(&self, server_name: &str) -> Option<RconLogger> {
        let mut loggers = self.loggers.lock().unwrap();
        
        if !loggers.contains_key(server_name) {
            match RconLogger::new(server_name.to_string()) {
                Ok(logger) => {
                    loggers.insert(server_name.to_string(), logger);
                }
                Err(e) => {
                    println!("Failed to create logger for {}: {}", server_name, e);
                    return None;
                }
            }
        }
        
        loggers.get(server_name).cloned()
    }

    pub fn remove_server(&self, server_name: &str) {
        // Stop heartbeat first
        crate::services::rcon_global::get_heartbeat_manager().stop_heartbeat(server_name.to_string());
        
        // Remove from configs
        {
            let mut configs = self.configs.lock().unwrap();
            configs.remove(server_name);
        }
        
        // Remove failure tracker
        {
            let mut trackers = self.failure_trackers.lock().unwrap();
            trackers.remove(server_name);
        }
        
        // Remove logger
        {
            let mut loggers = self.loggers.lock().unwrap();
            if let Some(logger) = loggers.remove(server_name) {
                logger.log_info("Server removed from RCON manager");
            }
        }
        
        // Disconnect and remove from connections
        self.remove_connection(server_name);
    }

    pub fn is_connected(&self, server_name: &str) -> bool {
        let connections = self.connections.lock().unwrap();
        connections.get(server_name)
            .map(|conn| conn.is_connected())
            .unwrap_or(false)
    }

    pub fn connect(&self, server_name: &str) -> Result<(), RconError> {
        let logger = self.get_logger(server_name);
        
        if let Some(ref logger) = logger {
        }

        let config = {
            let configs = self.configs.lock().unwrap();
            let config = configs.get(server_name)
                .ok_or_else(|| {
                    if let Some(ref logger) = logger {
                        logger.log_error(&format!("No config found for '{}'", server_name));
                    }
                    RconError::ConnectionFailed("Server not configured".to_string())
                })?
                .clone();
            
            if let Some(ref logger) = logger {
            }
            config
        };

        if let Some(ref logger) = logger {
            logger.log_connection(&config.host, config.port);
        }

        // Check if we already have a connected instance
        {
            let connections = self.connections.lock().unwrap();
            if let Some(connection) = connections.get(server_name) {
                if connection.is_connected() {
                    if let Some(ref logger) = logger {
                    }
                    return Ok(());
                }
            }
        }

        // Remove any existing connection and create fresh one
        {
            let mut connections = self.connections.lock().unwrap();
            if let Some(mut existing_connection) = connections.remove(server_name) {
                existing_connection.disconnect();
                if let Some(ref logger) = logger {
                    logger.log_disconnection("Removed stale connection");
                }
            }
        }

        let mut new_connection = RconConnection::new(
            config.host.clone(),
            config.port,
            config.password.clone(),
        );

        match new_connection.connect() {
            Ok(_) => {
                if let Some(ref logger) = logger {
                    logger.log_connection_success();
                }
                
                // Store the connection
                {
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(server_name.to_string(), new_connection);
                }
                
                // Start heartbeat for this server
                crate::services::rcon_global::get_heartbeat_manager().start_heartbeat(server_name.to_string());
                
                Ok(())
            }
            Err(e) => {
                if let Some(ref logger) = logger {
                    logger.log_connection_failed(&e.to_string());
                }
                Err(e)
            }
        }
    }

    pub fn disconnect(&self, server_name: &str) {
        // Stop heartbeat first
        crate::services::rcon_global::get_heartbeat_manager().stop_heartbeat(server_name.to_string());
        
        let logger = self.get_logger(server_name);
        
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get_mut(server_name) {
            connection.disconnect();
            if let Some(ref logger) = logger {
                logger.log_disconnection("Manual disconnection requested");
            }
        }
    }

    pub fn remove_connection(&self, server_name: &str) {
        let mut connections = self.connections.lock().unwrap();
        if let Some(mut connection) = connections.remove(server_name) {
            connection.disconnect();
        }
    }

    fn record_failure(&self, server_name: &str) {
        let mut trackers = self.failure_trackers.lock().unwrap();
        let tracker = trackers.entry(server_name.to_string()).or_default();
        
        tracker.consecutive_failures += 1;
        tracker.total_failures += 1;
        tracker.last_failure_time = Some(Instant::now());
        
    }

    fn record_success(&self, server_name: &str) {
        let mut trackers = self.failure_trackers.lock().unwrap();
        if let Some(tracker) = trackers.get_mut(server_name) {
            if tracker.consecutive_failures > 0 {
            }
            tracker.consecutive_failures = 0;
            tracker.last_failure_time = None;
        }
    }

    fn get_adaptive_delay(&self, server_name: &str) -> Duration {
        let trackers = self.failure_trackers.lock().unwrap();
        if let Some(tracker) = trackers.get(server_name) {
            let base_delay = Duration::from_millis(200);
            
            match tracker.consecutive_failures {
                0..=1 => base_delay,
                2..=4 => Duration::from_millis(1000),
                _ => Duration::from_millis(2000),
            }
        } else {
            Duration::from_millis(200)
        }
    }

    pub fn execute_command(&self, server_name: &str, command: &str) -> Result<String, RconError> {
        let logger = self.get_logger(server_name);
        
        if let Some(ref logger) = logger {
            logger.log_command(command, false);
        }
        
        // Auto-configure RCON if not configured
        self.ensure_server_configured(server_name);
        
        // Ensure we have a connection
        if !self.is_connected(server_name) {
            match self.connect(server_name) {
                Ok(_) => {},
                Err(e) => {
                    if let Some(ref logger) = logger {
                        logger.log_command_error(command, &e.to_string(), false);
                    }
                    return Err(e);
                }
            }
        }
        
        // Execute command on persistent connection
        let result = {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(server_name)
                .ok_or_else(|| {
                    RconError::ConnectionFailed("No connection available".to_string())
                })?;
            
            connection.send_command(command)
        };
        
        match &result {
            Ok(response) => {
                if let Some(ref logger) = logger {
                    logger.log_command_response(command, response, false);
                }
                self.record_success(server_name);
            },
            Err(e) => {
                if let Some(ref logger) = logger {
                    logger.log_command_error(command, &e.to_string(), false);
                }
                self.record_failure(server_name);
                
                // If command failed, the connection might be broken - let heartbeat handle reconnection
                // Or try reconnecting immediately for user commands
                if self.is_retryable_error(e) {
                    if let Some(ref logger) = logger {
                        logger.log_info("Command failed, attempting immediate reconnection...");
                    }
                    
                    // Try reconnecting once
                    match self.connect(server_name) {
                        Ok(_) => {
                            // Try command again
                            let retry_result = {
                                let mut connections = self.connections.lock().unwrap();
                                if let Some(connection) = connections.get_mut(server_name) {
                                    connection.send_command(command)
                                } else {
                                    return result; // Return original error
                                }
                            };
                            
                            match &retry_result {
                                Ok(response) => {
                                    if let Some(ref logger) = logger {
                                        logger.log_command_response(command, response, false);
                                        logger.log_info("Command succeeded after reconnection");
                                    }
                                    self.record_success(server_name);
                                    return retry_result;
                                }
                                Err(retry_error) => {
                                    if let Some(ref logger) = logger {
                                        logger.log_command_error(command, &retry_error.to_string(), false);
                                        logger.log_warning("Command failed even after reconnection");
                                    }
                                }
                            }
                        }
                        Err(reconnect_error) => {
                            if let Some(ref logger) = logger {
                                logger.log_error(&format!("Reconnection failed: {}", reconnect_error));
                            }
                        }
                    }
                }
            }
        }
        
        result
    }

    // Method specifically for heartbeat commands (called by HeartbeatManager)
    pub fn execute_heartbeat_command(&self, server_name: &str) -> Result<String, RconError> {
        let logger = self.get_logger(server_name);
        
        if let Some(ref logger) = logger {
            logger.log_command("list", true);
        }
        
        // Don't auto-configure for heartbeat - if server isn't configured, skip heartbeat
        let connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get(server_name) {
            if !connection.is_connected() {
                drop(connections); // Release lock before logging
                if let Some(ref logger) = logger {
                }
                return Err(RconError::NotConnected);
            }
        } else {
            drop(connections); // Release lock before logging
            if let Some(ref logger) = logger {
            }
            return Err(RconError::NotConnected);
        }
        drop(connections); // Release lock
        
        // Execute heartbeat command
        let result = {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(server_name)
                .ok_or(RconError::NotConnected)?;
            
            connection.send_command("list")
        };
        
        match &result {
            Ok(response) => {
                if let Some(ref logger) = logger {
                    logger.log_command_response("list", response, true);
                }
                self.record_success(server_name);
            },
            Err(e) => {
                if let Some(ref logger) = logger {
                    logger.log_command_error("list", &e.to_string(), true);
                }
                self.record_failure(server_name);
            }
        }
        
        result
    }


    fn is_retryable_error(&self, error: &RconError) -> bool {
        match error {
            RconError::BufferError(_) => true,           // Always retry buffer errors
            RconError::ServerClosedConnection => true,   // Always retry server disconnections
            RconError::NetworkTimeout => true,          // Always retry timeouts
            RconError::ConnectionFailed(_) => true,     // Always retry connection failures
            RconError::InvalidResponse => true,         // Retry invalid responses
            RconError::NotConnected => true,            // Retry not connected
            RconError::CommandFailed(msg) => {
                // Legacy fallback for unclassified errors
                msg.contains("failed to fill whole buffer") || 
                msg.contains("Connection closed by server") ||
                msg.contains("Connection reset")
            },
            RconError::AuthenticationFailed => false,   // Never retry auth failures
        }
    }

    fn ensure_server_configured(&self, server_name: &str) {
        let configs = self.configs.lock().unwrap();
        if !configs.contains_key(server_name) {
            drop(configs);
            
            // Read password from server.properties
            let server_path = PathBuf::from("storage").join(server_name);
            let properties_path = server_path.join("server.properties");
            
            let password = if properties_path.exists() {
                let properties_manager = ServerPropertiesManager::new(properties_path);
                match properties_manager.get_property("rcon.password") {
                    Ok(pwd) if !pwd.is_empty() => {
                        pwd
                    },
                    _ => {
                        "minecraft".to_string()
                    }
                }
            } else {
                "minecraft".to_string()
            };
            
            let config = RconConfig {
                host: "127.0.0.1".to_string(),
                port: 25575,
                password,
            };
            
            self.add_server(server_name.to_string(), config);
        } else {
        }
    }

    pub fn test_connection(&self, server_name: &str) -> Result<bool, RconError> {
        match self.execute_command(server_name, "list") {
            Ok(_) => Ok(true),
            Err(RconError::AuthenticationFailed) => Ok(false),
            Err(RconError::ConnectionFailed(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn get_connected_servers(&self) -> Vec<String> {
        let connections = self.connections.lock().unwrap();
        connections.iter()
            .filter(|(_, conn)| conn.is_connected())
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn heartbeat_all(&self) {
        // No heartbeat needed for ephemeral connections
    }

    /// Handle server going offline - automatically disconnect RCON and stop heartbeat
    pub fn handle_server_offline(&self, server_name: &str) {
        // Stop heartbeat first
        crate::services::rcon_global::get_heartbeat_manager().stop_heartbeat(server_name.to_string());
        
        // Disconnect RCON connection
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.remove(server_name) {
            drop(connection); // This will call disconnect in the Drop implementation
            
            // Log the automatic disconnection
            let loggers = self.loggers.lock().unwrap();
            if let Some(logger) = loggers.get(server_name) {
                logger.log_disconnection("Automatic disconnection - server went offline");
            }
            
            println!("🔌 RCON automatically disconnected for offline server: {}", server_name);
        }
    }

    pub fn disconnect_all(&self) {
        let mut connections = self.connections.lock().unwrap();
        for (_, connection) in connections.iter_mut() {
            connection.disconnect();
        }
        connections.clear();
    }
}

impl Default for RconManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RconManager {
    fn drop(&mut self) {
        self.disconnect_all();
    }
}