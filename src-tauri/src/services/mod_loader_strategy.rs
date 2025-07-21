use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use crate::models::version::LoaderType;

// Import all strategy implementations
use crate::services::vanilla_strategy::VanillaStrategy;
use crate::services::fabric_strategy::FabricStrategy;
use crate::services::forge_strategy::ForgeStrategy;
use crate::services::neoforge_strategy::NeoForgeStrategy;
use crate::services::paper_strategy::PaperStrategy;
use crate::services::quilt_strategy::QuiltStrategy;

/// Strategy trait for mod loader specific operations
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