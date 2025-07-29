use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerOrderConfig {
    pub order: Vec<String>,
    pub last_updated: DateTime<Utc>,
    pub version: u32,
}

impl Default for ServerOrderConfig {
    fn default() -> Self {
        Self {
            order: Vec::new(),
            last_updated: Utc::now(),
            version: 1,
        }
    }
}

/// Get the path to the server order configuration file
pub fn get_server_order_file_path() -> PathBuf {
    let storage_path = PathBuf::from("storage");
    storage_path.join("server_order.json")
}

/// Load the server order from the configuration file
pub fn load_server_order_config() -> Result<ServerOrderConfig, String> {
    let file_path = get_server_order_file_path();
    
    if !file_path.exists() {
        return Ok(ServerOrderConfig::default());
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Error reading server order file: {}", e))?;
    
    let config: ServerOrderConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Error parsing server order config: {}", e))?;
    
    Ok(config)
}

/// Save the server order configuration to file
pub fn save_server_order_config(config: &ServerOrderConfig) -> Result<(), String> {
    let file_path = get_server_order_file_path();
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Error creating storage directory: {}", e))?;
    }
    
    let json_content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error serializing server order config: {}", e))?;
    
    fs::write(&file_path, json_content)
        .map_err(|e| format!("Error writing server order file: {}", e))?;
    
    Ok(())
}

/// Get the current server order as a vector of server names
#[tauri::command]
pub async fn get_server_order() -> Result<Vec<String>, String> {
    let config = load_server_order_config()?;
    Ok(config.order)
}

/// Save a new server order
#[tauri::command]
pub async fn save_server_order(order: Vec<String>) -> Result<(), String> {
    println!("Saving server order: {:?}", order);
    
    let config = ServerOrderConfig {
        order: order.clone(),
        last_updated: Utc::now(),
        version: 1,
    };
    
    match save_server_order_config(&config) {
        Ok(_) => {
            println!("Server order saved successfully to storage/server_order.json");
            Ok(())
        },
        Err(e) => {
            println!("Failed to save server order: {}", e);
            Err(e)
        }
    }
}

/// Update the position of a specific server in the order (optimized for drag & drop)
#[tauri::command]
pub async fn update_server_position(server_name: String, new_index: usize) -> Result<(), String> {
    let mut config = load_server_order_config()?;
    
    // Remove server from current position
    if let Some(current_index) = config.order.iter().position(|name| name == &server_name) {
        config.order.remove(current_index);
    }
    
    // Insert at new position, ensuring we don't go out of bounds
    let insert_index = new_index.min(config.order.len());
    config.order.insert(insert_index, server_name);
    
    // Update timestamp
    config.last_updated = Utc::now();
    
    save_server_order_config(&config)
}

/// Clean up the server order by removing servers that no longer exist
pub fn cleanup_server_order(existing_servers: &[String]) -> Result<(), String> {
    let mut config = load_server_order_config()?;
    let original_len = config.order.len();
    
    // Keep only servers that still exist
    config.order.retain(|server_name| existing_servers.contains(server_name));
    
    // Only save if we actually removed something
    if config.order.len() != original_len {
        config.last_updated = Utc::now();
        save_server_order_config(&config)?;
    }
    
    Ok(())
}

/// Add new servers to the end of the order
pub fn add_new_servers_to_order(new_servers: &[String]) -> Result<(), String> {
    if new_servers.is_empty() {
        return Ok(());
    }
    
    let mut config = load_server_order_config()?;
    
    // Add new servers that aren't already in the order
    for server_name in new_servers {
        if !config.order.contains(server_name) {
            config.order.push(server_name.clone());
        }
    }
    
    config.last_updated = Utc::now();
    save_server_order_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("ALLAY_STORAGE_PATH", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_default_config() {
        let config = ServerOrderConfig::default();
        assert!(config.order.is_empty());
        assert_eq!(config.version, 1);
    }

    #[tokio::test]
    async fn test_save_and_load_order() {
        let _temp_dir = setup_test_env();
        
        let test_order = vec!["server1".to_string(), "server2".to_string(), "server3".to_string()];
        
        // Save order
        let result = save_server_order(test_order.clone()).await;
        assert!(result.is_ok());
        
        // Load order
        let loaded_order = get_server_order().await.unwrap();
        assert_eq!(loaded_order, test_order);
    }

    #[tokio::test]
    async fn test_update_server_position() {
        let _temp_dir = setup_test_env();
        
        let initial_order = vec!["server1".to_string(), "server2".to_string(), "server3".to_string()];
        save_server_order(initial_order).await.unwrap();
        
        // Move server2 to position 0
        update_server_position("server2".to_string(), 0).await.unwrap();
        
        let updated_order = get_server_order().await.unwrap();
        assert_eq!(updated_order, vec!["server2", "server1", "server3"]);
    }
}