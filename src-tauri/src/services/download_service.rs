use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use crate::models::version::LoaderType;
use crate::util::JarCacheManager;

pub struct DownloadService {
    client: Client,
    jar_cache: JarCacheManager,
}

impl DownloadService {
    pub fn new() -> Result<Self> {
        let cache_dir = PathBuf::from("storage/version_cache");
        let jar_cache = JarCacheManager::new(cache_dir)?;
        
        Ok(Self {
            client: Client::new(),
            jar_cache,
        })
    }

    pub async fn download_server_jar(
        &self,
        loader: LoaderType,
        minecraft_version: String,
        loader_version: Option<String>,
        server_path: PathBuf,
    ) -> Result<PathBuf> {
        let loader_version_ref = loader_version.as_deref();
        
        // Check if JAR is cached first
        if self.jar_cache.is_jar_cached(&loader, &minecraft_version, loader_version_ref) {
            println!("JAR found in cache, copying to server: {:?}", server_path);
            return self.jar_cache.copy_cached_jar_to_server(&loader, &minecraft_version, loader_version_ref, &server_path);
        }

        println!("JAR not in cache, downloading...");
        
        let download_url = self.get_download_url(&loader, &minecraft_version, &loader_version).await?;
        let jar_name = self.get_jar_filename(&loader, &minecraft_version, &loader_version);

        println!("Downloading {} from: {}", jar_name, download_url);

        // Download the JAR file
        let response = self.client.get(&download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download JAR: HTTP {}", response.status()));
        }

        let bytes = response.bytes().await?;

        // Cache the JAR first
        println!("Caching downloaded JAR...");
        self.jar_cache.cache_jar(&loader, &minecraft_version, loader_version_ref, &bytes)?;

        // Then copy it to the server directory
        println!("Copying cached JAR to server: {:?}", server_path);
        let jar_path = self.jar_cache.copy_cached_jar_to_server(&loader, &minecraft_version, loader_version_ref, &server_path)?;

        println!("Successfully downloaded and cached JAR: {:?}", jar_path);
        Ok(jar_path)
    }

    async fn get_download_url(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: &Option<String>,
    ) -> Result<String> {
        match loader {
            LoaderType::Vanilla => {
                self.get_vanilla_download_url(minecraft_version).await
            }
            LoaderType::Fabric => {
                let loader_ver = loader_version.as_ref()
                    .ok_or_else(|| anyhow!("Fabric loader version is required"))?;
                self.get_fabric_download_url(minecraft_version, loader_ver).await
            }
            LoaderType::Forge => {
                let loader_ver = loader_version.as_ref()
                    .ok_or_else(|| anyhow!("Forge loader version is required"))?;
                self.get_forge_download_url(minecraft_version, loader_ver).await
            }
            LoaderType::NeoForge => {
                let loader_ver = loader_version.as_ref()
                    .ok_or_else(|| anyhow!("NeoForge loader version is required"))?;
                self.get_neoforge_download_url(minecraft_version, loader_ver).await
            }
            LoaderType::Paper => {
                self.get_paper_download_url(minecraft_version).await
            }
            LoaderType::Quilt => {
                let loader_ver = loader_version.as_ref()
                    .ok_or_else(|| anyhow!("Quilt loader version is required"))?;
                self.get_quilt_download_url(minecraft_version, loader_ver).await
            }
        }
    }

    async fn get_vanilla_download_url(&self, minecraft_version: &str) -> Result<String> {
        // Get version manifest
        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest: serde_json::Value = self.client.get(manifest_url).send().await?.json().await?;
        
        // Find the specific version
        let versions = manifest["versions"].as_array()
            .ok_or_else(|| anyhow!("Invalid version manifest"))?;
        
        let version_info = versions.iter()
            .find(|v| v["id"].as_str() == Some(minecraft_version))
            .ok_or_else(|| anyhow!("Minecraft version {} not found", minecraft_version))?;
        
        let version_url = version_info["url"].as_str()
            .ok_or_else(|| anyhow!("Version URL not found"))?;
        
        // Get version details
        let version_details: serde_json::Value = self.client.get(version_url).send().await?.json().await?;
        
        // Get server JAR URL
        let server_url = version_details["downloads"]["server"]["url"].as_str()
            .ok_or_else(|| anyhow!("Server JAR URL not found for version {}", minecraft_version))?;
        
        Ok(server_url.to_string())
    }

    async fn get_fabric_download_url(&self, minecraft_version: &str, loader_version: &str) -> Result<String> {
        Ok(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.1/server/jar",
            minecraft_version, loader_version
        ))
    }

    async fn get_forge_download_url(&self, minecraft_version: &str, loader_version: &str) -> Result<String> {
        // Remove "forge-" prefix if it exists (version service adds it, but we need the raw version for URL)
        let clean_version = if loader_version.starts_with("forge-") {
            loader_version.strip_prefix("forge-").unwrap_or(loader_version)
        } else {
            loader_version
        };
        
        Ok(format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
            clean_version, clean_version
        ))
    }

    async fn get_neoforge_download_url(&self, minecraft_version: &str, loader_version: &str) -> Result<String> {
        // Remove "neoforge-" prefix if it exists (version service adds it, but we need the raw version for URL)
        let clean_version = if loader_version.starts_with("neoforge-") {
            loader_version.strip_prefix("neoforge-").unwrap_or(loader_version)
        } else {
            loader_version
        };
        
        Ok(format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            clean_version, clean_version
        ))
    }

    async fn get_paper_download_url(&self, minecraft_version: &str) -> Result<String> {
        // Get latest build for the version
        let builds_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", minecraft_version);
        let builds_response: serde_json::Value = self.client.get(&builds_url).send().await?.json().await?;
        
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

    async fn get_quilt_download_url(&self, minecraft_version: &str, loader_version: &str) -> Result<String> {
        Ok(format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/1.0.0/server/jar",
            minecraft_version, loader_version
        ))
    }

    fn get_jar_filename(
        &self,
        loader: &LoaderType,
        minecraft_version: &str,
        loader_version: &Option<String>,
    ) -> String {
        match loader {
            LoaderType::Vanilla => format!("server-{}.jar", minecraft_version),
            LoaderType::Fabric => {
                let version_str = loader_version.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
                format!("fabric-server-{}-{}.jar", minecraft_version, version_str)
            },
            LoaderType::Forge => {
                let version_str = loader_version.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
                // If version already has "forge-" prefix, use it directly, otherwise add prefix
                if version_str.starts_with("forge-") {
                    format!("{}-installer.jar", version_str)
                } else {
                    format!("forge-{}-installer.jar", version_str)
                }
            },
            LoaderType::NeoForge => {
                let version_str = loader_version.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
                // If version already has "neoforge-" prefix, use it directly, otherwise add prefix
                if version_str.starts_with("neoforge-") {
                    format!("{}-installer.jar", version_str)
                } else {
                    format!("neoforge-{}-installer.jar", version_str)
                }
            },
            LoaderType::Paper => format!("paper-{}.jar", minecraft_version),
            LoaderType::Quilt => {
                let version_str = loader_version.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
                format!("quilt-server-{}-{}.jar", minecraft_version, version_str)
            },
        }
    }
}