// Declaración de módulos
mod models;
mod util;
mod services;

use std::path::PathBuf;
use util::{ServerFileManager, ServerInstance, ServerCreationStatus, JarCacheManager, CacheStats, ServerPropertiesManager};
use services::version_manager::{VersionManager, VersionSummary};
use services::unified_server_service::UnifiedServerService;
use services::rcon_manager::{RconManager, RconConfig};
use services::simple_rcon_monitor::{SimpleRconMonitor, ServerStatus};
use models::version::{LoaderType, VersionResponse};
use models::query::{QueryResponse, QueryConfig};
use services::query_service::QueryService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;


// Global unified server service
lazy_static::lazy_static! {
    static ref UNIFIED_SERVER_SERVICE: Arc<Mutex<UnifiedServerService>> = {
        let service = UnifiedServerService::new().expect("Failed to initialize UnifiedServerService");
        Arc::new(Mutex::new(service))
    };
    
    static ref RCON_MANAGER: Arc<Mutex<RconManager>> = {
        Arc::new(Mutex::new(RconManager::new()))
    };

    static ref SERVER_MONITOR: Arc<Mutex<SimpleRconMonitor>> = {
        let monitor = SimpleRconMonitor::new(Arc::clone(&RCON_MANAGER));
        Arc::new(Mutex::new(monitor))
    };
    
    static ref MONITORING_INITIALIZED: Arc<Mutex<bool>> = {
        Arc::new(Mutex::new(false))
    };
}

// Helper functions for common operations
fn parse_loader_type(loader: &str) -> Result<LoaderType, String> {
    match loader {
        "vanilla" => Ok(LoaderType::Vanilla),
        "fabric" => Ok(LoaderType::Fabric),
        "forge" => Ok(LoaderType::Forge),
        "neoforge" => Ok(LoaderType::NeoForge),
        "paper" => Ok(LoaderType::Paper),
        "quilt" => Ok(LoaderType::Quilt),
        _ => Err(format!("Invalid loader type: {}", loader)),
    }
}

fn get_storage_path(server_name: &str) -> PathBuf {
    PathBuf::from("storage").join(server_name)
}

fn create_version_manager() -> Result<VersionManager, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    VersionManager::new(cache_dir).map_err(|e| e.to_string())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_server_instance(
    name: String,
    version: String,
    mod_loader: String,
    mod_loader_version: String,
) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let storage_path = PathBuf::from("storage");
    
    let manager = ServerFileManager::new(config_path);
    
    // Initialize a config file if it doesn't exist or is empty
    manager.initialize_config().map_err(|e| e.to_string())?;
    
    let instance = ServerInstance::new(
        name.clone(),
        version,
        mod_loader,
        mod_loader_version,
        &storage_path,
    ).map_err(|e| e.to_string())?;
    
    manager.add_instance(instance).map_err(|e| e.to_string())?;
    manager.create_storage_directory(&name, &storage_path).map_err(|e| e.to_string())?;
    
    Ok(format!("Server instance '{}' created successfully", name))
}

#[tauri::command]
fn get_all_server_instances() -> Result<Vec<ServerInstance>, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    // Initialize a config file if it doesn't exist or is empty
    manager.initialize_config().map_err(|e| e.to_string())?;
    
    manager.get_all_instances().map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_server_instance(name: String) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    manager.remove_instance(&name).map_err(|e| e.to_string())?;
    
    Ok(format!("Server instance '{}' removed successfully", name))
}

#[tauri::command]
fn delete_server_completely(name: String) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let storage_path = PathBuf::from("storage");
    let manager = ServerFileManager::new(config_path);
    
    manager.remove_instance_with_storage(&name, &storage_path).map_err(|e| e.to_string())?;
    
    Ok(format!("Server instance '{}' and its files deleted successfully", name))
}

#[tauri::command]
fn update_server_description(name: String, description: String) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    // Get the current instance
    let mut instance = manager.get_instance(&name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Server instance '{}' not found", name))?;
    
    // Update description
    instance.description = if description.trim().is_empty() {
        None
    } else {
        Some(description.trim().to_string())
    };
    
    // Save the updated instance
    manager.update_instance(&name, instance).map_err(|e| e.to_string())?;
    
    Ok(format!("Server '{}' description updated successfully", name))
}

