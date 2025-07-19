use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    pub server_port: u16,
    pub gamemode: String,
    pub difficulty: String,
    pub level_name: String,
    pub max_players: u32,
    pub motd: String,
    pub online_mode: bool,
    pub pvp: bool,
    pub level_seed: String,
    pub spawn_protection: u32,
    pub white_list: bool,
    pub enable_command_block: bool,
    pub spawn_monsters: bool,
    pub spawn_animals: bool,
    pub spawn_npcs: bool,
    pub allow_flight: bool,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub op_permission_level: u32,
    pub allow_nether: bool,
    pub enable_rcon: bool,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub query_port: u16,
    pub enable_query: bool,
    pub generator_settings: String,
    pub level_type: String,
    pub hardcore: bool,
    pub enable_status: bool,
    pub enable_jmx_monitoring: bool,
    pub broadcast_rcon_to_ops: bool,
    pub broadcast_console_to_ops: bool,
    pub enforce_whitelist: bool,
    pub resource_pack: String,
    pub resource_pack_prompt: String,
    pub resource_pack_sha1: String,
    pub require_resource_pack: bool,
    pub max_world_size: u32,
    pub function_permission_level: u32,
    pub max_tick_time: u64,
    pub rate_limit: u32,
    pub network_compression_threshold: i32,
    pub use_native_transport: bool,
    pub enable_jmx_monitoring_port: u16,
    pub enable_jmx_monitoring_rmi_port: u16,
    pub sync_chunk_writes: bool,
    pub server_ip: String,
    pub prevent_proxy_connections: bool,
    pub hide_online_players: bool,
    pub entity_broadcast_range_percentage: u32,
    pub player_idle_timeout: u32,
    pub force_gamemode: bool,
    pub debug: bool,
    pub max_chained_neighbor_updates: u32,
    pub text_filtering_config: String,
    pub initial_disabled_packs: String,
    pub initial_enabled_packs: String,
    pub log_ips: bool,
    pub pause_when_empty_seconds: u32,
    pub accepts_transfers: bool,
    pub generate_structures: bool,
    pub snooper_enabled: bool,
}

impl Default for ServerProperties {
    fn default() -> Self {
        Self {
            server_port: 25565,
            gamemode: "survival".to_string(),
            difficulty: "easy".to_string(),
            level_name: "world".to_string(),
            max_players: 20,
            motd: "THE Minecraft Server".to_string(),
            online_mode: true,
            pvp: true,
            level_seed: String::new(),
            spawn_protection: 4,
            white_list: false,
            enable_command_block: false,
            spawn_monsters: true,
            spawn_animals: true,
            spawn_npcs: true,
            allow_flight: true,
            view_distance: 10,
            simulation_distance: 10,
            op_permission_level: 4,
            allow_nether: true,
            enable_rcon: false,
            rcon_port: 25575,
            rcon_password: String::new(),
            query_port: 25565,
            enable_query: true,
            generator_settings: String::new(),
            level_type: "default".to_string(),
            hardcore: false,
            enable_status: true,
            enable_jmx_monitoring: false,
            broadcast_rcon_to_ops: true,
            broadcast_console_to_ops: false,
            enforce_whitelist: false,
            resource_pack: String::new(),
            resource_pack_prompt: String::new(),
            resource_pack_sha1: String::new(),
            require_resource_pack: false,
            max_world_size: 29999984,
            function_permission_level: 2,
            max_tick_time: 60000,
            rate_limit: 0,
            network_compression_threshold: 256,
            use_native_transport: true,
            enable_jmx_monitoring_port: 9999,
            enable_jmx_monitoring_rmi_port: 9998,
            sync_chunk_writes: true,
            server_ip: String::new(),
            prevent_proxy_connections: false,
            hide_online_players: false,
            entity_broadcast_range_percentage: 100,
            player_idle_timeout: 0,
            force_gamemode: false,
            debug: false,
            max_chained_neighbor_updates: 1000000,
            text_filtering_config: String::new(),
            initial_disabled_packs: String::new(),
            initial_enabled_packs: "vanilla".to_string(),
            log_ips: true,
            pause_when_empty_seconds: 120,
            accepts_transfers: false,
            generate_structures: true,
            snooper_enabled: true,
        }
    }
}

