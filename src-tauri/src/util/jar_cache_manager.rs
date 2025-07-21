use crate::models::version::LoaderType;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

pub struct JarCacheManager {
    cache_dir: PathBuf,
}

impl JarCacheManager {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory if it doesn't exist
        let jar_cache_dir = cache_dir.join("jars");
        if !jar_cache_dir.exists() {
            fs::create_dir_all(&jar_cache_dir)?;
        }

        Ok(Self {
            cache_dir: jar_cache_dir,
        })
    }

    /// Generate a unique cache key for a JAR file
    pub fn get_jar_cache_key(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> String {
        match loader {
            LoaderType::Vanilla => format!("vanilla-{}", minecraft_version),
            LoaderType::Fabric => {
                let loader_ver = loader_version.unwrap_or("unknown");
                format!("fabric-{}-{}", minecraft_version, loader_ver)
            }
            LoaderType::Forge => {
                let loader_ver = loader_version.unwrap_or("unknown");
                // Remove "forge-" prefix if it exists for consistent cache keys
                let clean_version = if loader_ver.starts_with("forge-") {
                    loader_ver.strip_prefix("forge-").unwrap_or(loader_ver)
                } else {
                    loader_ver
                };
                format!("forge-{}", clean_version)
            }
            LoaderType::NeoForge => {
                let loader_ver = loader_version.unwrap_or("unknown");
                // Remove "neoforge-" prefix if it exists for consistent cache keys
                let clean_version = if loader_ver.starts_with("neoforge-") {
                    loader_ver.strip_prefix("neoforge-").unwrap_or(loader_ver)
                } else {
                    loader_ver
                };
                format!("neoforge-{}", clean_version)
            }
            LoaderType::Paper => format!("paper-{}", minecraft_version),
            LoaderType::Quilt => {
                let loader_ver = loader_version.unwrap_or("unknown");
                format!("quilt-{}-{}", minecraft_version, loader_ver)
            }
        }
    }

    /// Get the filename for a JAR in cache
    pub fn get_jar_filename(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> String {
        match loader {
            LoaderType::Vanilla => format!("server-{}.jar", minecraft_version),
            LoaderType::Fabric => {
                let loader_ver = loader_version.unwrap_or("unknown");
                format!("fabric-server-{}-{}.jar", minecraft_version, loader_ver)
            }
            LoaderType::Forge => {
                let loader_ver = loader_version.unwrap_or("unknown");
                // If version already has "forge-" prefix, use it directly, otherwise add prefix
                if loader_ver.starts_with("forge-") {
                    format!("{}-installer.jar", loader_ver)
                } else {
                    format!("forge-{}-installer.jar", loader_ver)
                }
            }
            LoaderType::NeoForge => {
                let loader_ver = loader_version.unwrap_or("unknown");
                // If version already has "neoforge-" prefix, use it directly, otherwise add prefix
                if loader_ver.starts_with("neoforge-") {
                    format!("{}-installer.jar", loader_ver)
                } else {
                    format!("neoforge-{}-installer.jar", loader_ver)
                }
            }
            LoaderType::Paper => format!("paper-{}.jar", minecraft_version),
            LoaderType::Quilt => {
                let loader_ver = loader_version.unwrap_or("unknown");
                format!("quilt-server-{}-{}.jar", minecraft_version, loader_ver)
            }
        }
    }

    /// Get the path to a cached JAR file
    pub fn get_cached_jar_path(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> PathBuf {
        let cache_key = self.get_jar_cache_key(loader, minecraft_version, loader_version);
        let filename = self.get_jar_filename(loader, minecraft_version, loader_version);
        self.cache_dir.join(&cache_key).join(&filename)
    }

    /// Check if a JAR is cached
    pub fn is_jar_cached(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> bool {
        let cached_path = self.get_cached_jar_path(loader, minecraft_version, loader_version);
        cached_path.exists()
    }

    /// Store a JAR file in cache
    pub fn cache_jar(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
        jar_data: &[u8],
    ) -> Result<PathBuf> {
        let cache_key = self.get_jar_cache_key(loader, minecraft_version, loader_version);
        let cache_subdir = self.cache_dir.join(&cache_key);
        
        // Create subdirectory for this specific version
        if !cache_subdir.exists() {
            fs::create_dir_all(&cache_subdir)?;
        }

        let filename = self.get_jar_filename(loader, minecraft_version, loader_version);
        let cached_path = cache_subdir.join(&filename);

        // Write the JAR data to cache
        fs::write(&cached_path, jar_data)?;

        println!("JAR cached successfully: {:?}", cached_path);
        Ok(cached_path)
    }

    /// Copy a cached JAR to a server directory
    pub fn copy_cached_jar_to_server(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
        server_path: &PathBuf,
    ) -> Result<PathBuf> {
        let cached_path = self.get_cached_jar_path(loader, minecraft_version, loader_version);
        
        if !cached_path.exists() {
            return Err(anyhow!("JAR not found in cache: {:?}", cached_path));
        }

        // Create server directory if it doesn't exist
        if !server_path.exists() {
            fs::create_dir_all(server_path)?;
        }

        let filename = self.get_jar_filename(loader, minecraft_version, loader_version);
        let server_jar_path = server_path.join(&filename);

        // Copy the cached JAR to the server directory
        fs::copy(&cached_path, &server_jar_path)?;

        println!("JAR copied from cache to server: {:?} -> {:?}", cached_path, server_jar_path);
        Ok(server_jar_path)
    }

    /// Get cached JAR data
    pub fn get_cached_jar_data(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> Result<Vec<u8>> {
        let cached_path = self.get_cached_jar_path(loader, minecraft_version, loader_version);
        
        if !cached_path.exists() {
            return Err(anyhow!("JAR not found in cache: {:?}", cached_path));
        }

        let data = fs::read(&cached_path)?;
        Ok(data)
    }

    /// Clear cache for a specific JAR
    pub fn clear_jar_cache(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
    ) -> Result<()> {
        let cache_key = self.get_jar_cache_key(loader, minecraft_version, loader_version);
        let cache_subdir = self.cache_dir.join(&cache_key);
        
        if cache_subdir.exists() {
            fs::remove_dir_all(&cache_subdir)?;
            println!("Cleared JAR cache for: {}", cache_key);
        }
        
        Ok(())
    }

    /// Clear all JAR cache
    pub fn clear_all_jar_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
            println!("Cleared all JAR cache");
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats {
            total_cached_jars: 0,
            total_cache_size: 0,
            cached_loaders: Vec::new(),
        };

        if !self.cache_dir.exists() {
            return Ok(stats);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                stats.cached_loaders.push(dir_name);

                // Count JARs in this subdirectory
                for jar_entry in fs::read_dir(entry.path())? {
                    let jar_entry = jar_entry?;
                    if jar_entry.file_type()?.is_file() && 
                       jar_entry.file_name().to_string_lossy().ends_with(".jar") {
                        stats.total_cached_jars += 1;
                        if let Ok(metadata) = jar_entry.metadata() {
                            stats.total_cache_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    pub total_cached_jars: usize,
    pub total_cache_size: u64,
    pub cached_loaders: Vec<String>,
}