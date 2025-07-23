use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::LoaderType;
use crate::util::JarCacheManager;

/// Vanilla Minecraft strategy
pub struct VanillaStrategy;

#[async_trait]
impl ModLoaderStrategy for VanillaStrategy {
    // Uses default implementation from trait

    async fn get_download_url(&self, client: &Client, minecraft_version: &str, _loader_version: &str) -> Result<String> {
        // Get version manifest
        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest: serde_json::Value = client.get(manifest_url).send().await?.json().await?;
        
        // Find the specific version
        let versions = manifest["versions"].as_array()
            .ok_or_else(|| anyhow!("Invalid version manifest"))?;
        
        let version_info = versions.iter()
            .find(|v| v["id"].as_str() == Some(minecraft_version))
            .ok_or_else(|| anyhow!("Minecraft version {} not found", minecraft_version))?;
        
        let version_url = version_info["url"].as_str()
            .ok_or_else(|| anyhow!("Version URL not found"))?;
        
        // Get version details
        let version_details: serde_json::Value = client.get(version_url).send().await?.json().await?;
        
        // Get server JAR URL
        let server_url = version_details["downloads"]["server"]["url"].as_str()
            .ok_or_else(|| anyhow!("Server JAR URL not found for version {}", minecraft_version))?;
        
        Ok(server_url.to_string())
    }
    
    fn get_filename(&self, minecraft_version: &str, _loader_version: &str) -> String {
        format!("server-{}.jar", minecraft_version)
    }
    
    async fn setup_server(&self, _client: &Client, server_path: &PathBuf, minecraft_version: &str, _loader_version: &str) -> Result<()> {
        let jar_name = format!("server-{}.jar", minecraft_version);
        let jar_path = server_path.join(&jar_name);
        
        if !jar_path.exists() {
            return Err(anyhow!("Server JAR not found: {:?}", jar_path));
        }

        // Check if server is already initialized
        let world_folder = server_path.join("world");
        let logs_folder = server_path.join("logs");
        
        if world_folder.exists() || logs_folder.exists() {
            println!("Vanilla server already initialized: {:?}", jar_path);
            return Ok(());
        }

        println!("Initializing Vanilla server...");
        
        // Run the server JAR once to generate initial files
        let output = Command::new("java")
            .args(&[
                "-Xmx1G",
                "-Xms512M", 
                "-jar", 
                &jar_name,
                "nogui"
            ])
            .current_dir(server_path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!("Java is not installed or not found in PATH. Please install Java to run Minecraft servers.")
                } else {
                    anyhow!("Failed to execute Java: {}", e)
                }
            })?;
        
        println!("Vanilla server initialization exit status: {:?}", output.status);
        println!("Vanilla server initialized successfully");
        Ok(())
    }
    
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>> {
        let mut args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-jar".to_string(),
        ];

        // Find the vanilla server JAR
        let entries = fs::read_dir(server_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("server-") && file_name.ends_with(".jar") {
                args.push(file_name);
                args.push("nogui".to_string());
                return Ok(args);
            }
        }
        Err(anyhow!("Vanilla server JAR not found"))
    }
}