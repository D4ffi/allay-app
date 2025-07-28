use crate::models::query::{QueryResponse, QueryConfig};
use std::io::{self, Write, Read};
use std::net::{UdpSocket, ToSocketAddrs};
use std::time::Duration;

pub struct QueryService {
    config: QueryConfig,
}

impl QueryService {
    pub fn new(config: QueryConfig) -> Self {
        QueryService { config }
    }

    pub fn with_default() -> Self {
        QueryService::new(QueryConfig::default())
    }

    /// Performs a basic server list ping to check if server is online
    /// This is simpler than full query protocol and works for most servers
    pub async fn ping_server(&self) -> QueryResponse {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let timeout = self.config.timeout_ms;
        
        // Run blocking operation in a separate thread
        let result = tokio::task::spawn_blocking(move || {
            Self::perform_basic_ping_blocking(&address, timeout)
        }).await;
        
        match result {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => QueryResponse::offline(format!("Ping failed: {}", e)),
            Err(e) => QueryResponse::offline(format!("Task failed: {}", e)),
        }
    }

    /// Performs a more detailed query using the Minecraft Query protocol
    /// This requires enable-query=true in server.properties
    pub async fn query_server(&self) -> QueryResponse {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let timeout = self.config.timeout_ms;
        
        // Clone for potential fallback use
        let address_clone = address.clone();
        let timeout_clone = timeout;
        
        // Run a blocking operation in a separate thread
        let result = tokio::task::spawn_blocking(move || {
            Self::perform_query_blocking(&address, timeout)
        }).await;
        
        match result {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                // Fallback to basic ping if a query fails
                let ping_result = tokio::task::spawn_blocking(move || {
                    Self::perform_basic_ping_blocking(&address_clone, timeout_clone)
                }).await;
                
                match ping_result {
                    Ok(Ok(ping_response)) => ping_response,
                    _ => QueryResponse::offline(format!("Query and ping failed: {}", e)),
                }
            },
            Err(e) => QueryResponse::offline(format!("Task failed: {}", e)),
        }
    }

    fn perform_basic_ping_blocking(address: &str, timeout_ms: u64) -> Result<QueryResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Use a TCP connection attempt as a basic "ping"
        // This is the most reliable way to check if a Minecraft server is accepting connections
        
        let socket_addr = address.to_socket_addrs()?.next().ok_or("Invalid address")?;
        
        match std::net::TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms)) {
            Ok(_) => {
                // Server is accepting connections
                Ok(QueryResponse::online(0, 0, "Server Online".to_string(), "Unknown".to_string()))
            },
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }

    fn perform_query_blocking(address: &str, timeout_ms: u64) -> Result<QueryResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Minecraft Query Protocol implementation
        // This is more complex but provides detailed information
        
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
        socket.set_write_timeout(Some(Duration::from_millis(timeout_ms)))?;

        let target_addr = address.to_socket_addrs()?.next().ok_or("Invalid address")?;

        // Step 1: Handshake
        let session_id = 1u32;
        let handshake_packet = Self::create_handshake_packet(session_id);
        socket.send_to(&handshake_packet, target_addr)?;

        // Read handshake response
        let mut buffer = [0u8; 1024];
        let (size, _) = socket.recv_from(&mut buffer)?;
        let challenge_token = Self::parse_handshake_response(&buffer[..size])?;

        // Step 2: Query
        let query_packet = Self::create_query_packet(session_id, challenge_token);
        socket.send_to(&query_packet, target_addr)?;

        // Read query response
        let (size, _) = socket.recv_from(&mut buffer)?;
        let response = Self::parse_query_response(&buffer[..size])?;

        Ok(response)
    }

    fn create_handshake_packet(session_id: u32) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&[0xFE, 0xFD]); // Magic
        packet.push(0x09); // Handshake type
        packet.extend_from_slice(&session_id.to_be_bytes());
        packet
    }

    fn parse_handshake_response(data: &[u8]) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        if data.len() < 5 {
            return Err("Invalid handshake response".into());
        }
        
        // Skip header and session ID, extract challenge token
        let token_str = std::str::from_utf8(&data[5..data.len()-1])?;
        let token = token_str.parse::<u32>()?;
        Ok(token)
    }

    fn create_query_packet(session_id: u32, challenge_token: u32) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&[0xFE, 0xFD]); // Magic
        packet.push(0x00); // Query type
        packet.extend_from_slice(&session_id.to_be_bytes());
        packet.extend_from_slice(&challenge_token.to_be_bytes());
        packet
    }

    fn parse_query_response(data: &[u8]) -> Result<QueryResponse, Box<dyn std::error::Error + Send + Sync>> {
        if data.len() < 5 {
            return Err("Invalid query response".into());
        }

        // Skip header
        let mut offset = 5;
        
        // Parse K-V pairs
        let mut motd = String::new();
        let mut players_online = 0u32;
        let mut players_max = 0u32;
        let mut version = String::new();

        while offset < data.len() {
            // Find null-terminated key
            let key_end = data[offset..].iter().position(|&b| b == 0).unwrap_or(0) + offset;
            if key_end >= data.len() { break; }
            
            let key = std::str::from_utf8(&data[offset..key_end])?;
            offset = key_end + 1;

            // Find null-terminated value
            let value_end = data[offset..].iter().position(|&b| b == 0).unwrap_or(0) + offset;
            if value_end >= data.len() { break; }
            
            let value = std::str::from_utf8(&data[offset..value_end])?;
            offset = value_end + 1;

            match key {
                "hostname" => motd = value.to_string(),
                "numplayers" => players_online = value.parse().unwrap_or(0),
                "maxplayers" => players_max = value.parse().unwrap_or(0),
                "version" => version = value.to_string(),
                _ => {} // Ignore other fields
            }
        }

        Ok(QueryResponse::online(players_online, players_max, motd, version))
    }
}

// Convenience functions for common operations
impl QueryService {
    pub fn set_host(&mut self, host: String) {
        self.config.host = host;
    }

    pub fn set_port(&mut self, port: u16) {
        self.config.port = port;
    }

    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }
}