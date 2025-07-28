use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use chrono::Utc;
use crate::services::mod_loader_strategy::ModLoaderStrategy;
use crate::models::version::{LoaderType, VersionResponse, MinecraftVersion, VersionType};
use crate::util::JarCacheManager;

/// Forge strategy
pub struct ForgeStrategy;

#[async_trait]
impl ModLoaderStrategy for ForgeStrategy {
    async fn get_versions(&self, client: &Client, minecraft_version: Option<String>) -> Result<VersionResponse> {
        let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
        let response = client.get(url).send().await?.text().await?;

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
                        let minecraft_version_obj = MinecraftVersion {
                            id: format!("forge-{}", version_str),
                            version_type: VersionType::Release,
                            loader: LoaderType::Forge,
                            release_time: Utc::now(),
                            latest: i == 0,
                            recommended: i == 0,
                            minecraft_version: Some(mc_version.to_string()),
                        };
                        versions.push(minecraft_version_obj);
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
                        
                        let minecraft_version_obj = MinecraftVersion {
                            id: format!("forge-{}", version_str),
                            version_type: VersionType::Release,
                            loader: LoaderType::Forge,
                            release_time: Utc::now(),
                            latest: overall_index == 0,
                            recommended: overall_index == 0,
                            minecraft_version: Some(mc_version.to_string()),
                        };
                        versions.push(minecraft_version_obj);
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
    
    async fn get_download_url(&self, _client: &Client, minecraft_version: &str, loader_version: &str) -> Result<String> {
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
    
    fn get_filename(&self, _minecraft_version: &str, loader_version: &str) -> String {
        if loader_version.starts_with("forge-") {
            format!("{}-installer.jar", loader_version)
        } else {
            format!("forge-{}-installer.jar", loader_version)
        }
    }
    
    async fn setup_server(&self, _client: &Client, server_path: &PathBuf, _minecraft_version: &str, loader_version: &str) -> Result<()> {
        let installer_name = if loader_version.starts_with("forge-") {
            format!("{}-installer.jar", loader_version)
        } else {
            format!("forge-{}-installer.jar", loader_version)
        };
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("Forge installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let run_script = server_path.join("run.sh");
        let server_jar = if loader_version.starts_with("forge-") {
            server_path.join(format!("{}-server.jar", loader_version))
        } else {
            server_path.join(format!("forge-{}-server.jar", loader_version))
        };
        
        if run_script.exists() || server_jar.exists() {
            println!("Forge server already installed");
            return Ok(());
        }

        println!("Installing Forge server...");
        
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
            return Err(anyhow!("Forge installation failed: {}", error));
        }

        println!("Forge server installed successfully");
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
        
        // Find forge server JAR
        let entries = fs::read_dir(server_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.contains("forge") && file_name.ends_with("server.jar") {
                let args = vec![
                    format!("-Xmx{}G", memory_gb),
                    format!("-Xms{}G", memory_gb / 2),
                    "-jar".to_string(),
                    file_name,
                    "nogui".to_string(),
                ];
                return Ok(args);
            }
        }
        Err(anyhow!("Forge server JAR not found"))
    }
}