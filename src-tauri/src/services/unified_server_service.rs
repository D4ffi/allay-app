use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Child, Stdio};
use anyhow::{Result, anyhow};
use crate::models::version::LoaderType;
use crate::services::mod_loader_strategy::{get_strategy, ModLoaderStrategy};
use crate::util::{JarCacheManager, ServerPropertiesManager, ServerProperties};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct UnifiedServerService {
    client: Client,
    jar_cache: JarCacheManager,
    running_servers: Arc<Mutex<HashMap<String, Child>>>,
}

impl UnifiedServerService {
    pub fn new() -> Result<Self> {
        let cache_dir = PathBuf::from("storage/version_cache");
        let jar_cache = JarCacheManager::new(cache_dir)?;
        
        Ok(Self {
            client: Client::new(),
            jar_cache,
            running_servers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Downloads or retrieves server JAR from cache using the strategy pattern
    pub async fn download_server_jar(
        &self,
        loader: LoaderType,
        minecraft_version: String,
        loader_version: Option<String>,
        server_path: PathBuf,
    ) -> Result<PathBuf> {
        // Get the appropriate strategy
        let strategy = get_strategy(&loader);
        
        // For loaders that require loader_version, validate it exists
        let loader_version_str = match loader {
            LoaderType::Vanilla | LoaderType::Paper => "".to_string(), // These don't need loader version
            _ => {
                loader_version.clone()
                    .ok_or_else(|| anyhow!("{:?} requires a loader version", loader))?
            }
        };

        // Delegate download to the strategy
        strategy.download_server_jar(
            &self.client,
            &self.jar_cache,
            &minecraft_version,
            &loader_version_str,
            &server_path,
            &loader
        ).await
    }

    /// Sets up server using the strategy pattern
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
        
        // Get the appropriate strategy
        let strategy = get_strategy(&loader);
        
        // For loaders that require loader_version, validate it exists
        let loader_version_str = match loader {
            LoaderType::Vanilla | LoaderType::Paper => "".to_string(),
            _ => {
                loader_version
                    .ok_or_else(|| anyhow!("{:?} requires a loader version", loader))?
                    .to_string()
            }
        };

        println!("Starting {:?} server setup...", loader);
        strategy.setup_server(&self.client, server_path, minecraft_version, &loader_version_str).await?;
        println!("{:?} server setup completed", loader);
        
        // Generate common server files
        println!("Generating EULA file...");
        self.generate_eula_file(server_path)?;
        println!("Generating server properties...");
        self.generate_server_properties(server_path, server_name)?;
        
        println!("=== Server setup completed for: {} ===", server_name);
        Ok(())
    }

    /// Starts a server using the strategy pattern
    pub async fn start_server(&self, server_name: &str, server_path: &PathBuf, loader: LoaderType, memory_mb: u32) -> Result<()> {
        let mut servers = self.running_servers.lock().await;
        
        if servers.contains_key(server_name) {
            return Err(anyhow!("Server {} is already running", server_name));
        }

        let strategy = get_strategy(&loader);
        
        // Convert MB to GB for JVM args, ensure minimum 1GB
        let memory_gb = std::cmp::max(1, memory_mb / 1024);
        let min_memory_gb = std::cmp::max(1, memory_gb / 2); // Half of max memory for initial heap
        
        let command_args = strategy.build_start_command(server_path, memory_gb, min_memory_gb)?;
        
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
        
        let child = Command::new(&command)
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

    /// Stops a running server
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

    /// Check if a server is running
    pub async fn is_server_running(&self, server_name: &str) -> bool {
        let servers = self.running_servers.lock().await;
        servers.contains_key(server_name)
    }

    /// Get list of all running servers
    pub async fn get_running_servers(&self) -> Vec<String> {
        let servers = self.running_servers.lock().await;
        servers.keys().cloned().collect()
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
}