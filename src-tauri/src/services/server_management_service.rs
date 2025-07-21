use std::path::PathBuf;
use std::fs;
use std::process::{Command, Child, Stdio};
use anyhow::{Result, anyhow};
use crate::models::version::LoaderType;
use crate::util::{ServerPropertiesManager, ServerProperties};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServerManagementService {
    running_servers: Arc<Mutex<HashMap<String, Child>>>,
}

impl ServerManagementService {
    pub fn new() -> Self {
        Self {
            running_servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn setup_server(
        &self,
        server_name: &str,
        loader: LoaderType,
        minecraft_version: &str,
        loader_version: Option<&str>,
        server_path: &PathBuf,
    ) -> Result<()> {
        println!("=== Setting up server: {} with loader: {:?} ===", server_name, loader);
        println!("Minecraft version: {}", minecraft_version);
        println!("Loader version: {:?}", loader_version);
        println!("Server path: {:?}", server_path);
        
        // Create a server directory if it doesn't exist
        fs::create_dir_all(server_path)?;
        println!("Server directory created/verified");
        
        match loader {
            LoaderType::Vanilla => {
                println!("Starting Vanilla server setup...");
                self.setup_vanilla_server(server_path, minecraft_version).await?;
                println!("Vanilla server setup completed");
            }
            LoaderType::Fabric => {
                println!("Starting Fabric server setup...");
                self.setup_fabric_server(server_path, minecraft_version, loader_version).await?;
                println!("Fabric server setup completed");
            }
            LoaderType::Forge => {
                println!("Starting Forge server setup...");
                self.setup_forge_server(server_path, minecraft_version, loader_version).await?;
                println!("Forge server setup completed");
            }
            LoaderType::NeoForge => {
                println!("Starting NeoForge server setup...");
                self.setup_neoforge_server(server_path, minecraft_version, loader_version).await?;
                println!("NeoForge server setup completed");
            }
            LoaderType::Paper => {
                println!("Starting Paper server setup...");
                self.setup_paper_server(server_path, minecraft_version).await?;
                println!("Paper server setup completed");
            }
            LoaderType::Quilt => {
                println!("Starting Quilt server setup...");
                self.setup_quilt_server(server_path, minecraft_version, loader_version).await?;
                println!("Quilt server setup completed");
            }
        }
        
        // Generate common server files
        println!("Generating EULA file...");
        self.generate_eula_file(server_path)?;
        println!("Generating server properties...");
        self.generate_server_properties(server_path, server_name)?;
        
        println!("=== Server setup completed for: {} ===", server_name);
        Ok(())
    }

    async fn setup_vanilla_server(&self, server_path: &PathBuf, minecraft_version: &str) -> Result<()> {
        let jar_name = format!("server-{}.jar", minecraft_version);
        let jar_path = server_path.join(&jar_name);
        
        if !jar_path.exists() {
            return Err(anyhow!("Server JAR not found: {:?}", jar_path));
        }

        // Check if server is already initialized (eula.txt and server.properties will be created later)
        // We check for world folder or logs folder as indicators of initialization
        let world_folder = server_path.join("world");
        let logs_folder = server_path.join("logs");
        
        if world_folder.exists() || logs_folder.exists() {
            println!("Vanilla server already initialized: {:?}", jar_path);
            return Ok(());
        }

        println!("Initializing Vanilla server...");
        println!("JAR path: {:?}", jar_path);
        println!("Working directory: {:?}", server_path);
        
        // Run the server JAR once to generate initial files
        // This will create the server structure and then stop due to EULA agreement
        let output = Command::new("java")
            .args(&[
                "-Xmx1G",
                "-Xms512M", 
                "-jar", 
                &jar_name,
                "nogui"
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
        
        println!("Vanilla server initialization exit status: {:?}", output.status);
        println!("Vanilla server initialization stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Vanilla server initialization stderr: {}", String::from_utf8_lossy(&output.stderr));

        // The server will exit with non-zero status due to EULA, this is expected
        // We don't check output.status.success() because the server exits due to EULA not being accepted
        
        println!("Vanilla server initialized successfully");
        Ok(())
    }

    async fn setup_fabric_server(&self, server_path: &PathBuf, minecraft_version: &str, loader_version: Option<&str>) -> Result<()> {
        let loader_ver = loader_version.ok_or_else(|| anyhow!("Fabric loader version required"))?;
        
        // Extract clean loader version
        let clean_version = if loader_ver.starts_with("fabric-") {
            let without_prefix = loader_ver.strip_prefix("fabric-").unwrap_or(loader_ver);
            if let Some(dash_pos) = without_prefix.find('-') {
                &without_prefix[..dash_pos]
            } else {
                without_prefix
            }
        } else {
            loader_ver
        };
        
        // Fabric uses a pre-built server launcher with specific filename format
        let server_jar_name = format!("fabric-server-mc.{}-loader.{}-launcher.1.0.3.jar", minecraft_version, clean_version);
        let server_jar = server_path.join(&server_jar_name);
        
        if !server_jar.exists() {
            return Err(anyhow!("Fabric server launcher not found: {:?}", server_jar));
        }

        println!("Fabric server launcher ready: {:?}", server_jar);
        Ok(())
    }

    async fn setup_forge_server(&self, server_path: &PathBuf, minecraft_version: &str, loader_version: Option<&str>) -> Result<()> {
        let loader_ver = loader_version.ok_or_else(|| anyhow!("Forge loader version required"))?;
        
        // Handle the case where loader_ver already has "forge-" prefix
        let installer_name = if loader_ver.starts_with("forge-") {
            format!("{}-installer.jar", loader_ver)
        } else {
            format!("forge-{}-installer.jar", loader_ver)
        };
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("Forge installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let run_script = server_path.join("run.sh");
        let server_jar = if loader_ver.starts_with("forge-") {
            server_path.join(format!("{}-server.jar", loader_ver))
        } else {
            server_path.join(format!("forge-{}-server.jar", loader_ver))
        };
        
        if run_script.exists() || server_jar.exists() {
            println!("Forge server already installed");
            return Ok(());
        }

        println!("Installing Forge server...");
        println!("Installer path: {:?}", installer_path);
        println!("Working directory: {:?}", server_path);
        
        // Run Forge installer with more verbose output
        // Use just the filename since we're setting the working directory
        let installer_filename = installer_path.file_name()
            .ok_or_else(|| anyhow!("Invalid installer filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid installer filename encoding"))?;
        
        println!("Using installer filename: {}", installer_filename);
        
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
        
        println!("Forge installer exit status: {:?}", output.status);
        println!("Forge installer stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Forge installer stderr: {}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Forge installation failed: {}", error));
        }

        println!("Forge server installed successfully");
        Ok(())
    }

    async fn setup_neoforge_server(&self, server_path: &PathBuf, minecraft_version: &str, loader_version: Option<&str>) -> Result<()> {
        let loader_ver = loader_version.ok_or_else(|| anyhow!("NeoForge loader version required"))?;
        
        // Handle the case where loader_ver already has "neoforge-" prefix
        let installer_name = if loader_ver.starts_with("neoforge-") {
            format!("{}-installer.jar", loader_ver)
        } else {
            format!("neoforge-{}-installer.jar", loader_ver)
        };
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("NeoForge installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let run_script = server_path.join("run.sh");
        let server_jar = if loader_ver.starts_with("neoforge-") {
            server_path.join(format!("{}-server.jar", loader_ver))
        } else {
            server_path.join(format!("neoforge-{}-server.jar", loader_ver))
        };
        
        if run_script.exists() || server_jar.exists() {
            println!("NeoForge server already installed");
            return Ok(());
        }

        println!("Installing NeoForge server...");
        
        // Run NeoForge installer
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

    async fn setup_paper_server(&self, server_path: &PathBuf, minecraft_version: &str) -> Result<()> {
        let jar_name = format!("paper-{}.jar", minecraft_version);
        let jar_path = server_path.join(&jar_name);
        
        if !jar_path.exists() {
            return Err(anyhow!("Paper JAR not found: {:?}", jar_path));
        }

        // For Paper, the JAR is ready to use directly
        println!("Paper server ready: {:?}", jar_path);
        Ok(())
    }

    async fn setup_quilt_server(&self, server_path: &PathBuf, minecraft_version: &str, loader_version: Option<&str>) -> Result<()> {
        let _loader_ver = loader_version.ok_or_else(|| anyhow!("Quilt loader version required"))?;
        
        // Check if Quilt profile JSON exists
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
                    self.download_library(name, url, &libraries_dir).await?;
                }
            }
        }

        // Download vanilla server JAR if needed
        let vanilla_jar = server_path.join("server.jar");
        if !vanilla_jar.exists() {
            println!("Downloading vanilla Minecraft server for Quilt...");
            // Use the vanilla download service to get the server JAR
            let vanilla_url = self.get_vanilla_server_url(minecraft_version).await?;
            let response = reqwest::get(&vanilla_url).await?;
            let bytes = response.bytes().await?;
            fs::write(&vanilla_jar, &bytes)?;
            println!("Vanilla server JAR downloaded: {:?}", vanilla_jar);
        }

        println!("Quilt server setup completed");
        Ok(())
    }

    async fn download_library(&self, name: &str, base_url: &str, libraries_dir: &PathBuf) -> Result<()> {
        // Convert Maven coordinate to file path
        // Example: "org.quiltmc:quilt-loader:0.19.0" -> "org/quiltmc/quilt-loader/0.19.0/quilt-loader-0.19.0.jar"
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
        
        let response = reqwest::get(&download_url).await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download library {}: HTTP {}", name, response.status()));
        }
        
        let bytes = response.bytes().await?;
        fs::write(&jar_path, &bytes)?;
        
        println!("Downloaded library: {:?}", jar_path);
        Ok(())
    }

    async fn get_vanilla_server_url(&self, minecraft_version: &str) -> Result<String> {
        // Get version manifest
        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest: serde_json::Value = reqwest::get(manifest_url).await?.json().await?;
        
        // Find the specific version
        let versions = manifest["versions"].as_array()
            .ok_or_else(|| anyhow!("Invalid version manifest"))?;
        
        let version_info = versions.iter()
            .find(|v| v["id"].as_str() == Some(minecraft_version))
            .ok_or_else(|| anyhow!("Minecraft version {} not found", minecraft_version))?;
        
        let version_url = version_info["url"].as_str()
            .ok_or_else(|| anyhow!("Version URL not found"))?;
        
        // Get version details
        let version_details: serde_json::Value = reqwest::get(version_url).await?.json().await?;
        
        // Get server JAR URL
        let server_url = version_details["downloads"]["server"]["url"].as_str()
            .ok_or_else(|| anyhow!("Server JAR URL not found for version {}", minecraft_version))?;
        
        Ok(server_url.to_string())
    }

    fn build_quilt_start_command(&self, server_path: &PathBuf, memory_gb: u32, min_memory_gb: u32) -> Result<Vec<String>> {
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
        let mut args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-cp".to_string(),
            classpath_str,
            main_class.to_string(),
            "nogui".to_string(),
        ];

        Ok(args)
    }

