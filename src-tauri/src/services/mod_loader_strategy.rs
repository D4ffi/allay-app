use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use crate::models::version::LoaderType;
use crate::util::JarCacheManager;

// Import all strategy implementations
use crate::services::vanilla_strategy::VanillaStrategy;
use crate::services::fabric_strategy::FabricStrategy;
use crate::services::forge_strategy::ForgeStrategy;
use crate::services::neoforge_strategy::NeoForgeStrategy;
use crate::services::paper_strategy::PaperStrategy;
use crate::services::quilt_strategy::QuiltStrategy;

/// Strategy trait for mod-loader-specific operations
#[async_trait]
pub trait ModLoaderStrategy: Send + Sync {
    /// Get the download URL for this mod loader
    async fn get_download_url(&self, client: &Client, minecraft_version: &str, loader_version: &str) -> Result<String>;
    
    /// Get the filename for the downloaded file
    fn get_filename(&self, minecraft_version: &str, loader_version: &str) -> String;
    
    /// Setup/install the mod loader in the server directory
    async fn setup_server(&self, client: &Client, server_path: &PathBuf, minecraft_version: &str, loader_version: &str) -> Result<()>;
    
    /// Build the start command for this mod loader
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>>;

    /// Default implementation for downloading and caching JAR files
    /// Can be overridden by strategies that need special handling
    async fn download_server_jar(
        &self, 
        client: &Client, 
        jar_cache: &JarCacheManager,
        minecraft_version: &str, 
        loader_version: &str, 
        server_path: &PathBuf,
        loader_type: &LoaderType
    ) -> Result<PathBuf> {
        let loader_version_opt = if loader_version.is_empty() { None } else { Some(loader_version) };
        
        // Check if JAR is cached first
        if jar_cache.is_jar_cached(loader_type, minecraft_version, loader_version_opt) {
            println!("{:?} JAR found in cache, copying to server: {:?}", loader_type, server_path);
            return jar_cache.copy_cached_jar_to_server(loader_type, minecraft_version, loader_version_opt, server_path);
        }

        println!("{:?} JAR not in cache, downloading...", loader_type);
        
        let download_url = self.get_download_url(client, minecraft_version, loader_version).await?;
        let jar_name = self.get_filename(minecraft_version, loader_version);

        println!("Downloading {} from: {}", jar_name, download_url);

        // Download the JAR file
        let response = client.get(&download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download {:?} JAR: HTTP {}", loader_type, response.status()));
        }

        let bytes = response.bytes().await?;

        // Cache the JAR first
        println!("Caching downloaded {:?} JAR...", loader_type);
        jar_cache.cache_jar(loader_type, minecraft_version, loader_version_opt, &bytes)?;

        // Then copy it to the server directory
        println!("Copying cached {:?} JAR to server: {:?}", loader_type, server_path);
        let jar_path = jar_cache.copy_cached_jar_to_server(loader_type, minecraft_version, loader_version_opt, server_path)?;

        println!("Successfully downloaded and cached {:?} JAR: {:?}", loader_type, jar_path);
        Ok(jar_path)
    }
}

/// Factory function to get the appropriate strategy
pub fn get_strategy(loader_type: &LoaderType) -> Box<dyn ModLoaderStrategy> {
    match loader_type {
        LoaderType::Vanilla => Box::new(VanillaStrategy),
        LoaderType::Fabric => Box::new(FabricStrategy),
        LoaderType::Forge => Box::new(ForgeStrategy),
        LoaderType::NeoForge => Box::new(NeoForgeStrategy),
        LoaderType::Paper => Box::new(PaperStrategy),
        LoaderType::Quilt => Box::new(QuiltStrategy),
    }
}