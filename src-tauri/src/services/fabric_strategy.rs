use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::LoaderType;
use crate::util::JarCacheManager;

/// Fabric strategy
pub struct FabricStrategy;

#[async_trait]
impl ModLoaderStrategy for FabricStrategy {
    // Uses default implementation from trait

    async fn get_download_url(&self, _client: &Client, minecraft_version: &str, loader_version: &str) -> Result<String> {
        // Extract clean loader version
        let clean_loader_version = if loader_version.starts_with("fabric-") {
            let without_prefix = loader_version.strip_prefix("fabric-").unwrap_or(loader_version);
            if let Some(dash_pos) = without_prefix.find('-') {
                &without_prefix[..dash_pos]
            } else {
                without_prefix
            }
        } else {
            loader_version
        };
        
        Ok(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.3/server/jar",
            minecraft_version, clean_loader_version
        ))
    }
    
    fn get_filename(&self, minecraft_version: &str, loader_version: &str) -> String {
        let clean_version = if loader_version.starts_with("fabric-") {
            let without_prefix = loader_version.strip_prefix("fabric-").unwrap_or(loader_version);
            if let Some(dash_pos) = without_prefix.find('-') {
                &without_prefix[..dash_pos]
            } else {
                without_prefix
            }
        } else {
            loader_version
        };
        format!("fabric-server-mc.{}-loader.{}-launcher.1.0.3.jar", minecraft_version, clean_version)
    }
    
    async fn setup_server(&self, _client: &Client, server_path: &PathBuf, minecraft_version: &str, loader_version: &str) -> Result<()> {
        let clean_version = if loader_version.starts_with("fabric-") {
            let without_prefix = loader_version.strip_prefix("fabric-").unwrap_or(loader_version);
            if let Some(dash_pos) = without_prefix.find('-') {
                &without_prefix[..dash_pos]
            } else {
                without_prefix
            }
        } else {
            loader_version
        };
        
        let server_jar_name = format!("fabric-server-mc.{}-loader.{}-launcher.1.0.3.jar", minecraft_version, clean_version);
        let server_jar = server_path.join(&server_jar_name);
        
        if !server_jar.exists() {
            return Err(anyhow!("Fabric server launcher not found: {:?}", server_jar));
        }

        println!("Fabric server launcher ready: {:?}", server_jar);
        Ok(())
    }
    
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>> {
        let mut args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-jar".to_string(),
        ];

        // Find the fabric server launcher
        let entries = fs::read_dir(server_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("fabric-server-mc.") && file_name.contains("-loader.") && file_name.contains("-launcher.") && file_name.ends_with(".jar") {
                args.push(file_name);
                args.push("nogui".to_string());
                return Ok(args);
            }
        }
        Err(anyhow!("Fabric server launcher JAR not found"))
    }
}