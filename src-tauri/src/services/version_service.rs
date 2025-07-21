use crate::models::version::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::collections::HashMap;

pub struct VersionService {
    client: Client,
}

impl VersionService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_versions(&self, loader: LoaderType) -> Result<VersionResponse> {
        self.get_versions_for_minecraft(loader, None).await
    }

    pub async fn get_versions_for_minecraft(&self, loader: LoaderType, minecraft_version: Option<String>) -> Result<VersionResponse> {
        match loader {
            LoaderType::Vanilla => self.get_vanilla_versions().await,
            LoaderType::Fabric => self.get_fabric_versions(minecraft_version).await,
            LoaderType::Forge => self.get_forge_versions(minecraft_version).await,
            LoaderType::NeoForge => self.get_neoforge_versions(minecraft_version).await,
            LoaderType::Paper => self.get_paper_versions(minecraft_version).await,
            LoaderType::Quilt => self.get_quilt_versions(minecraft_version).await,
        }
    }

    async fn get_vanilla_versions(&self) -> Result<VersionResponse> {
        let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let response: MojangVersionManifest = self.client.get(url).send().await?.json().await?;

        let mut versions = Vec::new();
        let latest_release = response.latest.release.clone();
        let latest_snapshot = response.latest.snapshot.clone();

        // Get ALL release versions (from 1.0 to the latest)
        let release_versions: Vec<_> = response
            .versions
            .iter()
            .filter(|v| v.version_type == "release")
            .collect();

        for version in release_versions {
            let is_latest = version.id == latest_release;
            let minecraft_version = MinecraftVersion {
                id: version.id.clone(),
                version_type: VersionType::Release,
                loader: LoaderType::Vanilla,
                release_time: version.release_time,
                latest: is_latest,
                recommended: is_latest, // For vanilla, the latest release is recommended
                minecraft_version: None,
            };
            versions.push(minecraft_version);
        }

        let latest = versions.iter().find(|v| v.latest).cloned();
        let recommended = versions.iter().find(|v| v.recommended).cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }

    async fn get_fabric_versions(&self, minecraft_version: Option<String>) -> Result<VersionResponse> {
        // Get game versions
        let game_url = "https://meta.fabricmc.net/v2/versions";
        let game_response: FabricVersions = self.client.get(game_url).send().await?.json().await?;

        // Get loader versions
        let loader_url = "https://meta.fabricmc.net/v2/versions/loader";
        let loader_response: Vec<FabricLoaderVersion> = self.client.get(loader_url).send().await?.json().await?;

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
                    let minecraft_version = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: i == 0,
                        recommended: loader.stable,
                        minecraft_version: Some(game_version.version.clone()),
                    };
                    versions.push(minecraft_version);
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
                    let minecraft_version = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: i == 0 && loader.version == latest_loader.unwrap().version,
                        recommended: true,
                        minecraft_version: Some(game_version.version.clone()),
                    };
                    versions.push(minecraft_version);
                }
            }

            // Add the latest loader with the latest MC version if different
            if let (Some(latest_loader), Some(latest_game)) = (latest_loader, game_response.game.first()) {
                if stable_loader.is_none() || latest_loader.version != stable_loader.unwrap().version {
                    let version_id = format!("fabric-{}-{}", latest_loader.version, latest_game.version);
                    let minecraft_version = MinecraftVersion {
                        id: version_id,
                        version_type: VersionType::Release,
                        loader: LoaderType::Fabric,
                        release_time: Utc::now(),
                        latest: true,
                        recommended: false,
                        minecraft_version: Some(latest_game.version.clone()),
                    };
                    versions.insert(0, minecraft_version);
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

    async fn get_forge_versions(&self, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
        let response = self.client.get(url).send().await?.text().await?;

        // Parse XML manually (simple approach for this case)
        let mut versions = Vec::new();
        let version_regex = regex::Regex::new(r"<version>([^<]+)</version>").unwrap();
        
        let mut forge_versions: Vec<String> = version_regex
            .captures_iter(&response)
            .map(|cap| cap[1].to_string())
            .collect();

        // Sort versions by MC version (newest first)
        forge_versions.sort_by(|a, b| {
            let a_mc = a.split('-').next().unwrap_or("");
            let b_mc = b.split('-').next().unwrap_or("");
            b_mc.cmp(a_mc)
        });

        if let Some(target_mc_version) = minecraft_version {
            // Filter for specific MC version and get ALL forge versions for that MC version
            for (i, version_str) in forge_versions.iter().enumerate() {
                let parts: Vec<&str> = version_str.split('-').collect();
                if parts.len() >= 2 {
                    let mc_version = parts[0];
                    
                    if mc_version == target_mc_version {
                        let minecraft_version = MinecraftVersion {
                            id: format!("forge-{}", version_str),
                            version_type: VersionType::Release,
                            loader: LoaderType::Forge,
                            release_time: Utc::now(),
                            latest: i == 0,
                            recommended: i == 0,
                            minecraft_version: Some(mc_version.to_string()),
                        };
                        versions.push(minecraft_version);
                    }
                }
            }
        } else {
            // Group by minecraft version and take the latest from each (no 5 limit)
            let mut mc_versions_seen: HashMap<String, bool> = HashMap::new();

            for (overall_index, version_str) in forge_versions.iter().enumerate() {
                let parts: Vec<&str> = version_str.split('-').collect();
                if parts.len() >= 2 {
                    let mc_version = parts[0];

                    if !mc_versions_seen.contains_key(mc_version) {
                        mc_versions_seen.insert(mc_version.to_string(), true);
                        
                        let minecraft_version = MinecraftVersion {
                            id: format!("forge-{}", version_str),
                            version_type: VersionType::Release,
                            loader: LoaderType::Forge,
                            release_time: Utc::now(),
                            latest: overall_index == 0,
                            recommended: overall_index == 0,
                            minecraft_version: Some(mc_version.to_string()),
                        };
                        versions.push(minecraft_version);
                    }
                }
            }
        }

        let latest = versions.first().cloned();
        let recommended = versions.first().cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }

    async fn get_neoforge_versions(&self, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let url = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
        let response: NeoForgeVersions = self.client.get(url).send().await?.json().await?;

        let mut versions = Vec::new();
        
        // Filter and sort versions (newest first)
        // Note: NeoForge marks stable releases as "beta", so we only exclude alpha and snapshot versions
        let mut neoforge_versions: Vec<String> = response.versions
            .into_iter()
            .filter(|v| !v.contains("alpha") && !v.contains("snapshot"))
            .collect();
        
        neoforge_versions.sort_by(|a, b| {
            // Parse version numbers for proper semantic versioning comparison
            let parse_version = |v: &str| -> Vec<u32> {
                v.replace("-beta", "")
                    .split('.')
                    .filter_map(|s| s.parse().ok())
                    .collect()
            };
            
            let a_parts = parse_version(a);
            let b_parts = parse_version(b);
            
            // Compare version parts (newest first)
            for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
                let a_part = a_parts.get(i).unwrap_or(&0);
                let b_part = b_parts.get(i).unwrap_or(&0);
                match b_part.cmp(a_part) {
                    std::cmp::Ordering::Equal => continue,
                    other => return other,
                }
            }
            std::cmp::Ordering::Equal
        });

        if let Some(target_mc_version) = minecraft_version {
            // Filter for specific MC version and get ALL neoforge versions for that MC version
            for (i, version_str) in neoforge_versions.iter().enumerate() {
                // More precise Minecraft version mapping for NeoForge
                let mc_version = if version_str.starts_with("21.") {
                    // NeoForge 21.x targets Minecraft 1.21.x
                    "1.21"
                } else if version_str.starts_with("20.6") {
                    // NeoForge 20.6.x targets Minecraft 1.20.6
                    "1.20.6"
                } else if version_str.starts_with("20.4") {
                    // NeoForge 20.4.x targets Minecraft 1.20.4
                    "1.20.4"
                } else if version_str.starts_with("20.2") {
                    // NeoForge 20.2.x targets Minecraft 1.20.2
                    "1.20.2"
                } else if version_str.starts_with("20.") {
                    // Other 20.x versions target Minecraft 1.20.1
                    "1.20.1"
                } else {
                    "unknown"
                };

                if mc_version == target_mc_version || (target_mc_version.starts_with(mc_version) && mc_version != "unknown") {
                    let minecraft_version = MinecraftVersion {
                        id: format!("neoforge-{}", version_str),
                        version_type: VersionType::Release,
                        loader: LoaderType::NeoForge,
                        release_time: Utc::now(),
                        latest: i == 0,
                        recommended: i == 0,
                        minecraft_version: Some(mc_version.to_string()),
                    };
                    versions.push(minecraft_version);
                }
            }
        } else {
            // Take all versions (no 5 limit)
            for (i, version_str) in neoforge_versions.iter().enumerate() {
                // More precise Minecraft version mapping for NeoForge
                let mc_version = if version_str.starts_with("21.") {
                    // NeoForge 21.x targets Minecraft 1.21.x
                    "1.21"
                } else if version_str.starts_with("20.6") {
                    // NeoForge 20.6.x targets Minecraft 1.20.6
                    "1.20.6"
                } else if version_str.starts_with("20.4") {
                    // NeoForge 20.4.x targets Minecraft 1.20.4
                    "1.20.4"
                } else if version_str.starts_with("20.2") {
                    // NeoForge 20.2.x targets Minecraft 1.20.2
                    "1.20.2"
                } else if version_str.starts_with("20.") {
                    // Other 20.x versions target Minecraft 1.20.1
                    "1.20.1"
                } else {
                    "unknown"
                };

                let minecraft_version = MinecraftVersion {
                    id: format!("neoforge-{}", version_str),
                    version_type: VersionType::Release,
                    loader: LoaderType::NeoForge,
                    release_time: Utc::now(),
                    latest: i == 0,
                    recommended: i == 0,
                    minecraft_version: Some(mc_version.to_string()),
                };
                versions.push(minecraft_version);
            }
        }

        let latest = versions.first().cloned();
        let recommended = versions.first().cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }

    async fn get_paper_versions(&self, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let url = "https://api.papermc.io/v2/projects/paper";
        let response: PaperProject = self.client.get(url).send().await?.json().await?;

        let mut versions = Vec::new();
        
        if let Some(target_mc_version) = minecraft_version {
            // Filter for specific MC version
            if response.versions.contains(&target_mc_version) {
                let minecraft_version = MinecraftVersion {
                    id: format!("paper-{}", target_mc_version),
                    version_type: VersionType::Release,
                    loader: LoaderType::Paper,
                    release_time: Utc::now(),
                    latest: true,
                    recommended: true,
                    minecraft_version: Some(target_mc_version.clone()),
                };
                versions.push(minecraft_version);
            }
        } else {
            // Take all minecraft versions supported by Paper (no 5 limit)
            for (i, mc_version) in response.versions.iter().rev().enumerate() {
                let minecraft_version = MinecraftVersion {
                    id: format!("paper-{}", mc_version),
                    version_type: VersionType::Release,
                    loader: LoaderType::Paper,
                    release_time: Utc::now(),
                    latest: i == 0,
                    recommended: i == 0,
                    minecraft_version: Some(mc_version.clone()),
                };
                versions.push(minecraft_version);
            }
        }

        let latest = versions.first().cloned();
        let recommended = versions.first().cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }

    async fn get_quilt_versions(&self, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let base_url = "https://meta.quiltmc.org/v3/versions";
        let response: QuiltVersions = self.client.get(base_url).send().await?.json().await?;

        let mut versions = Vec::new();
        
        if let Some(target_mc_version) = minecraft_version {
            // Check if the target MC version exists in game versions
            let target_game_version = response.game.iter()
                .find(|v| v.version == target_mc_version);
            
            if let Some(game_version) = target_game_version {
                // Get loader versions for this game version (similar to Fabric pattern)
                let loader_url = format!("{}/loader/{}", base_url, target_mc_version);
                let loader_response: Vec<serde_json::Value> = self.client.get(&loader_url).send().await?.json().await?;
                
                // Create versions for each loader version (using Fabric-like pattern)
                for (i, loader) in loader_response.iter().enumerate() {
                    if let Some(loader_version) = loader["version"].as_str() {
                        let version_id = format!("quilt-{}-{}", loader_version, game_version.version);
                        let minecraft_version_obj = MinecraftVersion {
                            id: version_id,
                            version_type: VersionType::Release,
                            loader: LoaderType::Quilt,
                            release_time: Utc::now(),
                            latest: i == 0,
                            recommended: i == 0,
                            minecraft_version: Some(game_version.version.clone()),
                        };
                        versions.push(minecraft_version_obj);
                    }
                }
            }
        } else {
            // Return only stable game versions (like Fabric does)
            // The loader versions will be fetched when a specific MC version is selected
            let stable_game_versions: Vec<_> = response
                .game
                .iter()
                .filter(|v| v.stable)
                .collect();

            for (i, game_version) in stable_game_versions.iter().enumerate() {
                let minecraft_version_obj = MinecraftVersion {
                    id: format!("quilt-{}", game_version.version),
                    version_type: VersionType::Release,
                    loader: LoaderType::Quilt,
                    release_time: Utc::now(),
                    latest: i == 0,
                    recommended: i == 0,
                    minecraft_version: Some(game_version.version.clone()),
                };
                versions.push(minecraft_version_obj);
            }
        }

        let latest = versions.first().cloned();
        let recommended = versions.first().cloned();

        Ok(VersionResponse {
            latest,
            recommended,
            versions,
        })
    }
}