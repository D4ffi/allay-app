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
        
        // Create server directory if it doesn't exist
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
        let installer_name = format!("fabric-server-{}-{}.jar", minecraft_version, loader_ver);
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("Fabric installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let server_jar = server_path.join("fabric-server-launch.jar");
        if server_jar.exists() {
            println!("Fabric server already installed: {:?}", server_jar);
            return Ok(());
        }

        println!("Installing Fabric server...");
        
        // Run Fabric installer
        let installer_filename = installer_path.file_name()
            .ok_or_else(|| anyhow!("Invalid installer filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid installer filename encoding"))?;

        let output = Command::new("java")
            .args(&[
                "-jar", 
                installer_filename,
                "server",
                "-mcversion", minecraft_version,
                "-loader", loader_ver,
                "-downloadMinecraft"
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
            return Err(anyhow!("Fabric installation failed: {}", error));
        }

        println!("Fabric server installed successfully");
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
        let loader_ver = loader_version.ok_or_else(|| anyhow!("Quilt loader version required"))?;
        let installer_name = format!("quilt-server-{}-{}.jar", minecraft_version, loader_ver);
        let installer_path = server_path.join(&installer_name);
        
        if !installer_path.exists() {
            return Err(anyhow!("Quilt installer not found: {:?}", installer_path));
        }

        // Check if server is already installed
        let server_jar = server_path.join("quilt-server-launch.jar");
        if server_jar.exists() {
            println!("Quilt server already installed: {:?}", server_jar);
            return Ok(());
        }

        println!("Installing Quilt server...");
        
        // Run Quilt installer (similar to Fabric)
        let installer_filename = installer_path.file_name()
            .ok_or_else(|| anyhow!("Invalid installer filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid installer filename encoding"))?;

        let output = Command::new("java")
            .args(&[
                "-jar", 
                installer_filename,
                "install",
                "server",
                minecraft_version,
                "--install-dir", ".",
                "--loader-version", loader_ver
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
            return Err(anyhow!("Quilt installation failed: {}", error));
        }

        println!("Quilt server installed successfully");
        Ok(())
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

    pub async fn start_server(&self, server_name: &str, server_path: &PathBuf, loader: LoaderType) -> Result<()> {
        let mut servers = self.running_servers.lock().await;
        
        if servers.contains_key(server_name) {
            return Err(anyhow!("Server {} is already running", server_name));
        }

        let java_command = self.build_start_command(server_path, loader)?;
        
        println!("Starting server: {} with command: {:?}", server_name, java_command);
        
        let mut child = Command::new("java")
            .args(&java_command)
            .current_dir(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!("Java is not installed or not found in PATH. Please install Java to run Minecraft servers.")
                } else {
                    anyhow!("Failed to execute Java: {}", e)
                }
            })?;

        servers.insert(server_name.to_string(), child);
        println!("Server {} started successfully", server_name);
        
        Ok(())
    }

    fn build_start_command(&self, server_path: &PathBuf, loader: LoaderType) -> Result<Vec<String>> {
        let mut args = vec![
            "-Xmx2G".to_string(),
            "-Xms1G".to_string(),
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
            LoaderType::Fabric => "fabric-server-launch.jar".to_string(),
            LoaderType::Forge | LoaderType::NeoForge => {
                // Look for run.sh script first, otherwise find the server JAR
                let run_script = server_path.join("run.sh");
                if run_script.exists() {
                    // Use the run script instead of direct java command
                    return Ok(vec!["./run.sh".to_string()]);
                }
                
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
            LoaderType::Quilt => "quilt-server-launch.jar".to_string(),
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