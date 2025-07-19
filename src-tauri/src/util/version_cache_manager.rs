use crate::models::version::*;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct VersionCacheManager {
    cache_dir: PathBuf,
    cache_duration: Duration,
}

impl VersionCacheManager {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            cache_dir,
            cache_duration: Duration::hours(6), // Cache for 6 hours
        })
    }

    pub fn get_cache_file_path(&self, loader: &LoaderType) -> PathBuf {
        let filename = match loader {
            LoaderType::Vanilla => "vanilla_versions.json",
            LoaderType::Fabric => "fabric_versions.json",
            LoaderType::Forge => "forge_versions.json",
            LoaderType::NeoForge => "neoforge_versions.json",
            LoaderType::Paper => "paper_versions.json",
            LoaderType::Quilt => "quilt_versions.json",
        };
        self.cache_dir.join(filename)
    }

    pub fn is_cache_valid(&self, loader: &LoaderType) -> Result<bool> {
        let cache_file = self.get_cache_file_path(loader);
        
        if !cache_file.exists() {
            return Ok(false);
        }

        let cache_data = self.load_cache(loader)?;
        if let Some(cache) = cache_data {
            Ok(cache.expires_at > Utc::now())
        } else {
            Ok(false)
        }
    }

    pub fn load_cache(&self, loader: &LoaderType) -> Result<Option<VersionCache>> {
        let cache_file = self.get_cache_file_path(loader);
        
        if !cache_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(cache_file)?;
        let cache: VersionCache = serde_json::from_str(&content)?;
        
        // Check if cache is expired
        if cache.expires_at <= Utc::now() {
            return Ok(None);
        }

        Ok(Some(cache))
    }

    pub fn save_cache(&self, loader: LoaderType, versions: Vec<MinecraftVersion>) -> Result<()> {
        let now = Utc::now();
        let cache = VersionCache {
            loader: loader.clone(),
            versions,
            last_updated: now,
            expires_at: now + self.cache_duration,
        };

        let cache_file = self.get_cache_file_path(&loader);
        let content = serde_json::to_string_pretty(&cache)?;
        fs::write(cache_file, content)?;

        Ok(())
    }

    pub fn clear_cache(&self, loader: &LoaderType) -> Result<()> {
        let cache_file = self.get_cache_file_path(loader);
        if cache_file.exists() {
            fs::remove_file(cache_file)?;
        }
        Ok(())
    }

    pub fn clear_all_cache(&self) -> Result<()> {
        let loaders = vec![
            LoaderType::Vanilla,
            LoaderType::Fabric,
            LoaderType::Forge,
            LoaderType::NeoForge,
            LoaderType::Paper,
            LoaderType::Quilt,
        ];

        for loader in loaders {
            self.clear_cache(&loader)?;
        }

        Ok(())
    }

    pub fn get_cache_info(&self) -> Result<HashMap<String, CacheInfo>> {
        let mut info = HashMap::new();
        let loaders = vec![
            ("vanilla", LoaderType::Vanilla),
            ("fabric", LoaderType::Fabric),
            ("forge", LoaderType::Forge),
            ("neoforge", LoaderType::NeoForge),
            ("paper", LoaderType::Paper),
            ("quilt", LoaderType::Quilt),
        ];

        for (name, loader) in loaders {
            let cache_file = self.get_cache_file_path(&loader);
            let cache_info = if cache_file.exists() {
                match self.load_cache(&loader) {
                    Ok(Some(cache)) => CacheInfo {
                        exists: true,
                        last_updated: Some(cache.last_updated),
                        expires_at: Some(cache.expires_at),
                        valid: cache.expires_at > Utc::now(),
                        version_count: cache.versions.len(),
                    },
                    _ => CacheInfo {
                        exists: true,
                        last_updated: None,
                        expires_at: None,
                        valid: false,
                        version_count: 0,
                    }
                }
            } else {
                CacheInfo {
                    exists: false,
                    last_updated: None,
                    expires_at: None,
                    valid: false,
                    version_count: 0,
                }
            };

            info.insert(name.to_string(), cache_info);
        }

        Ok(info)
    }

    pub fn cleanup_expired_cache(&self) -> Result<Vec<String>> {
        let mut cleaned_loaders = Vec::new();
        let loaders = vec![
            ("vanilla", LoaderType::Vanilla),
            ("fabric", LoaderType::Fabric),
            ("forge", LoaderType::Forge),
            ("neoforge", LoaderType::NeoForge),
            ("paper", LoaderType::Paper),
            ("quilt", LoaderType::Quilt),
        ];

        for (name, loader) in loaders {
            if let Ok(cache) = self.load_cache(&loader) {
                if cache.is_none() {
                    // Cache is expired or invalid, remove it
                    if let Ok(()) = self.clear_cache(&loader) {
                        cleaned_loaders.push(name.to_string());
                    }
                }
            }
        }

        Ok(cleaned_loaders)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheInfo {
    pub exists: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub valid: bool,
    pub version_count: usize,
}