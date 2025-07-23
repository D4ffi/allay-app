use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::LoaderType;
use crate::util::JarCacheManager;

/// Quilt strategy
pub struct QuiltStrategy;

#[async_trait]
impl ModLoaderStrategy for QuiltStrategy {
    // Custom implementation for Quilt since it downloads JSON profiles, not JARs
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
        
        // Check if profile JSON is cached first
        if jar_cache.is_jar_cached(loader_type, minecraft_version, loader_version_opt) {
            println!("Quilt profile found in cache, copying to server: {:?}", server_path);
            return jar_cache.copy_cached_jar_to_server(loader_type, minecraft_version, loader_version_opt, server_path);
        }

        println!("Quilt profile not in cache, downloading...");
        
        let download_url = self.get_download_url(client, minecraft_version, loader_version).await?;
        let profile_name = self.get_filename(minecraft_version, loader_version);

        println!("Downloading {} from: {}", profile_name, download_url);

        // Download the profile JSON
        let response = client.get(&download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download Quilt profile: HTTP {}", response.status()));
        }

        let bytes = response.bytes().await?;

        // Cache the profile first
        println!("Caching downloaded Quilt profile...");
        jar_cache.cache_jar(loader_type, minecraft_version, loader_version_opt, &bytes)?;

        // Then copy it to the server directory
        println!("Copying cached Quilt profile to server: {:?}", server_path);
        let profile_path = jar_cache.copy_cached_jar_to_server(loader_type, minecraft_version, loader_version_opt, server_path)?;

        println!("Successfully downloaded and cached Quilt profile: {:?}", profile_path);
        Ok(profile_path)
    }

    async fn get_download_url(&self, _client: &Client, minecraft_version: &str, loader_version: &str) -> Result<String> {
        let actual_loader_version = if loader_version.starts_with("quilt-") {
            let without_prefix = loader_version.strip_prefix("quilt-").unwrap_or(loader_version);
            if let Some(dash_pos) = without_prefix.find('-') {
                &without_prefix[..dash_pos]
            } else {
                without_prefix
            }
        } else {
            loader_version
        };
        
        let profile_url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/server/json",
            minecraft_version, actual_loader_version
        );
        
        Ok(profile_url)
    }
    
    fn get_filename(&self, _minecraft_version: &str, _loader_version: &str) -> String {
        "quilt-server-profile.json".to_string()
    }
    
    async fn setup_server(&self, client: &Client, server_path: &PathBuf, minecraft_version: &str, _loader_version: &str) -> Result<()> {
        let profile_json = server_path.join("quilt-server-profile.json");
        if !profile_json.exists() {
            return Err(anyhow!("Quilt server profile not found: {:?}", profile_json));
        }

        // Check if libraries are already downloaded
        let libraries_dir = server_path.join("libraries");
        if libraries_dir.exists() {
            println!("Quilt server libraries already installed");
            return Ok(());
        }

        println!("Installing Quilt server libraries...");

        // Read and parse the profile JSON
        let profile_content = fs::read_to_string(&profile_json)?;
        let profile: serde_json::Value = serde_json::from_str(&profile_content)?;

        // Create libraries directory
        fs::create_dir_all(&libraries_dir)?;

        // Download all required libraries
        if let Some(libraries) = profile["libraries"].as_array() {
            for library in libraries {
                if let (Some(name), Some(url)) = (library["name"].as_str(), library["url"].as_str()) {
                    self.download_library(client, name, url, &libraries_dir).await?;
                }
            }
        }

        // Download vanilla server JAR if needed
        let vanilla_jar = server_path.join("server.jar");
        if !vanilla_jar.exists() {
            println!("Downloading vanilla Minecraft server for Quilt...");
            let vanilla_url = self.get_vanilla_server_url(client, minecraft_version).await?;
            let response = client.get(&vanilla_url).send().await?;
            let bytes = response.bytes().await?;
            fs::write(&vanilla_jar, &bytes)?;
            println!("Vanilla server JAR downloaded: {:?}", vanilla_jar);
        }

        println!("Quilt server setup completed");
        Ok(())
    }
    
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>> {
        // Read Quilt profile to get mainClass and libraries
        let profile_json = server_path.join("quilt-server-profile.json");
        if !profile_json.exists() {
            return Err(anyhow!("Quilt server profile not found: {:?}", profile_json));
        }

        let profile_content = fs::read_to_string(&profile_json)?;
        let profile: serde_json::Value = serde_json::from_str(&profile_content)?;

        // Get mainClass
        let main_class = profile["mainClass"].as_str()
            .ok_or_else(|| anyhow!("mainClass not found in Quilt profile"))?;

        // Build classpath with all libraries
        let mut classpath = Vec::new();
        
        // Add vanilla server.jar first
        let vanilla_jar = server_path.join("server.jar");
        if vanilla_jar.exists() {
            classpath.push("server.jar".to_string());
        }

        // Add all Quilt libraries
        if let Some(libraries) = profile["libraries"].as_array() {
            for library in libraries {
                if let Some(name) = library["name"].as_str() {
                    let jar_path = self.get_library_jar_path(name)?;
                    classpath.push(format!("libraries/{}", jar_path));
                }
            }
        }

        let classpath_str = classpath.join(if cfg!(windows) { ";" } else { ":" });

        // Build the complete command
        let args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-cp".to_string(),
            classpath_str,
            main_class.to_string(),
            "nogui".to_string(),
        ];

        Ok(args)
    }
}

impl QuiltStrategy {
    async fn download_library(&self, client: &Client, name: &str, base_url: &str, libraries_dir: &PathBuf) -> Result<()> {
        // Convert Maven coordinate to file path
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid library name format: {}", name));
        }
        
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        
        let jar_name = format!("{}-{}.jar", artifact, version);
        let relative_path = format!("{}/{}/{}/{}", group, artifact, version, jar_name);
        let download_url = format!("{}{}", base_url.trim_end_matches('/'), relative_path);
        
        // Create the directory structure
        let lib_dir = libraries_dir.join(&group).join(artifact).join(version);
        fs::create_dir_all(&lib_dir)?;
        
        let jar_path = lib_dir.join(&jar_name);
        
        // Skip if already exists
        if jar_path.exists() {
            println!("Library already exists: {}", jar_name);
            return Ok(());
        }
        
        println!("Downloading library: {} from {}", jar_name, download_url);
        
        let response = client.get(&download_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download library {}: HTTP {}", name, response.status()));
        }
        
        let bytes = response.bytes().await?;
        fs::write(&jar_path, &bytes)?;
        
        println!("Downloaded library: {:?}", jar_path);
        Ok(())
    }

    async fn get_vanilla_server_url(&self, client: &Client, minecraft_version: &str) -> Result<String> {
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

    fn get_library_jar_path(&self, name: &str) -> Result<String> {
        // Convert Maven coordinate to relative JAR path
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid library name format: {}", name));
        }
        
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        
        let jar_name = format!("{}-{}.jar", artifact, version);
        Ok(format!("{}/{}/{}/{}", group, artifact, version, jar_name))
    }
}