impl ServerProperties {
    pub fn to_properties_string(&self) -> String {
        format!(
            r#"#        MinecraftServerProperties
#        Allay necessary configs
enable-query={}
query.port={}
broadcast-console-to-ops={}
#        Minecraft World Settings
spawn-protection={}
allow-nether={}
allow-flight={}
difficulty={}
gamemode={}
force-gamemode={}
spawn-monsters={}
pvp={}
level-type={}
hardcore={}
enable-command-block={}
simulation-distance={}
level-name={}
level-seed={}
generate-structures={}
#        Server Settings
snooper-enabled={}
sync-chunk-writes={}
op-permission-level={}
max-tick-time={}
player-idle-timeout={}
generator-settings={}
enforce-whitelist={}
server-ip={}
server-port={}
motd={}
max-players={}
max-tick-time={}
entity-broadcast-range-percentage={}
# Only available in 1.21
pause-when-empty-seconds={}"#,
            self.enable_query,
            self.query_port,
            self.broadcast_console_to_ops,
            self.spawn_protection,
            self.allow_nether,
            self.allow_flight,
            self.difficulty,
            self.gamemode,
            self.force_gamemode,
            self.spawn_monsters,
            self.pvp,
            self.level_type,
            self.hardcore,
            self.enable_command_block,
            self.simulation_distance,
            self.level_name,
            self.level_seed,
            self.generate_structures,
            self.snooper_enabled,
            self.sync_chunk_writes,
            self.op_permission_level,
            self.max_tick_time,
            self.player_idle_timeout,
            self.generator_settings,
            self.enforce_whitelist,
            self.server_ip,
            self.server_port,
            self.motd,
            self.max_players,
            self.max_tick_time,
            self.entity_broadcast_range_percentage,
            self.pause_when_empty_seconds
        )
    }
    
    pub fn from_properties_string(content: &str) -> Result<Self, Error> {
        let mut properties = ServerProperties::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "server-port" => properties.server_port = value.parse().unwrap_or(25565),
                    "gamemode" => properties.gamemode = value.to_string(),
                    "difficulty" => properties.difficulty = value.to_string(),
                    "level-name" => properties.level_name = value.to_string(),
                    "max-players" => properties.max_players = value.parse().unwrap_or(20),
                    "motd" => properties.motd = value.to_string(),
                    "online-mode" => properties.online_mode = value.parse().unwrap_or(true),
                    "pvp" => properties.pvp = value.parse().unwrap_or(true),
                    "level-seed" => properties.level_seed = value.to_string(),
                    "spawn-protection" => properties.spawn_protection = value.parse().unwrap_or(16),
                    "white-list" => properties.white_list = value.parse().unwrap_or(false),
                    "enable-command-block" => properties.enable_command_block = value.parse().unwrap_or(false),
                    "spawn-monsters" => properties.spawn_monsters = value.parse().unwrap_or(true),
                    "spawn-animals" => properties.spawn_animals = value.parse().unwrap_or(true),
                    "spawn-npcs" => properties.spawn_npcs = value.parse().unwrap_or(true),
                    "allow-flight" => properties.allow_flight = value.parse().unwrap_or(false),
                    "view-distance" => properties.view_distance = value.parse().unwrap_or(10),
                    "simulation-distance" => properties.simulation_distance = value.parse().unwrap_or(10),
                    "op-permission-level" => properties.op_permission_level = value.parse().unwrap_or(4),
                    "allow-nether" => properties.allow_nether = value.parse().unwrap_or(true),
                    "enable-rcon" => properties.enable_rcon = value.parse().unwrap_or(false),
                    "rcon.port" => properties.rcon_port = value.parse().unwrap_or(25575),
                    "rcon.password" => properties.rcon_password = value.to_string(),
                    "query.port" => properties.query_port = value.parse().unwrap_or(25565),
                    "enable-query" => properties.enable_query = value.parse().unwrap_or(false),
                    "generator-settings" => properties.generator_settings = value.to_string(),
                    "level-type" => properties.level_type = value.to_string(),
                    "hardcore" => properties.hardcore = value.parse().unwrap_or(false),
                    "enable-status" => properties.enable_status = value.parse().unwrap_or(true),
                    "enable-jmx-monitoring" => properties.enable_jmx_monitoring = value.parse().unwrap_or(false),
                    "broadcast-rcon-to-ops" => properties.broadcast_rcon_to_ops = value.parse().unwrap_or(true),
                    "broadcast-console-to-ops" => properties.broadcast_console_to_ops = value.parse().unwrap_or(true),
                    "enforce-whitelist" => properties.enforce_whitelist = value.parse().unwrap_or(false),
                    "resource-pack" => properties.resource_pack = value.to_string(),
                    "resource-pack-prompt" => properties.resource_pack_prompt = value.to_string(),
                    "resource-pack-sha1" => properties.resource_pack_sha1 = value.to_string(),
                    "require-resource-pack" => properties.require_resource_pack = value.parse().unwrap_or(false),
                    "max-world-size" => properties.max_world_size = value.parse().unwrap_or(29999984),
                    "function-permission-level" => properties.function_permission_level = value.parse().unwrap_or(2),
                    "max-tick-time" => properties.max_tick_time = value.parse().unwrap_or(60000),
                    "rate-limit" => properties.rate_limit = value.parse().unwrap_or(0),
                    "network-compression-threshold" => properties.network_compression_threshold = value.parse().unwrap_or(256),
                    "use-native-transport" => properties.use_native_transport = value.parse().unwrap_or(true),
                    "enable-jmx-monitoring.port" => properties.enable_jmx_monitoring_port = value.parse().unwrap_or(9999),
                    "enable-jmx-monitoring.rmi.port" => properties.enable_jmx_monitoring_rmi_port = value.parse().unwrap_or(9998),
                    "sync-chunk-writes" => properties.sync_chunk_writes = value.parse().unwrap_or(true),
                    "server-ip" => properties.server_ip = value.to_string(),
                    "prevent-proxy-connections" => properties.prevent_proxy_connections = value.parse().unwrap_or(false),
                    "hide-online-players" => properties.hide_online_players = value.parse().unwrap_or(false),
                    "entity-broadcast-range-percentage" => properties.entity_broadcast_range_percentage = value.parse().unwrap_or(100),
                    "player-idle-timeout" => properties.player_idle_timeout = value.parse().unwrap_or(0),
                    "force-gamemode" => properties.force_gamemode = value.parse().unwrap_or(false),
                    "debug" => properties.debug = value.parse().unwrap_or(false),
                    "max-chained-neighbor-updates" => properties.max_chained_neighbor_updates = value.parse().unwrap_or(1000000),
                    "text-filtering-config" => properties.text_filtering_config = value.to_string(),
                    "initial-disabled-packs" => properties.initial_disabled_packs = value.to_string(),
                    "initial-enabled-packs" => properties.initial_enabled_packs = value.to_string(),
                    "log-ips" => properties.log_ips = value.parse().unwrap_or(true),
                    "pause-when-empty-seconds" => properties.pause_when_empty_seconds = value.parse().unwrap_or(60),
                    "accepts-transfers" => properties.accepts_transfers = value.parse().unwrap_or(false),
                    "generate-structures" => properties.generate_structures = value.parse().unwrap_or(true),
                    "snooper-enabled" => properties.snooper_enabled = value.parse().unwrap_or(true),
                    _ => {}
                }
            }
        }
        
        Ok(properties)
    }
}

