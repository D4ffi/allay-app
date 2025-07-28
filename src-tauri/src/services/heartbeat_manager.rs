use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;
use crate::util::RconLogger;

#[derive(Debug, Clone)]
pub enum HeartbeatCommand {
    StartHeartbeat(String),    // server_name
    StopHeartbeat(String),     // server_name
    StopAll,
}

#[derive(Debug)]
struct HeartbeatTask {
    server_name: String,
    cancel_sender: tokio::sync::mpsc::Sender<()>,
}

pub struct HeartbeatManager {
    active_heartbeats: Arc<Mutex<HashMap<String, HeartbeatTask>>>,
    command_sender: mpsc::UnboundedSender<HeartbeatCommand>,
}

impl HeartbeatManager {
    pub fn new() -> Self {
        let active_heartbeats = Arc::new(Mutex::new(HashMap::new()));
        let (command_sender, mut command_receiver) = mpsc::unbounded_channel();

        let heartbeats_clone = active_heartbeats.clone();

        // Spawn the main heartbeat manager task
        tokio::spawn(async move {
            while let Some(command) = command_receiver.recv().await {
                match command {
                    HeartbeatCommand::StartHeartbeat(server_name) => {
                        Self::start_heartbeat_task(heartbeats_clone.clone(), server_name).await;
                    }
                    HeartbeatCommand::StopHeartbeat(server_name) => {
                        Self::stop_heartbeat_task(heartbeats_clone.clone(), &server_name).await;
                    }
                    HeartbeatCommand::StopAll => {
                        Self::stop_all_heartbeats(heartbeats_clone.clone()).await;
                    }
                }
            }
        });

        Self {
            active_heartbeats,
            command_sender,
        }
    }

    pub fn start_heartbeat(&self, server_name: String) {
        let _ = self.command_sender.send(HeartbeatCommand::StartHeartbeat(server_name));
    }

    pub fn stop_heartbeat(&self, server_name: String) {
        let _ = self.command_sender.send(HeartbeatCommand::StopHeartbeat(server_name));
    }

    pub fn stop_all(&self) {
        let _ = self.command_sender.send(HeartbeatCommand::StopAll);
    }

    async fn start_heartbeat_task(
        active_heartbeats: Arc<Mutex<HashMap<String, HeartbeatTask>>>,
        server_name: String,
    ) {
        // Stop existing heartbeat if running
        Self::stop_heartbeat_task(active_heartbeats.clone(), &server_name).await;

        let (cancel_sender, mut cancel_receiver) = tokio::sync::mpsc::channel(1);
        
        // Create logger for this server
        let logger = match RconLogger::new(server_name.clone()) {
            Ok(logger) => logger,
            Err(e) => {
                println!("Failed to create RCON logger for {}: {}", server_name, e);
                return;
            }
        };

        logger.log_heartbeat_start();

        let server_name_clone = server_name.clone();
        
        // Spawn the heartbeat task
        let heartbeat_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            let mut consecutive_failures = 0;
            let max_failures = 3;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform heartbeat
                        match Self::perform_heartbeat(&server_name_clone, &logger).await {
                            Ok(_) => {
                                consecutive_failures = 0;
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                logger.log_command_error("list (heartbeat)", &e, true);
                                
                                if consecutive_failures >= max_failures {
                                    logger.log_error(&format!(
                                        "Heartbeat failed {} consecutive times for {}. Attempting reconnection...", 
                                        consecutive_failures, server_name_clone
                                    ));
                                    
                                    // Attempt reconnection
                                    match Self::attempt_reconnection(&server_name_clone, &logger).await {
                                        Ok(_) => {
                                            consecutive_failures = 0;
                                            logger.log_reconnection_success();
                                        }
                                        Err(e) => {
                                            logger.log_reconnection_failed(&e);
                                            // Continue trying, don't break the loop
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ = cancel_receiver.recv() => {
                        logger.log_heartbeat_stop();
                        break;
                    }
                }
            }
        });

        // Store the task
        let task = HeartbeatTask {
            server_name: server_name.clone(),
            cancel_sender,
        };

        {
            let mut heartbeats = active_heartbeats.lock().await;
            heartbeats.insert(server_name, task);
        }
    }

    async fn stop_heartbeat_task(
        active_heartbeats: Arc<Mutex<HashMap<String, HeartbeatTask>>>,
        server_name: &str,
    ) {
        let mut heartbeats = active_heartbeats.lock().await;
        if let Some(task) = heartbeats.remove(server_name) {
            let _ = task.cancel_sender.send(()).await;
        }
    }

    async fn stop_all_heartbeats(
        active_heartbeats: Arc<Mutex<HashMap<String, HeartbeatTask>>>,
    ) {
        let mut heartbeats = active_heartbeats.lock().await;
        for (_, task) in heartbeats.drain() {
            let _ = task.cancel_sender.send(()).await;
        }
    }

    async fn perform_heartbeat(server_name: &str, logger: &RconLogger) -> Result<String, String> {
        
        let rcon_manager = crate::services::rcon_global::get_rcon_manager();
        match rcon_manager.execute_heartbeat_command(server_name) {
            Ok(response) => {
                Ok(response)
            }
            Err(e) => {
                let error_msg = e.to_string();
                Err(error_msg)
            }
        }
    }

    async fn attempt_reconnection(server_name: &str, logger: &RconLogger) -> Result<(), String> {
        logger.log_reconnection_attempt(1);
        
        let rcon_manager = crate::services::rcon_global::get_rcon_manager();
        match rcon_manager.connect(server_name) {
            Ok(_) => {
                logger.log_reconnection_success();
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                logger.log_reconnection_failed(&error_msg);
                Err(error_msg)
            }
        }
    }

    pub async fn is_heartbeat_active(&self, server_name: &str) -> bool {
        let heartbeats = self.active_heartbeats.lock().await;
        heartbeats.contains_key(server_name)
    }

    pub async fn get_active_heartbeats(&self) -> Vec<String> {
        let heartbeats = self.active_heartbeats.lock().await;
        heartbeats.keys().cloned().collect()
    }
}

impl Drop for HeartbeatManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}