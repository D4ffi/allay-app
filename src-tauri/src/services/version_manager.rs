use crate::models::version::*;
use crate::services::mod_loader_strategy::get_strategy;
use crate::util::version_cache_manager::{VersionCacheManager, CacheInfo};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use reqwest::Client;

pub struct VersionManager {
    client: Client,
    cache_manager: VersionCacheManager,
}

impl VersionManager {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            cache_manager: VersionCacheManager::new(cache_dir)?,
        })
    }

    pub async fn get_versions(&self, loader: LoaderType, force_refresh: bool) -> Result<VersionResponse> {
        self.get_versions_for_minecraft(loader, force_refresh, None).await
    }

    pub async fn get_versions_for_minecraft(&self, loader: LoaderType, force_refresh: bool, minecraft_version: Option<String>) -> Result<VersionResponse> {
        // For loaders with minecraft version filtering, always fetch fresh data
        let should_force_refresh = force_refresh || minecraft_version.is_some();
        
        // Check the cache first unless force refresh is requested, or we need a specific MC version
        if !should_force_refresh {
            if let Ok(true) = self.cache_manager.is_cache_valid(&loader) {
                if let Ok(Some(cache)) = self.cache_manager.load_cache(&loader) {
                    let latest = cache.versions.iter().find(|v| v.latest).cloned();
                    let recommended = cache.versions.iter().find(|v| v.recommended).cloned();
                    
                    return Ok(VersionResponse {
                        latest,
                        recommended,
                        versions: cache.versions,
                    });
                }
            }
        }

        // Fetch from API using strategy pattern
        let strategy = get_strategy(&loader);
        let response = strategy.get_versions(&self.client, minecraft_version.clone()).await?;
        
        // Save to cache (only if no specific minecraft version was requested)
        if minecraft_version.is_none() {
            if let Err(e) = self.cache_manager.save_cache(loader, response.versions.clone()) {
                eprintln!("Failed to save cache: {}", e);
            }
        }

        Ok(response)
    }

    pub async fn get_all_versions(&self, force_refresh: bool) -> Result<HashMap<String, VersionResponse>> {
        let mut results = HashMap::new();
        let loaders = vec![
            ("vanilla", LoaderType::Vanilla),
            ("fabric", LoaderType::Fabric),
            ("forge", LoaderType::Forge),
            ("neoforge", LoaderType::NeoForge),
            ("paper", LoaderType::Paper),
            ("quilt", LoaderType::Quilt),
        ];

        for (name, loader) in loaders {
            match self.get_versions(loader, force_refresh).await {
                Ok(response) => {
                    results.insert(name.to_string(), response);
                }
                Err(e) => {
                    eprintln!("Failed to get versions for {}: {}", name, e);
                }
            }
        }

        Ok(results)
    }

    pub fn get_cache_info(&self) -> Result<HashMap<String, CacheInfo>> {
        self.cache_manager.get_cache_info()
    }

    pub fn clear_cache(&self, loader: &LoaderType) -> Result<()> {
        self.cache_manager.clear_cache(loader)
    }

    pub fn clear_all_cache(&self) -> Result<()> {
        self.cache_manager.clear_all_cache()
    }

    pub fn cleanup_expired_cache(&self) -> Result<Vec<String>> {
        self.cache_manager.cleanup_expired_cache()
    }

    pub async fn refresh_cache(&self, loader: Option<LoaderType>) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();

        if let Some(loader) = loader {
            // Refresh specific loader
            let loader_name = match loader {
                LoaderType::Vanilla => "vanilla",
                LoaderType::Fabric => "fabric",
                LoaderType::Forge => "forge",
                LoaderType::NeoForge => "neoforge",
                LoaderType::Paper => "paper",
                LoaderType::Quilt => "quilt",
            };

            match self.get_versions(loader, true).await {
                Ok(_) => {
                    results.insert(loader_name.to_string(), true);
                }
                Err(e) => {
                    eprintln!("Failed to refresh cache for {}: {}", loader_name, e);
                    results.insert(loader_name.to_string(), false);
                }
            }
        } else {
            // Refresh all loaders
            let loaders = vec![
                ("vanilla", LoaderType::Vanilla),
                ("fabric", LoaderType::Fabric),
                ("forge", LoaderType::Forge),
                ("neoforge", LoaderType::NeoForge),
                ("paper", LoaderType::Paper),
                ("quilt", LoaderType::Quilt),
            ];

            for (name, loader) in loaders {
                match self.get_versions(loader, true).await {
                    Ok(_) => {
                        results.insert(name.to_string(), true);
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh cache for {}: {}", name, e);
                        results.insert(name.to_string(), false);
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn get_version_summary(&self) -> Result<VersionSummary> {
        let cache_info = self.get_cache_info()?;
        let mut summary = VersionSummary {
            total_loaders: 6,
            cached_loaders: 0,
            valid_cache_count: 0,
            expired_cache_count: 0,
            latest_versions: HashMap::new(),
            cache_status: cache_info.clone(),
        };

        for (loader_name, info) in &cache_info {
            if info.exists {
                summary.cached_loaders += 1;
                if info.valid {
                    summary.valid_cache_count += 1;
                } else {
                    summary.expired_cache_count += 1;
                }
            }
        }

        // Get latest versions from cache or API
        let loaders = vec![
            ("vanilla", LoaderType::Vanilla),
            ("fabric", LoaderType::Fabric),
            ("forge", LoaderType::Forge),
            ("neoforge", LoaderType::NeoForge),
            ("paper", LoaderType::Paper),
            ("quilt", LoaderType::Quilt),
        ];

        for (name, loader) in loaders {
            if let Ok(response) = self.get_versions(loader, false).await {
                if let Some(latest) = response.latest {
                    summary.latest_versions.insert(name.to_string(), latest.id);
                }
            }
        }

        Ok(summary)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionSummary {
    pub total_loaders: usize,
    pub cached_loaders: usize,
    pub valid_cache_count: usize,
    pub expired_cache_count: usize,
    pub latest_versions: HashMap<String, String>,
    pub cache_status: HashMap<String, CacheInfo>,
}