#[tauri::command]
fn update_server_memory(name: String, memory_mb: u32) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    // Get the current instance
    let mut instance = manager.get_instance(&name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Server instance '{}' not found", name))?;
    
    // Update memory
    instance.memory_mb = memory_mb;
    
    // For Forge servers, also update user_jvm_args.txt
    if instance.mod_loader == "forge" {
        update_forge_jvm_args(&instance.storage_path, memory_mb)
            .map_err(|e| e.to_string())?;
    }
    
    // Save the updated instance
    manager.update_instance(&name, instance).map_err(|e| e.to_string())?;
    
    Ok(format!("Server '{}' memory updated to {}MB successfully", name, memory_mb))
}

fn update_forge_jvm_args(server_path: &PathBuf, memory_mb: u32) -> Result<(), std::io::Error> {
    let jvm_args_path = server_path.join("user_jvm_args.txt");
    
    // Convert MB to GB for JVM args
    let memory_gb = memory_mb / 1024;
    let memory_arg = format!("-Xmx{}G", memory_gb);
    
    if jvm_args_path.exists() {
        // Read existing content
        let content = std::fs::read_to_string(&jvm_args_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        // Find and replace existing -Xmx argument or add to the end
        let mut found = false;
        for line in &mut lines {
            if line.trim().starts_with("-Xmx") {
                *line = memory_arg.clone();
                found = true;
                break;
            }
        }
        
        if !found {
            lines.push(memory_arg);
        }
        
        // Write back to the file
        let new_content = lines.join("\n");
        std::fs::write(&jvm_args_path, new_content)?;
    } else {
        // Create a new file with a memory argument
        std::fs::write(&jvm_args_path, memory_arg)?;
    }
    
    Ok(())
}

// Version management commands
#[tauri::command]
async fn get_minecraft_versions(loader: String, force_refresh: bool, minecraft_version: Option<String>) -> Result<VersionResponse, String> {
    let manager = create_version_manager()?;
    let loader_type = parse_loader_type(&loader)?;
    
    manager.get_versions_for_minecraft(loader_type, force_refresh, minecraft_version).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_minecraft_versions(force_refresh: bool) -> Result<HashMap<String, VersionResponse>, String> {
    let manager = create_version_manager()?;
    manager.get_all_versions(force_refresh).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_version_summary() -> Result<VersionSummary, String> {
    let manager = create_version_manager()?;
    manager.get_version_summary().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_version_cache(loader: Option<String>) -> Result<HashMap<String, bool>, String> {
    let manager = create_version_manager()?;
    
    let loader_type = if let Some(loader) = loader {
        Some(parse_loader_type(&loader)?)
    } else {
        None
    };
    
    manager.refresh_cache(loader_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_version_cache(loader: Option<String>) -> Result<String, String> {
    let manager = create_version_manager()?;
    
    if let Some(loader) = loader {
        let loader_type = parse_loader_type(&loader)?;
        manager.clear_cache(&loader_type).map_err(|e| e.to_string())?;
        Ok(format!("Cache cleared for {}", loader))
    } else {
        manager.clear_all_cache().map_err(|e| e.to_string())?;
        Ok("All version cache cleared".to_string())
    }
}

#[tauri::command]
async fn download_server_jar(
    server_name: String,
    loader: String,
    minecraft_version: String,
    loader_version: Option<String>,
) -> Result<String, String> {
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    let storage_path = get_storage_path(&server_name);
    let loader_type = parse_loader_type(&loader)?;
    
    match service.download_server_jar(
        loader_type,
        minecraft_version,
        loader_version,
        storage_path,
    ).await {
        Ok(jar_path) => Ok(format!("Server JAR downloaded successfully to: {:?}", jar_path)),
        Err(e) => Err(format!("Failed to download server JAR: {}", e)),
    }
}

#[tauri::command]
async fn setup_server(
    server_name: String,
    loader: String,
    minecraft_version: String,
    loader_version: Option<String>,
) -> Result<String, String> {
    let storage_path = get_storage_path(&server_name);
    let loader_type = parse_loader_type(&loader)?;
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    
    match service.setup_server(
        &server_name,
        loader_type,
        &minecraft_version,
        loader_version.as_deref(),
        &storage_path,
    ).await {
        Ok(_) => Ok(format!("Server '{}' setup completed successfully", server_name)),
        Err(e) => Err(format!("Failed to setup server '{}': {}", server_name, e)),
    }
}

#[tauri::command]
async fn start_server(server_name: String, loader: String) -> Result<String, String> {
    let storage_path = get_storage_path(&server_name);
    let loader_type = parse_loader_type(&loader)?;
    
    // Get server memory configuration
    let config_path = PathBuf::from("storage/server_config.json");
    let file_manager = ServerFileManager::new(config_path);
    let memory_mb = match file_manager.get_instance(&server_name) {
        Ok(Some(instance)) => instance.memory_mb,
        _ => 2048, // Default 2GB if not found
    };
    
    // Start RCON monitoring for this server
    {
        let monitor = SERVER_MONITOR.lock().await;
        monitor.start_monitoring(server_name.clone()).await;
    }
    
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    
    match service.start_server(&server_name, &storage_path, loader_type, memory_mb).await {
        Ok(_) => {
            // Server process started successfully
            // Monitoring will detect when it's actually responding and update to online
            println!("Server '{}' process started, monitoring will detect when fully online", server_name);
            Ok(format!("Server '{}' started successfully", server_name))
        },
        Err(e) => {
            // Failed to start process, stop monitoring
            let monitor = SERVER_MONITOR.lock().await;
            monitor.stop_monitoring(&server_name).await;
            Err(format!("Failed to start server '{}': {}", server_name, e))
        }
    }
}

#[tauri::command]
async fn stop_server(server_name: String) -> Result<String, String> {
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    
    match service.stop_server(&server_name).await {
        Ok(_) => {
            // Server stopped, stop monitoring
            {
                let monitor = SERVER_MONITOR.lock().await;
                monitor.stop_monitoring(&server_name).await;
            }
            Ok(format!("Server '{}' stopped successfully", server_name))
        },
        Err(e) => {
            // Failed to stop, let monitoring detect the actual state
            Err(format!("Failed to stop server '{}': {}", server_name, e))
        }
    }
}

#[tauri::command]
async fn toggle_server(server_name: String, loader: String) -> Result<String, String> {
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    let is_running = service.is_server_running(&server_name).await;
    
    if is_running {
        // Stop the server
        match service.stop_server(&server_name).await {
            Ok(_) => Ok(format!("Server '{}' stopped successfully", server_name)),
            Err(e) => Err(format!("Failed to stop server '{}': {}", server_name, e)),
        }
    } else {
        // Start the server
        let storage_path = get_storage_path(&server_name);
        let loader_type = parse_loader_type(&loader)?;
        
        // Get memory allocation from the server file manager
        let config_path = PathBuf::from("storage/server_config.json");
        let file_manager = ServerFileManager::new(config_path);
        let memory_mb = file_manager.get_server_memory(&server_name)
            .unwrap_or(2048); // Default to 2GB if not found
        
        match service.start_server(&server_name, &storage_path, loader_type, memory_mb).await {
            Ok(_) => Ok(format!("Server '{}' started successfully", server_name)),
            Err(e) => Err(format!("Failed to start server '{}': {}", server_name, e)),
        }
    }
}

#[tauri::command]
fn get_server_loader_type(server_name: String) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    match manager.get_instance(&server_name) {
        Ok(Some(server)) => Ok(server.mod_loader.clone()),
        Ok(None) => Err(format!("Server '{}' not found", server_name)),
        Err(e) => Err(format!("Failed to get server info: {}", e)),
    }
}

#[tauri::command]
async fn is_server_running(server_name: String) -> bool {
    let monitor = SERVER_MONITOR.lock().await;
    let status = monitor.get_server_status(&server_name).await;
    
    match status {
        ServerStatus::Online => true,
        _ => false,
    }
}

fn get_server_port(server_name: &str) -> Option<u16> {
    // Try to read port from server.properties
    let storage_path = std::path::PathBuf::from("storage").join(server_name);
    let properties_path = storage_path.join("server.properties");
    
    if properties_path.exists() {
        let properties_manager = ServerPropertiesManager::new(properties_path);
        
        if let Ok(port_str) = properties_manager.get_property("server-port") {
            if let Ok(port) = port_str.parse::<u16>() {
                return Some(port);
            }
        }
    }
    
    None // Return None to use the default port
}

#[tauri::command]
async fn query_server_status(server_name: String) -> Result<QueryResponse, String> {
    let port = get_server_port(&server_name).unwrap_or(25565);
    
    let config = QueryConfig {
        host: "127.0.0.1".to_string(),
        port,
        timeout_ms: 5000, // 5 second timeout for detailed query
    };
    
    let query_service = QueryService::new(config);
    let response = query_service.query_server().await;
    
    Ok(response)
}

#[tauri::command]
async fn start_server_monitoring(server_name: String) -> Result<String, String> {
    let monitor = SERVER_MONITOR.lock().await;
    monitor.start_monitoring(server_name.clone()).await;
    Ok(format!("Started RCON monitoring for server: {}", server_name))
}

#[tauri::command]
async fn stop_server_monitoring(server_name: String) -> Result<String, String> {
    let monitor = SERVER_MONITOR.lock().await;
    monitor.stop_monitoring(&server_name).await;
    Ok(format!("Stopped RCON monitoring for server: {}", server_name))
}

#[tauri::command]
async fn get_server_monitor_status(server_name: String) -> String {
    let monitor = SERVER_MONITOR.lock().await;
    let status = monitor.get_server_status(&server_name).await;
    
    match status {
        ServerStatus::Offline => "offline".to_string(),
        ServerStatus::Online => "online".to_string(),
    }
}

#[tauri::command]
async fn get_all_server_monitor_statuses() -> HashMap<String, String> {
    let monitor = SERVER_MONITOR.lock().await;
    let all_statuses = monitor.get_all_statuses().await;
    
    all_statuses.into_iter()
        .map(|(name, status)| {
            let status_str = match status {
                ServerStatus::Offline => "offline".to_string(),
                ServerStatus::Online => "online".to_string(),
            };
            (name, status_str)
        })
        .collect()
}

#[tauri::command]
async fn update_server_monitor_status(server_name: String, status: String) -> Result<String, String> {
    let monitor_status = match status.as_str() {
        "offline" => ServerStatus::Offline,
        "online" => ServerStatus::Online,
        _ => return Err("Invalid status (only 'offline' and 'online' supported)".to_string()),
    };
    
    let monitor = SERVER_MONITOR.lock().await;
    monitor.update_server_status(&server_name, monitor_status).await;
    Ok(format!("Updated server {} status to: {}", server_name, status))
}

#[tauri::command]
async fn get_running_servers() -> Vec<String> {
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    service.get_running_servers().await
}

// JAR Cache management commands
#[tauri::command]
fn get_jar_cache_stats() -> Result<CacheStats, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let jar_cache = JarCacheManager::new(cache_dir).map_err(|e| e.to_string())?;
    jar_cache.get_cache_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_jar_cache(
    loader: Option<String>,
    minecraft_version: Option<String>,
    loader_version: Option<String>,
) -> Result<String, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let jar_cache = JarCacheManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    if let (Some(loader_str), Some(mc_version)) = (loader, minecraft_version) {
        let loader_type = parse_loader_type(&loader_str)?;
        
        jar_cache.clear_jar_cache(&loader_type, &mc_version, loader_version.as_deref())
            .map_err(|e| e.to_string())?;
        Ok(format!("Cleared JAR cache for {} {}", loader_str, mc_version))
    } else {
        jar_cache.clear_all_jar_cache().map_err(|e| e.to_string())?;
        Ok("Cleared all JAR cache".to_string())
    }
}

#[tauri::command]
fn is_jar_cached(
    loader: String,
    minecraft_version: String,
    loader_version: Option<String>,
) -> Result<bool, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let jar_cache = JarCacheManager::new(cache_dir).map_err(|e| e.to_string())?;
    let loader_type = parse_loader_type(&loader)?;
    
    Ok(jar_cache.is_jar_cached(&loader_type, &minecraft_version, loader_version.as_deref()))
}

// Server properties management commands
#[tauri::command]
fn get_server_motd(server_name: String) -> Result<String, String> {
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    if !properties_path.exists() {
        return Ok("A Minecraft Server".to_string()); // Default MOTD
    }
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    match properties_manager.get_property("motd") {
        Ok(motd) => Ok(motd),
        Err(_) => Ok("A Minecraft Server".to_string()), // Default MOTD if reading fails
    }
}

#[tauri::command]
fn get_server_max_players(server_name: String) -> Result<u32, String> {
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    if !properties_path.exists() {
        return Ok(20); // Default max players
    }
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    match properties_manager.get_property("max-players") {
        Ok(max_players_str) => {
            match max_players_str.parse::<u32>() {
                Ok(max_players) => Ok(max_players),
                Err(_) => Ok(20), // Default if parsing fails
            }
        },
        Err(_) => Ok(20), // Default max players if property not found
    }
}

#[tauri::command]
fn update_server_property(
    server_name: String,
    property_key: String,
    property_value: String,
) -> Result<String, String> {
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    properties_manager.update_property(&property_key, &property_value)
        .map_err(|e| e.to_string())?;
    
    Ok(format!("Updated {} to {}", property_key, property_value))
}

// System information commands
#[tauri::command]
async fn create_server_transactional(
    name: String,
    version: String,
    mod_loader: String,
    mod_loader_version: String,
) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let storage_path = PathBuf::from("storage");
    let manager = ServerFileManager::new(config_path);
    
    // Initialize a config file if it doesn't exist
    manager.initialize_config().map_err(|e| e.to_string())?;
    
    // Check if a server already exists
    if manager.instance_exists(&name).map_err(|e| e.to_string())? {
        return Err(format!("Server instance '{}' already exists", name));
    }
    
    println!("Starting transactional server creation for: {}", name);
    
    // Step 1: Create a server instance with PENDING status
    let instance = ServerInstance::new(
        name.clone(),
        version.clone(),
        mod_loader.clone(),
        mod_loader_version.clone(),
        &storage_path,
    ).map_err(|e| e.to_string())?;
    
    manager.add_instance(instance).map_err(|e| e.to_string())?;
    manager.create_storage_directory(&name, &storage_path).map_err(|e| {
        // If directory creation fails, remove from config
        let _ = manager.remove_instance(&name);
        e.to_string()
    })?;
    
    println!("Server instance created with PENDING status");
    
    // Step 2: Download server JAR
    let loader_type = match parse_loader_type(&mod_loader) {
        Ok(loader_type) => loader_type,
        Err(e) => {
            // Cleanup on invalid loader
            let _ = manager.remove_instance_with_storage(&name, &storage_path);
            return Err(e);
        }
    };
    
    let service = UNIFIED_SERVER_SERVICE.lock().await;
    let server_storage_path = get_storage_path(&name);
    
    // Prepare loader version reference
    let loader_version_ref = if mod_loader != "vanilla" { 
        Some(mod_loader_version.as_str()) 
    } else { 
        None 
    };
    
    // Download JAR with rollback on failure
    match service.download_server_jar(
        loader_type.clone(),
        version.clone(),
        loader_version_ref.map(|s| s.to_string()),
        server_storage_path.clone(),
    ).await {
        Ok(_) => {
            // Update status to JAR_DOWNLOADED
            manager.update_server_status(&name, ServerCreationStatus::JarDownloaded)
                .map_err(|e| e.to_string())?;
            println!("JAR downloaded successfully, status updated to JAR_DOWNLOADED");
        },
        Err(e) => {
            // Rollback: mark as failed and cleanup
            let _ = manager.update_server_status(&name, ServerCreationStatus::Failed);
            let _ = manager.remove_instance_with_storage(&name, &storage_path);
            return Err(format!("Failed to download server JAR: {}", e));
        }
    }
    
    // Step 3: Setup server with rollback on failure
    match service.setup_server(
        &name,
        loader_type,
        &version,
        loader_version_ref,
        &server_storage_path,
    ).await {
        Ok(_) => {
            // Update status to SETUP_COMPLETE
            manager.update_server_status(&name, ServerCreationStatus::SetupComplete)
                .map_err(|e| e.to_string())?;
            println!("Server setup completed, status updated to SETUP_COMPLETE");
        },
        Err(e) => {
            // Rollback: mark as failed and cleanup
            let _ = manager.update_server_status(&name, ServerCreationStatus::Failed);
            let _ = manager.remove_instance_with_storage(&name, &storage_path);
            return Err(format!("Failed to setup server: {}", e));
        }
    }
    
    // Step 4: Mark as completed
    manager.update_server_status(&name, ServerCreationStatus::Completed)
        .map_err(|e| e.to_string())?;
    
    println!("Server '{}' created successfully with COMPLETED status", name);
    
    Ok(format!("Server instance '{}' created successfully", name))
}

#[tauri::command]
fn cleanup_incomplete_servers() -> Result<Vec<String>, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let storage_path = PathBuf::from("storage");
    let manager = ServerFileManager::new(config_path);
    
    // Initialize config if needed
    manager.initialize_config().map_err(|e| e.to_string())?;
    
    // Get incomplete servers
    let incomplete_servers = manager.get_incomplete_servers(&storage_path)
        .map_err(|e| e.to_string())?;
    
    let mut cleaned_servers = Vec::new();
    
    for server_name in incomplete_servers {
        match manager.cleanup_incomplete_server(&server_name, &storage_path) {
            Ok(_) => {
                println!("Cleaned up incomplete server: {}", server_name);
                cleaned_servers.push(server_name);
            },
            Err(e) => {
                println!("Failed to cleanup server {}: {}", server_name, e);
            }
        }
    }
    
    Ok(cleaned_servers)
}

// RCON commands
#[tauri::command]
async fn setup_rcon_for_server(
    server_name: String,
    host: String,
    port: u16,
    _password: String, // This parameter is ignored, we use the one from server.properties
) -> Result<String, String> {
    println!("Setting up RCON for server: {}", server_name);
    
    // Get the actual password from server.properties
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    let actual_password = if properties_path.exists() {
        let properties_manager = ServerPropertiesManager::new(properties_path.clone());
        
        // Get the existing RCON password from server.properties
        match properties_manager.get_property("rcon.password") {
            Ok(existing_password) if !existing_password.is_empty() => {
                println!("Using existing RCON password from server.properties: '{}'", existing_password);
                existing_password
            },
            _ => {
                println!("No RCON password found in server.properties, this shouldn't happen");
                return Err("No RCON password found in server.properties".to_string());
            }
        }
    } else {
        return Err("Server properties file not found".to_string());
    };
    
    println!("RCON config - host: '{}', port: {}, password: '{}'", host, port, actual_password);
    
    let rcon_manager = RCON_MANAGER.lock().await;
    
    let config = RconConfig {
        host: host.clone(),
        port,
        password: actual_password.clone(),
    };
    
    rcon_manager.add_server(server_name.clone(), config);
    
    // Ensure RCON is enabled in server.properties (don't change the password)
    if properties_path.exists() {
        let properties_manager = ServerPropertiesManager::new(properties_path);
        
        // Ensure RCON is enabled and the port is correct
        let _ = properties_manager.update_property("enable-rcon", "true");
        let _ = properties_manager.update_property("rcon.port", &port.to_string());
        // Don't update the password - keep the existing one
        
        println!("RCON enabled in server.properties for {} (password unchanged)", server_name);
    }
    
    Ok(format!("RCON configured for server '{}' with password from server.properties", server_name))
}

#[tauri::command]
async fn connect_rcon(server_name: String) -> Result<String, String> {
    println!("Attempting to connect to RCON for server: {}", server_name);
    let rcon_manager = RCON_MANAGER.lock().await;
    
    match rcon_manager.connect(&server_name) {
        Ok(_) => {
            println!("Successfully connected to RCON for server: {}", server_name);
            Ok(format!("Connected to RCON server '{}'", server_name))
        },
        Err(e) => {
            println!("Failed to connect to RCON for server {}: {}", server_name, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn disconnect_rcon(server_name: String) -> Result<String, String> {
    let rcon_manager = RCON_MANAGER.lock().await;
    
    rcon_manager.disconnect(&server_name);
    
    Ok(format!("Disconnected from RCON server '{}'", server_name))
}

#[tauri::command]
async fn is_rcon_connected(server_name: String) -> bool {
    let rcon_manager = services::rcon_global::get_rcon_manager();
    rcon_manager.is_connected(&server_name)
}

#[tauri::command]
async fn execute_rcon_command(server_name: String, command: String) -> Result<String, String> {
    let rcon_manager = services::rcon_global::get_rcon_manager();
    
    rcon_manager.execute_command(&server_name, &command)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_rcon_connection(server_name: String) -> Result<bool, String> {
    let rcon_manager = services::rcon_global::get_rcon_manager();
    
    rcon_manager.test_connection(&server_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_connected_rcon_servers() -> Vec<String> {
    let rcon_manager = services::rcon_global::get_rcon_manager();
    rcon_manager.get_connected_servers()
}

#[tauri::command]
async fn remove_rcon_server(server_name: String) -> Result<String, String> {
    let rcon_manager = services::rcon_global::get_rcon_manager();
    
    rcon_manager.remove_server(&server_name);
    
    Ok(format!("RCON server '{}' removed", server_name))
}

#[tauri::command]
async fn wait_for_server_ready(server_name: String, max_wait_seconds: u64) -> Result<bool, String> {
    println!("Waiting for server '{}' to be fully ready (max {} seconds)", server_name, max_wait_seconds);
    
    let start_time = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(max_wait_seconds);
    
    while start_time.elapsed() < max_duration {
        // Check if the server is still running
        let service = UNIFIED_SERVER_SERVICE.lock().await;
        if !service.is_server_running(&server_name).await {
            return Err("Server stopped running while waiting".to_string());
        }
        drop(service);
        
        // Wait a bit before the next check
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Check if we can connect to RCON (basic test)
        let rcon_manager = RCON_MANAGER.lock().await;
        if rcon_manager.is_connected(&server_name) {
            println!("Server '{}' appears to be ready (RCON already connected)", server_name);
            return Ok(true);
        }
        drop(rcon_manager);
        
        println!("Server '{}' still starting up... ({:.1}s elapsed)", 
                 server_name, start_time.elapsed().as_secs_f32());
    }
    
    println!("Timeout waiting for server '{}' to be ready", server_name);
    Ok(true) // Return true anyway, let RCON connection handle the rest
}

#[tauri::command]
fn fix_server_rcon_password(server_name: String) -> Result<String, String> {
    let server_path = get_storage_path(&server_name);
    let properties_path = server_path.join("server.properties");
    
    if !properties_path.exists() {
        return Err("server.properties not found".to_string());
    }
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    // Generate a new password without special characters
    let mut rng = rand::thread_rng();
    let mut new_password = String::from("allay_");
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for _ in 0..4 {
        let idx = rng.gen_range(0..chars.len());
        new_password.push(chars.chars().nth(idx).unwrap());
    }
    
    // Update the password in server.properties
    match properties_manager.update_property("rcon.password", &new_password) {
        Ok(_) => {
            println!("Updated RCON password for server '{}' to: '{}'", server_name, new_password);
            Ok(format!("RCON password updated to: {}", new_password))
        },
        Err(e) => Err(format!("Failed to update RCON password: {}", e))
    }
}

#[tauri::command]
fn check_server_rcon_enabled(server_name: String) -> Result<bool, String> {
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    if !properties_path.exists() {
        return Ok(false);
    }
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    match properties_manager.get_property("enable-rcon") {
        Ok(enabled) => Ok(enabled.to_lowercase() == "true"),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
fn get_server_rcon_password(server_name: String) -> Result<String, String> {
    let server_path = PathBuf::from("storage").join(&server_name);
    let properties_path = server_path.join("server.properties");
    
    if !properties_path.exists() {
        return Err("Server properties file not found".to_string());
    }
    
    let properties_manager = ServerPropertiesManager::new(properties_path);
    
    match properties_manager.get_property("rcon.password") {
        Ok(password) => {
            if password.is_empty() {
                Err("RCON password not set".to_string())
            } else {
                Ok(password)
            }
        },
        Err(_) => Err("Failed to read RCON password".to_string()),
    }
}

#[tauri::command]
fn get_system_memory_mb() -> Result<u64, String> {
    use sysinfo::System;
    
    println!("Detecting system memory...");
    
    // Initialize system info
    let mut system = System::new_all();
    system.refresh_memory();
    
    // Get total memory in bytes
    let total_memory_bytes = system.total_memory();
    
    if total_memory_bytes == 0 {
        println!("Warning: Could not detect system memory, using fallback");
        return Ok(8192); // 8GB fallback
    }
    
    // Convert bytes to megabytes (1 MB = 1024 * 1024 bytes)
    let total_memory_mb = total_memory_bytes / (1024 * 1024);
    
    println!("Detected system memory: {} MB ({} GB)", total_memory_mb, total_memory_mb / 1024);
    
    // Sanity check - ensure we have at least 1GB detected
    if total_memory_mb < 1024 {
        println!("Warning: Detected memory ({} MB) seems too low, using fallback", total_memory_mb);
        return Ok(4096); // 4GB fallback for systems with very low detected memory
    }
    
    // Sanity check - cap at 1TB to prevent unrealistic values
    if total_memory_mb > 1024 * 1024 {
        println!("Warning: Detected memory ({} MB) seems too high, capping at 1TB", total_memory_mb);
        return Ok(1024 * 1024); // 1TB cap
    }
    
    Ok(total_memory_mb)
}

#[tauri::command]
async fn initialize_server_monitoring() -> Result<String, String> {
    let mut initialized = MONITORING_INITIALIZED.lock().await;
    if *initialized {
        return Ok("Simple RCON monitoring already initialized".to_string());
    }
    
    let mut monitor = SERVER_MONITOR.lock().await;
    monitor.start_background_monitoring();
    *initialized = true;
    
    Ok("Simple RCON monitoring initialized (5s intervals)".to_string())
}

#[tauri::command]
async fn diagnose_event_system() -> Result<String, String> {
    let monitor = SERVER_MONITOR.lock().await;
    let statuses = monitor.get_all_statuses().await;
    
    let diagnostic = format!(
        "🎯 Simple RCON Monitor Status:\n\
         • Servers monitored: {}\n\
         • Mode: RCON-only monitoring (5s intervals)\n\
         • Status: {}",
        statuses.len(),
        if statuses.is_empty() { "No servers" } else { "Active" }
    );
    
    println!("{}", diagnostic);
    Ok(diagnostic)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_server_instance,
            create_server_transactional,
            cleanup_incomplete_servers,
            get_all_server_instances,
            remove_server_instance,
            delete_server_completely,
            update_server_description,
            update_server_memory,
            get_minecraft_versions,
            get_all_minecraft_versions,
            get_version_summary,
            refresh_version_cache,
            clear_version_cache,
            download_server_jar,
            setup_server,
            start_server,
            stop_server,
            toggle_server,
            get_server_loader_type,
            is_server_running,
            query_server_status,
            start_server_monitoring,
            stop_server_monitoring,
            get_server_monitor_status,
            get_all_server_monitor_statuses,
            update_server_monitor_status,
            initialize_server_monitoring,
            diagnose_event_system,
            get_running_servers,
            get_jar_cache_stats,
            clear_jar_cache,
            is_jar_cached,
            get_server_motd,
            get_server_max_players,
            update_server_property,
            setup_rcon_for_server,
            connect_rcon,
            disconnect_rcon,
            is_rcon_connected,
            execute_rcon_command,
            test_rcon_connection,
            get_connected_rcon_servers,
            remove_rcon_server,
            wait_for_server_ready,
            check_server_rcon_enabled,
            fix_server_rcon_password,
            get_server_rcon_password,
            get_system_memory_mb
        ])
        .setup(|app| {
            // Set app handle for event emission in Simple RCON Monitor
            let app_handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                let mut monitor = SERVER_MONITOR.lock().await;
                monitor.set_app_handle(app_handle);
                println!("🎯 Simple RCON Monitor configured with app handle for events");
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
