use super::server_state::{ServerState, ServerType};

#[derive(Debug)]
pub struct Server {
    name: String,
    description: String,
    has_custom_img: bool,
    version: String,
    server_type: ServerType,
    server_state: ServerState,
}

impl Server {
    
    pub fn new(name: String, description: String,has_custom_img: bool, version: String, server_type: ServerType, server_state: ServerState) -> Self {
        Server {
            name,
            description,
            has_custom_img,
            version,
            server_type,
            server_state,
        }
    }

    pub fn get_server_state(&self) -> &ServerState {
        &self.server_state
    }

    pub fn set_server_state(&mut self, server_state: ServerState) {
        self.server_state = server_state;
    }
    
}