    fn get_library_jar_path(&self, name: &str) -> Result<String> {
        // Convert Maven coordinate to relative JAR path
        // Example: "org.quiltmc:quilt-loader:0.19.0" -> "org/quiltmc/quilt-loader/0.19.0/quilt-loader-0.19.0.jar"
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

    fn generate_eula_file(&self, server_path: &PathBuf) -> Result<()> {
        let eula_path = server_path.join("eula.txt");
        let eula_content = "# EULA accepted automatically by Allay\neula=true\n";
        fs::write(eula_path, eula_content)?;
        println!("Generated eula.txt");
        Ok(())
    }

    fn generate_server_properties(&self, server_path: &PathBuf, server_name: &str) -> Result<()> {
        let properties_path = server_path.join("server.properties");
        
        // Don't overwrite existing properties
        if properties_path.exists() {
            println!("server.properties already exists, skipping generation");
            return Ok(());
        }

        let properties_manager = ServerPropertiesManager::new(properties_path);
        
        // Create default properties and customize for Allay
        let mut properties = ServerProperties::default();
        properties.motd = format!("{} - Managed by Allay", server_name);
        properties.level_name = "world".to_string();
        properties.gamemode = "survival".to_string();
        properties.difficulty = "easy".to_string();
        properties.max_players = 20;
        properties.online_mode = true;
        properties.pvp = true;
        properties.spawn_protection = 16;
        properties.enable_command_block = false;
        properties.white_list = false;
        properties.enable_rcon = false;
        properties.server_port = 25565;
        
        properties_manager.save_properties(&properties).map_err(|e| anyhow!("Failed to save server.properties: {}", e))?;
        println!("Generated server.properties using ServerPropertiesManager");
        Ok(())
    }

    pub async fn start_server(&self, server_name: &str, server_path: &PathBuf, loader: LoaderType, memory_mb: u32) -> Result<()> {
        let mut servers = self.running_servers.lock().await;
        
        if servers.contains_key(server_name) {
            return Err(anyhow!("Server {} is already running", server_name));
        }

        let command_args = self.build_start_command(server_path, loader, memory_mb)?;
        
        println!("Starting server: {} with command: {:?}", server_name, command_args);
        
        // Determine the command and arguments based on the first element
        let (command, args) = if command_args.len() > 0 {
            let first_arg = &command_args[0];
            
            if first_arg == "cmd" || first_arg == "bash" || first_arg == "./run.sh" || first_arg.ends_with(".sh") || first_arg.ends_with(".bat") {
                // This is a script command, use the first argument as the command
                if first_arg == "cmd" || first_arg == "bash" {
                    // Windows: cmd /c run.bat or Unix: bash ./run.sh
                    (first_arg.clone(), command_args[1..].to_vec())
                } else {
                    // Direct script execution: ./run.sh
                    (first_arg.clone(), command_args[1..].to_vec())
                }
            } else {
                // This is a Java command
                ("java".to_string(), command_args)
            }
        } else {
            return Err(anyhow!("No command arguments provided"));
        };
        
        let mut child = Command::new(&command)
            .args(&args)
            .current_dir(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    match command.as_str() {
                        "java" => anyhow!("Java is not installed or not found in PATH. Please install Java to run Minecraft servers."),
                        "bash" => anyhow!("Bash is not available or not found in PATH. Please install bash or use a different shell."),
                        "cmd" => anyhow!("Command Prompt (cmd) is not available. This should not happen on Windows."),
                        _ => anyhow!("{} is not available or not found in PATH. Error: {}", command, e)
                    }
                } else {
                    anyhow!("Failed to execute {}: {}", command, e)
                }
            })?;