pub struct ServerPropertiesManager {
    properties_path: PathBuf,
}

impl ServerPropertiesManager {
    pub fn new(properties_path: PathBuf) -> Self {
        Self { properties_path }
    }
    
    pub fn load_properties(&self) -> Result<ServerProperties, Error> {
        if !self.properties_path.exists() {
            return Ok(ServerProperties::default());
        }
        
        let content = fs::read_to_string(&self.properties_path)?;
        ServerProperties::from_properties_string(&content)
    }
    
    pub fn save_properties(&self, properties: &ServerProperties) -> Result<(), Error> {
        if let Some(parent) = self.properties_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = properties.to_properties_string();
        fs::write(&self.properties_path, content)?;
        Ok(())
    }
    
    pub fn update_property(&self, key: &str, value: &str) -> Result<(), Error> {
        let mut properties = self.load_properties()?;
        
        match key {
            "server-port" => properties.server_port = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid port number"))?,
            "gamemode" => properties.gamemode = value.to_string(),
            "difficulty" => properties.difficulty = value.to_string(),
            "level-name" => properties.level_name = value.to_string(),
            "max-players" => properties.max_players = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid max players number"))?,
            "motd" => properties.motd = value.to_string(),
            "online-mode" => properties.online_mode = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid boolean value"))?,
            "pvp" => properties.pvp = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid boolean value"))?,
            "level-seed" => properties.level_seed = value.to_string(),
            "spawn-protection" => properties.spawn_protection = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid spawn protection number"))?,
            "white-list" => properties.white_list = value.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid boolean value"))?,
            _ => return Err(Error::new(ErrorKind::InvalidInput, format!("Unknown property: {}", key))),
        }
        
        self.save_properties(&properties)?;
        Ok(())
    }
    
    pub fn get_property(&self, key: &str) -> Result<String, Error> {
        let properties = self.load_properties()?;
        
        let value = match key {
            "server-port" => properties.server_port.to_string(),
            "gamemode" => properties.gamemode,
            "difficulty" => properties.difficulty,
            "level-name" => properties.level_name,
            "max-players" => properties.max_players.to_string(),
            "motd" => properties.motd,
            "online-mode" => properties.online_mode.to_string(),
            "pvp" => properties.pvp.to_string(),
            "level-seed" => properties.level_seed,
            "spawn-protection" => properties.spawn_protection.to_string(),
            "white-list" => properties.white_list.to_string(),
            _ => return Err(Error::new(ErrorKind::InvalidInput, format!("Unknown property: {}", key))),
        };
        
        Ok(value)
    }
    
    pub fn create_default_properties(&self) -> Result<(), Error> {
        let properties = ServerProperties::default();
        self.save_properties(&properties)?;
        Ok(())
    }
}