// Declaración de módulos
mod models;
mod util;
mod services;

use std::path::PathBuf;
use util::{ServerFileManager, ServerInstance, JarCacheManager, CacheStats, ServerPropertiesManager};
use services::version_manager::{VersionManager, VersionSummary};
use services::download_service::DownloadService;
use services::server_management_service::ServerManagementService;
use models::version::{LoaderType, VersionResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;


// Global server management service
lazy_static::lazy_static! {
    static ref SERVER_MANAGER: Arc<Mutex<ServerManagementService>> = 
        Arc::new(Mutex::new(ServerManagementService::new()));
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
    
    // Initialize config file if it doesn't exist or is empty
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

// Version management commands
#[tauri::command]
async fn get_minecraft_versions(loader: String, force_refresh: bool, minecraft_version: Option<String>) -> Result<VersionResponse, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let manager = VersionManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    let loader_type = match loader.as_str() {
        "vanilla" => LoaderType::Vanilla,
        "fabric" => LoaderType::Fabric,
        "forge" => LoaderType::Forge,
        "neoforge" => LoaderType::NeoForge,
        "paper" => LoaderType::Paper,
        "quilt" => LoaderType::Quilt,
        _ => return Err("Invalid loader type".to_string()),
    };
    
    manager.get_versions_for_minecraft(loader_type, force_refresh, minecraft_version).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_minecraft_versions(force_refresh: bool) -> Result<HashMap<String, VersionResponse>, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let manager = VersionManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    manager.get_all_versions(force_refresh).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_version_summary() -> Result<VersionSummary, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let manager = VersionManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    manager.get_version_summary().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_version_cache(loader: Option<String>) -> Result<HashMap<String, bool>, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let manager = VersionManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    let loader_type = if let Some(loader) = loader {
        Some(match loader.as_str() {
            "vanilla" => LoaderType::Vanilla,
            "fabric" => LoaderType::Fabric,
            "forge" => LoaderType::Forge,
            "neoforge" => LoaderType::NeoForge,
            "paper" => LoaderType::Paper,
            "quilt" => LoaderType::Quilt,
            _ => return Err("Invalid loader type".to_string()),
        })
    } else {
        None
    };
    
    manager.refresh_cache(loader_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_version_cache(loader: Option<String>) -> Result<String, String> {
    let cache_dir = PathBuf::from("storage/version_cache");
    let manager = VersionManager::new(cache_dir).map_err(|e| e.to_string())?;
    
    if let Some(loader) = loader {
        let loader_type = match loader.as_str() {
            "vanilla" => LoaderType::Vanilla,
            "fabric" => LoaderType::Fabric,
            "forge" => LoaderType::Forge,
            "neoforge" => LoaderType::NeoForge,
            "paper" => LoaderType::Paper,
            "quilt" => LoaderType::Quilt,
            _ => return Err("Invalid loader type".to_string()),
        };
        
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
    let download_service = DownloadService::new().map_err(|e| e.to_string())?;
    let storage_path = PathBuf::from("storage").join(&server_name);
    
    let loader_type = match loader.as_str() {
        "vanilla" => LoaderType::Vanilla,
        "fabric" => LoaderType::Fabric,
        "forge" => LoaderType::Forge,
        "neoforge" => LoaderType::NeoForge,
        "paper" => LoaderType::Paper,
        "quilt" => LoaderType::Quilt,
        _ => return Err("Invalid loader type".to_string()),
    };
    
    match download_service.download_server_jar(
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
    let storage_path = PathBuf::from("storage").join(&server_name);
    
    let loader_type = match loader.as_str() {
        "vanilla" => LoaderType::Vanilla,
        "fabric" => LoaderType::Fabric,
        "forge" => LoaderType::Forge,
        "neoforge" => LoaderType::NeoForge,
        "paper" => LoaderType::Paper,
        "quilt" => LoaderType::Quilt,
        _ => return Err("Invalid loader type".to_string()),
    };
    
    let manager = SERVER_MANAGER.lock().await;
    
    match manager.setup_server(
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
    let storage_path = PathBuf::from("storage").join(&server_name);
    
    let loader_type = match loader.as_str() {
        "vanilla" => LoaderType::Vanilla,
        "fabric" => LoaderType::Fabric,
        "forge" => LoaderType::Forge,
        "neoforge" => LoaderType::NeoForge,
        "paper" => LoaderType::Paper,
        "quilt" => LoaderType::Quilt,
        _ => return Err("Invalid loader type".to_string()),
    };
    
    let manager = SERVER_MANAGER.lock().await;
    
    match manager.start_server(&server_name, &storage_path, loader_type).await {
        Ok(_) => Ok(format!("Server '{}' started successfully", server_name)),
        Err(e) => Err(format!("Failed to start server '{}': {}", server_name, e)),
    }
}

#[tauri::command]
async fn stop_server(server_name: String) -> Result<String, String> {
    let manager = SERVER_MANAGER.lock().await;
    
    match manager.stop_server(&server_name).await {
        Ok(_) => Ok(format!("Server '{}' stopped successfully", server_name)),
        Err(e) => Err(format!("Failed to stop server '{}': {}", server_name, e)),
    }
}

#[tauri::command]
async fn is_server_running(server_name: String) -> bool {
    let manager = SERVER_MANAGER.lock().await;
    manager.is_server_running(&server_name).await
}

#[tauri::command]
async fn get_running_servers() -> Vec<String> {
    let manager = SERVER_MANAGER.lock().await;
    manager.get_running_servers().await
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
        let loader_type = match loader_str.as_str() {
            "vanilla" => LoaderType::Vanilla,
            "fabric" => LoaderType::Fabric,
            "forge" => LoaderType::Forge,
            "neoforge" => LoaderType::NeoForge,
            "paper" => LoaderType::Paper,
            "quilt" => LoaderType::Quilt,
            _ => return Err("Invalid loader type".to_string()),
        };
        
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
    
    let loader_type = match loader.as_str() {
        "vanilla" => LoaderType::Vanilla,
        "fabric" => LoaderType::Fabric,
        "forge" => LoaderType::Forge,
        "neoforge" => LoaderType::NeoForge,
        "paper" => LoaderType::Paper,
        "quilt" => LoaderType::Quilt,
        _ => return Err("Invalid loader type".to_string()),
    };
    
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_server_instance,
            get_all_server_instances,
            remove_server_instance,
            get_minecraft_versions,
            get_all_minecraft_versions,
            get_version_summary,
            refresh_version_cache,
            clear_version_cache,
            download_server_jar,
            setup_server,
            start_server,
            stop_server,
            is_server_running,
            get_running_servers,
            get_jar_cache_stats,
            clear_jar_cache,
            is_jar_cached,
            get_server_motd,
            update_server_property,
            get_system_memory_mb
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