        servers.insert(server_name.to_string(), child);
        println!("Server {} started successfully", server_name);
        
        Ok(())
    }

    fn build_start_command(&self, server_path: &PathBuf, loader: LoaderType, memory_mb: u32) -> Result<Vec<String>> {
        // Convert MB to GB for JVM args, ensure minimum 1GB
        let memory_gb = std::cmp::max(1, memory_mb / 1024);
        let min_memory_gb = std::cmp::max(1, memory_gb / 2); // Half of max memory for initial heap
        
        let mut args = vec![
            format!("-Xmx{}G", memory_gb),
            format!("-Xms{}G", min_memory_gb),
            "-jar".to_string(),
        ];

        let jar_name = match loader {
            LoaderType::Vanilla => {
                // Find the vanilla server JAR
                let entries = fs::read_dir(server_path)?;
                for entry in entries {
                    let entry = entry?;
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("server-") && file_name.ends_with(".jar") {
                        return Ok([args, vec![file_name, "nogui".to_string()]].concat());
                    }
                }
                return Err(anyhow!("Vanilla server JAR not found"));
            }
            LoaderType::Fabric => {
                // Find the fabric server launcher with the specific naming format
                let entries = fs::read_dir(server_path)?;
                for entry in entries {
                    let entry = entry?;
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("fabric-server-mc.") && file_name.contains("-loader.") && file_name.contains("-launcher.") && file_name.ends_with(".jar") {
                        return Ok([args, vec![file_name, "nogui".to_string()]].concat());
                    }
                }
                return Err(anyhow!("Fabric server launcher JAR not found"));
            }
            LoaderType::Forge | LoaderType::NeoForge => {
                // Check OS and use appropriate script
                let (script_path, script_command) = if cfg!(windows) {
                    (server_path.join("run.bat"), "run.bat".to_string())
                } else {
                    (server_path.join("run.sh"), "./run.sh".to_string())
                };
                
                if script_path.exists() {
                    // Use the OS-appropriate run script
                    println!("Using {} script for {}", script_command, if cfg!(windows) { "Windows" } else { "Unix" });
                    
                    if cfg!(windows) {
                        // For Windows batch files, we need to use cmd.exe
                        return Ok(vec!["cmd".to_string(), "/c".to_string(), script_command]);
                    } else {
                        // For Unix, ensure the script is executable
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(metadata) = script_path.metadata() {
                                let permissions = metadata.permissions();
                                if permissions.mode() & 0o111 == 0 {
                                    // Script is not executable, try to make it executable
                                    let mut new_permissions = permissions.clone();
                                    new_permissions.set_mode(permissions.mode() | 0o755);
                                    let _ = std::fs::set_permissions(&script_path, new_permissions);
                                    println!("Made {} executable", script_command);
                                }
                            }
                        }
                        
                        // Use bash to execute the script for better compatibility
                        return Ok(vec!["bash".to_string(), script_command]);
                    }
                }
                
                println!("No run script found, falling back to direct JAR execution");
                
                // Find forge/neoforge server JAR
                let entries = fs::read_dir(server_path)?;
                for entry in entries {
                    let entry = entry?;
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if (file_name.contains("forge") || file_name.contains("neoforge")) 
                        && file_name.ends_with("server.jar") {
                        return Ok([args, vec![file_name, "nogui".to_string()]].concat());
                    }
                }
                return Err(anyhow!("Forge/NeoForge server JAR not found"));
            }
            LoaderType::Paper => {
                // Find Paper JAR
                let entries = fs::read_dir(server_path)?;
                for entry in entries {
                    let entry = entry?;
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("paper-") && file_name.ends_with(".jar") {
                        return Ok([args, vec![file_name, "nogui".to_string()]].concat());
                    }
                }
                return Err(anyhow!("Paper server JAR not found"));
            }
            LoaderType::Quilt => {
                // Quilt needs to build a custom classpath and use mainClass
                return self.build_quilt_start_command(server_path, memory_gb, min_memory_gb);
            }
        };

        args.push(jar_name);
        args.push("nogui".to_string());
        
        Ok(args)
    }

    pub async fn stop_server(&self, server_name: &str) -> Result<()> {
        let mut servers = self.running_servers.lock().await;
        
        if let Some(mut child) = servers.remove(server_name) {
            // Send stop command to server
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                writeln!(stdin, "stop")?;
            }
            
            // Wait for graceful shutdown
            match child.wait() {
                Ok(_) => println!("Server {} stopped gracefully", server_name),
                Err(_) => {
                    println!("Force killing server {}", server_name);
                    child.kill()?;
                }
            }
            
            Ok(())
        } else {
            Err(anyhow!("Server {} is not running", server_name))
        }
    }

    pub async fn is_server_running(&self, server_name: &str) -> bool {
        let servers = self.running_servers.lock().await;
        servers.contains_key(server_name)
    }

    pub async fn get_running_servers(&self) -> Vec<String> {
        let servers = self.running_servers.lock().await;
        servers.keys().cloned().collect()
    }
}