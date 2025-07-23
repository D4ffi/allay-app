use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use chrono::Utc;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::{LoaderType, VersionResponse, MinecraftVersion, VersionType, NeoForgeVersions};
use crate::util::JarCacheManager;

/// NeoForge strategy
pub struct NeoForgeStrategy;

#[async_trait]
impl ModLoaderStrategy for NeoForgeStrategy {
    async fn get_versions(&self, client: &Client, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let url = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
        let response: NeoForgeVersions = client.get(url).send().await?.json().await?;

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
                    let minecraft_version_obj = MinecraftVersion {
                        id: format!("neoforge-{}", version_str),
                        version_type: VersionType::Release,
                        loader: LoaderType::NeoForge,
                        release_time: Utc::now(),
                        latest: i == 0,
                        recommended: i == 0,
                        minecraft_version: Some(mc_version.to_string()),
                    };
                    versions.push(minecraft_version_obj);
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

                let minecraft_version_obj = MinecraftVersion {
                    id: format!("neoforge-{}", version_str),
                    version_type: VersionType::Release,
                    loader: LoaderType::NeoForge,
                    release_time: Utc::now(),
                    latest: i == 0,
                    recommended: i == 0,
                    minecraft_version: Some(mc_version.to_string()),
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
    
    async fn get_download_url(&self, _client: &Client, minecraft_version: &str, loader_version: &str) -> Result<String> {
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
    
    fn get_filename(&self, _minecraft_version: &str, loader_version: &str) -> String {
        if loader_version.starts_with("neoforge-") {
            format!("{}-installer.jar", loader_version)
        } else {
            format!("neoforge-{}-installer.jar", loader_version)
        }
    }
    
    async fn setup_server(&self, _client: &Client, server_path: &PathBuf, _minecraft_version: &str, loader_version: &str) -> Result<()> {
        let installer_name = if loader_version.starts_with("neoforge-") {
            format!("{}-installer.jar", loader_version)
        } else {
            format!("neoforge-{}-installer.jar", loader_version)
        };
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("NeoForge installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let run_script = server_path.join("run.sh");
        let server_jar = if loader_version.starts_with("neoforge-") {
            server_path.join(format!("{}-server.jar", loader_version))
        } else {
            server_path.join(format!("neoforge-{}-server.jar", loader_version))
        };
        
        if run_script.exists() || server_jar.exists() {
            println!("NeoForge server already installed");
            return Ok(());
        }

        println!("Installing NeoForge server...");
        
        let installer_filename = installer_path.file_name()
            .ok_or_else(|| anyhow!("Invalid installer filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid installer filename encoding"))?;

        let output = Command::new("java")
            .args(&[
                "-jar", 
                installer_filename,
                "--installServer"
            ])
            .current_dir(server_path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!("Java is not installed or not found in PATH. Please install Java to run Minecraft servers.")
                } else {
                    anyhow!("Failed to execute Java: {}", e)
                }
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("NeoForge installation failed: {}", error));
        }

        println!("NeoForge server installed successfully");
        Ok(())
    }
    
    fn build_start_command(&self, server_path: &PathBuf, memory_gb: u32, _min_memory_gb: u32) -> Result<Vec<String>> {
        // Check OS and use appropriate script
        let (script_path, script_command) = if cfg!(windows) {
            (server_path.join("run.bat"), "run.bat".to_string())
        } else {
            (server_path.join("run.sh"), "./run.sh".to_string())
        };
        
        if script_path.exists() {
            println!("Using {} script for {}", script_command, if cfg!(windows) { "Windows" } else { "Unix" });
            
            if cfg!(windows) {
                return Ok(vec!["cmd".to_string(), "/c".to_string(), script_command]);
            } else {
                // For Unix, ensure the script is executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = script_path.metadata() {
                        let permissions = metadata.permissions();
                        if permissions.mode() & 0o111 == 0 {
                            let mut new_permissions = permissions.clone();
                            new_permissions.set_mode(permissions.mode() | 0o755);
                            let _ = std::fs::set_permissions(&script_path, new_permissions);
                            println!("Made {} executable", script_command);
                        }
                    }
                }
                
                return Ok(vec!["bash".to_string(), script_command]);
            }
        }
        
        println!("No run script found, falling back to direct JAR execution");
        
        // Find neoforge server JAR
        let entries = fs::read_dir(server_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.contains("neoforge") && file_name.ends_with("server.jar") {
                let mut args = vec![
                    format!("-Xmx{}G", memory_gb),
                    format!("-Xms{}G", memory_gb / 2),
                    "-jar".to_string(),
                    file_name,
                    "nogui".to_string(),
                ];
                return Ok(args);
            }
        }
        Err(anyhow!("NeoForge server JAR not found"))
    }
}