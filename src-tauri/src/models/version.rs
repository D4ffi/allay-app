use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_type: VersionType,
    pub loader: LoaderType,
    pub release_time: DateTime<Utc>,
    pub latest: bool,
    pub recommended: bool,
    pub minecraft_version: Option<String>, // For loaders, this is the MC version they support
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionType {
    Release,
    Snapshot,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoaderType {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
    Paper,
    Quilt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCache {
    pub loader: LoaderType,
    pub versions: Vec<MinecraftVersion>,
    pub last_updated: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub latest: Option<MinecraftVersion>,
    pub recommended: Option<MinecraftVersion>,
    pub versions: Vec<MinecraftVersion>,
}

// API Response structures
#[derive(Debug, Deserialize)]
pub struct MojangVersionManifest {
    pub latest: MojangLatest,
    pub versions: Vec<MojangVersion>,
}

#[derive(Debug, Deserialize)]
pub struct MojangLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct MojangVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FabricVersions {
    pub game: Vec<FabricGameVersion>,
    pub mappings: Vec<FabricMapping>,
}

#[derive(Debug, Deserialize)]
pub struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricMapping {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct NeoForgeVersions {
    #[serde(rename = "isSnapshot")]
    pub is_snapshot: bool,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaperProject {
    pub project_id: String,
    pub project_name: String,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuiltVersions {
    pub game: Vec<QuiltGameVersion>,
    pub mappings: Vec<QuiltMapping>,
}

#[derive(Debug, Deserialize)]
pub struct QuiltGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct QuiltMapping {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub hashed: String,
}