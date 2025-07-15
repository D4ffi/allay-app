
#[derive(Debug, Clone)]
pub enum ServerState {
    Offline,
    Online,
    Connecting,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone)]
pub enum ServerType {
    Vanilla,
    Fabric,
    Forge,
    Neoforge,
    Paper,
    Quilt
}