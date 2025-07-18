// Declaración de módulos
mod models;
mod util;

use std::path::PathBuf;
use util::{ServerFileManager, ServerInstance};

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
    
    manager.get_all_instances().map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_server_instance(name: String) -> Result<String, String> {
    let config_path = PathBuf::from("storage/server_config.json");
    let manager = ServerFileManager::new(config_path);
    
    manager.remove_instance(&name).map_err(|e| e.to_string())?;
    
    Ok(format!("Server instance '{}' removed successfully", name))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_server_instance,
            get_all_server_instances,
            remove_server_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
