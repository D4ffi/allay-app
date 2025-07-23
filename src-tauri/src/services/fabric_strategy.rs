use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use chrono::Utc;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::{LoaderType, VersionResponse, MinecraftVersion, VersionType, FabricVersions, FabricLoaderVersion};
use crate::util::JarCacheManager;

/// Fabric strategy
pub struct FabricStrategy;

#[async_trait]
impl ModLoaderStrategy for FabricStrategy {
    async fn get_versions(&self, client: &Client, minecraft_version: Option<String>) -> Result<VersionResponse> {
        // Get game versions
        let game_url = "https://meta.fabricmc.net/v2/versions";
        let game_response: FabricVersions = client.get(game_url).send().await?.json().await?;

        // Get loader versions
        let loader_url = "https://meta.fabricmc.net/v2/versions/loader";
        let loader_response: Vec<FabricLoaderVersion> = client.get(loader_url).send().await?.json().await?;

        let mut versions = Vec::new();
        
        // If minecraft_version is specified, create versions for that specific MC version
        if let Some(target_mc_version) = minecraft_version {
            // Check if the target MC version exists in game versions
            let target_game_version = game_response.game.iter()
                .find(|v| v.version == target_mc_version);
            
            if let Some(game_version) = target_game_version {
                // Get all loader versions for this MC version
                for (i, loader) in loader_response.iter().enumerate() {
                    let version_id = format!("fabric-{}-{}", loader.version, game_version.version);
                    let minecraft_version_obj = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: i == 0,
                        recommended: loader.stable,
                        minecraft_version: Some(game_version.version.clone()),
                    };
                    versions.push(minecraft_version_obj);
                }
            }
        } else {
            // Get all fabric loader versions for all stable MC versions (no limits)
            let stable_game_versions: Vec<_> = game_response
                .game
                .iter()
                .filter(|v| v.stable)
                .collect();

            let stable_loader = loader_response.iter().find(|v| v.stable);
            let latest_loader = loader_response.first();

            // Create versions for stable MC versions with stable loader
            if let Some(loader) = stable_loader {
                for (i, game_version) in stable_game_versions.iter().enumerate() {
                    let version_id = format!("fabric-{}-{}", loader.version, game_version.version);
                    let minecraft_version_obj = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: i == 0 && loader.version == latest_loader.unwrap().version,
                        recommended: true,
                        minecraft_version: Some(game_version.version.clone()),
                    };
                    versions.push(minecraft_version_obj);
                }
            }

            // Add the latest loader with the latest MC version if different
            if let (Some(latest_loader), Some(latest_game)) = (latest_loader, game_response.game.first()) {
                if stable_loader.is_none() || latest_loader.version != stable_loader.unwrap().version {
                    let version_id = format!("fabric-{}-{}", latest_loader.version, latest_game.version);
                    let minecraft_version_obj = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: true,
                        recommended: false,
                        minecraft_version: Some(latest_game.version.clone()),
                    };
                    versions.insert(0, minecraft_version_obj);
                }
            }
        }

        let latest = versions.iter().find(|v| v.latest).cloned();
        let recommended = versions.iter().find(|v| v.recommended).cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }

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