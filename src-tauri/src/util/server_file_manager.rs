use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub name: String,
    pub version: String,
    pub mod_loader: String,
    pub mod_loader_version: String,
    pub storage_path: PathBuf,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_memory")]
    pub memory_mb: u32,
}

fn default_memory() -> u32 {
    2048 // Default 2GB in MB
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub instances: HashMap<String, ServerInstance>,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }
}

pub struct ServerFileManager {
    config_path: PathBuf,
}

impl ServerFileManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn load_config(&self) -> Result<ServerConfig, Error> {
        if !self.config_path.exists() {
            return Ok(ServerConfig::new());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let content = content.trim();
        
        // Handle empty or whitespace-only files
        if content.is_empty() {
            return Ok(ServerConfig::new());
        }
        
        let config: ServerConfig = serde_json::from_str(content)
            .map_err(|e| {
                eprintln!("JSON parsing error: {}", e);
                eprintln!("File content: '{}'", content);
                Error::new(ErrorKind::InvalidData, format!("Failed to parse JSON: {}", e))
            })?;
        
        Ok(config)
    }

    pub fn save_config(&self, config: &ServerConfig) -> Result<(), Error> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add_instance(&self, instance: ServerInstance) -> Result<(), Error> {
        let mut config = self.load_config()?;
        
        if config.instances.contains_key(&instance.name) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("Instance with name '{}' already exists", instance.name),
            ));
        }

        config.instances.insert(instance.name.clone(), instance);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn remove_instance(&self, name: &str) -> Result<(), Error> {
        let mut config = self.load_config()?;
        
        if !config.instances.contains_key(name) {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Instance with name '{}' not found", name),
            ));
        }

        config.instances.remove(name);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn remove_instance_with_storage(&self, name: &str, base_storage_path: &Path) -> Result<(), Error> {
        // Get instance info before removing it
        let config = self.load_config()?;
        let instance = config.instances.get(name)
            .ok_or_else(|| Error::new(
                ErrorKind::NotFound,
                format!("Instance with name '{}' not found", name),
            ))?;

        // Build storage path
        let storage_path = base_storage_path.join(name);
        
        // Remove from config first
        self.remove_instance(name)?;
        
        // Then remove the storage directory if it exists
        if storage_path.exists() {
            fs::remove_dir_all(&storage_path).map_err(|e| {
                Error::new(
                    ErrorKind::PermissionDenied,
                    format!("Failed to delete server folder '{}': {}", storage_path.display(), e),
                )
            })?;
        }

        Ok(())
    }

    pub fn update_instance(&self, name: &str, updated_instance: ServerInstance) -> Result<(), Error> {
        let mut config = self.load_config()?;
        
        if !config.instances.contains_key(name) {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Instance with name '{}' not found", name),
            ));
        }

        config.instances.insert(name.to_string(), updated_instance);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn get_instance(&self, name: &str) -> Result<Option<ServerInstance>, Error> {
        let config = self.load_config()?;
        Ok(config.instances.get(name).cloned())
    }

    pub fn get_all_instances(&self) -> Result<Vec<ServerInstance>, Error> {
        let config = self.load_config()?;
        Ok(config.instances.values().cloned().collect())
    }

    pub fn instance_exists(&self, name: &str) -> Result<bool, Error> {
        let config = self.load_config()?;
        Ok(config.instances.contains_key(name))
    }

    pub fn create_storage_directory(&self, instance_name: &str, base_storage_path: &Path) -> Result<PathBuf, Error> {
        let storage_path = base_storage_path.join(instance_name);
        fs::create_dir_all(&storage_path)?;
        Ok(storage_path)
    }

    pub fn initialize_config(&self) -> Result<(), Error> {
        if !self.config_path.exists() || fs::read_to_string(&self.config_path)?.trim().is_empty() {
            let config = ServerConfig::new();
            self.save_config(&config)?;
        }
        Ok(())
    }
}

impl ServerInstance {
    pub fn new(
        name: String,
        version: String,
        mod_loader: String,
        mod_loader_version: String,
        base_storage_path: &Path,
    ) -> Result<Self, Error> {
        let storage_path = base_storage_path.join(&name);
        
        Ok(Self {
            name,
            description: None,
            version,
            mod_loader,
            mod_loader_version,
            storage_path,
            memory_mb: default_memory(),
        })
    }
}