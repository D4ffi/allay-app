use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct RconLogger {
    server_name: String,
    log_path: PathBuf,
}

impl RconLogger {
    pub fn new(server_name: String) -> std::io::Result<Self> {
        let log_dir = PathBuf::from("storage").join("logs").join(&server_name);
        
        // Create logs directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)?;
        }
        
        let log_path = log_dir.join("rcon.log");
        
        Ok(Self {
            server_name,
            log_path,
        })
    }

    fn write_log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] [{}] {}\n", timestamp, level, message);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path) 
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }

    pub fn log_connection(&self, host: &str, port: u16) {
        self.write_log("CONNECTION", &format!("Connecting to RCON at {}:{}", host, port));
    }

    pub fn log_connection_success(&self) {
        self.write_log("CONNECTION", "RCON connection established successfully");
    }

    pub fn log_connection_failed(&self, error: &str) {
        self.write_log("ERROR", &format!("RCON connection failed: {}", error));
    }

    pub fn log_authentication(&self, password: &str) {
        // Don't log the actual password, just that authentication was attempted
        self.write_log("AUTH", &format!("Attempting authentication (password length: {})", password.len()));
    }

    pub fn log_authentication_success(&self) {
        self.write_log("AUTH", "RCON authentication successful");
    }

    pub fn log_authentication_failed(&self, error: &str) {
        self.write_log("ERROR", &format!("RCON authentication failed: {}", error));
    }

    pub fn log_command(&self, command: &str, is_heartbeat: bool) {
        let log_type = if is_heartbeat { "HEARTBEAT" } else { "COMMAND" };
        self.write_log(log_type, &format!("Executing: {}", command));
    }

    pub fn log_command_response(&self, command: &str, response: &str, is_heartbeat: bool) {
        let log_type = if is_heartbeat { "HEARTBEAT" } else { "COMMAND" };
        self.write_log(log_type, &format!("Response for '{}': {}", command, response));
    }

    pub fn log_command_error(&self, command: &str, error: &str, is_heartbeat: bool) {
        let log_type = if is_heartbeat { "HEARTBEAT_ERROR" } else { "COMMAND_ERROR" };
        self.write_log(log_type, &format!("Error executing '{}': {}", command, error));
    }

    pub fn log_heartbeat_start(&self) {
        self.write_log("HEARTBEAT", "Starting heartbeat system");
    }

    pub fn log_heartbeat_stop(&self) {
        self.write_log("HEARTBEAT", "Stopping heartbeat system");
    }

    pub fn log_disconnection(&self, reason: &str) {
        self.write_log("DISCONNECTION", &format!("RCON disconnected: {}", reason));
    }

    pub fn log_reconnection_attempt(&self, attempt: u32) {
        self.write_log("RECONNECTION", &format!("Attempting reconnection #{}", attempt));
    }

    pub fn log_reconnection_success(&self) {
        self.write_log("RECONNECTION", "Reconnection successful");
    }

    pub fn log_reconnection_failed(&self, error: &str) {
        self.write_log("ERROR", &format!("Reconnection failed: {}", error));
    }

    pub fn log_info(&self, message: &str) {
        self.write_log("INFO", message);
    }

    pub fn log_warning(&self, message: &str) {
        self.write_log("WARN", message);
    }

    pub fn log_error(&self, message: &str) {
        self.write_log("ERROR", message);
    }

    pub fn log_debug(&self, message: &str) {
        self.write_log("DEBUG", message);
    }

    // Rotate log file if it gets too large (> 10MB)
    pub fn rotate_if_needed(&self) -> std::io::Result<()> {
        if let Ok(metadata) = fs::metadata(&self.log_path) {
            if metadata.len() > 10 * 1024 * 1024 { // 10MB
                let backup_path = self.log_path.with_extension("log.bak");
                fs::rename(&self.log_path, backup_path)?;
                self.write_log("INFO", "Log file rotated");
            }
        }
        Ok(())
    }
}