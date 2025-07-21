use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use crate::services::mod_loader_strategy::ModLoaderStrategy;

/// Paper strategy
pub struct PaperStrategy;

#[async_trait]
impl ModLoaderStrategy for PaperStrategy {
    async fn get_download_url(&self, client: &Client, minecraft_version: &str, _loader_version: &str) -> Result<String> {
        // Get latest build for the version
        let builds_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", minecraft_version);
        let builds_response: serde_json::Value = client.get(&builds_url).send().await?.json().await?;
        
        let builds = builds_response["builds"].as_array()
            .ok_or_else(|| anyhow!("No builds found for Paper version {}", minecraft_version))?;
        
        let latest_build = builds.last()
            .ok_or_else(|| anyhow!("No builds available for Paper version {}", minecraft_version))?;
        
        let build_number = latest_build["build"].as_u64()
            .ok_or_else(|| anyhow!("Invalid build number"))?;
        
        let jar_name = latest_build["downloads"]["application"]["name"].as_str()
            .ok_or_else(|| anyhow!("JAR name not found"))?;
        
        Ok(format!(
            "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}",
            minecraft_version, build_number, jar_name
        ))
    }
    
    fn get_filename(&self, minecraft_version: &str, _loader_version: &str) -> String {
        format!("paper-{}.jar", minecraft_version)
    }
    
    async fn setup_server(&self, _client: &Client, server_path: &PathBuf, minecraft_version: &str, _loader_version: &str) -> Result<()> {
        let jar_name = format!("paper-{}.jar", minecraft_version);
        let jar_path = server_path.join(&jar_name);
        
        if !jar_path.exists() {
            return Err(anyhow!("Paper JAR not found: {:?}", jar_path));
        }

        println!("Paper server ready: {:?}", jar_path);
        Ok(())
    }
    
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>> {
        let mut args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-jar".to_string(),
        ];

        // Find Paper JAR
        let entries = fs::read_dir(server_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("paper-") && file_name.ends_with(".jar") {
                args.push(file_name);
                args.push("nogui".to_string());
                return Ok(args);
            }
        }
        Err(anyhow!("Paper server JAR not found"))
    }
}