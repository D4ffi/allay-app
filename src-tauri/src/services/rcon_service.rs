use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

// RCON Protocol Constants
const RCON_TYPE_LOGIN: i32 = 3;
const RCON_TYPE_COMMAND: i32 = 2;
const RCON_TYPE_RESPONSE: i32 = 0;

#[derive(Debug, Clone)]
pub enum RconError {
    ConnectionFailed(String),
    AuthenticationFailed,
    CommandFailed(String),
    InvalidResponse,
    NotConnected,
    BufferError(String),      // Specific for "failed to fill whole buffer" errors
    ServerClosedConnection,   // Server closed connection prematurely
    NetworkTimeout,           // Network timeout errors
}

impl std::fmt::Display for RconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RconError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            RconError::AuthenticationFailed => write!(f, "Authentication failed"),
            RconError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            RconError::InvalidResponse => write!(f, "Invalid response from server"),
            RconError::NotConnected => write!(f, "Not connected to server"),
            RconError::BufferError(msg) => write!(f, "Buffer error: {}", msg),
            RconError::ServerClosedConnection => write!(f, "Server closed connection"),
            RconError::NetworkTimeout => write!(f, "Network timeout"),
        }
    }
}

impl std::error::Error for RconError {}

#[derive(Debug)]
pub struct RconPacket {
    pub request_id: i32,
    pub packet_type: i32,
    pub payload: String,
}

pub struct RconConnection {
    stream: Option<TcpStream>,
    host: String,
    port: u16,
    password: String,
    request_id: i32,
    authenticated: bool,
    last_heartbeat: Option<Instant>,
    connection_lost: bool,
    pending_commands: std::collections::HashMap<i32, String>,
}

impl RconConnection {
    pub fn new(host: String, port: u16, password: String) -> Self {
        Self {
            stream: None,
            host,
            port,
            password,
            request_id: 1,
            authenticated: false,
            last_heartbeat: None,
            connection_lost: false,
            pending_commands: HashMap::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.authenticated && !self.connection_lost
    }

    pub fn needs_heartbeat(&self) -> bool {
        // No longer using active heartbeat - server sends Keep Alive messages
        false
    }

    pub fn connect(&mut self) -> Result<(), RconError> {
        let address = format!("{}:{}", self.host, self.port);
        println!("Connecting to RCON server at {}", address);

        // Validate host and port first
        if self.host.is_empty() {
            return Err(RconError::ConnectionFailed("Host cannot be empty".to_string()));
        }
        if self.port == 0 {
            return Err(RconError::ConnectionFailed("Port cannot be zero".to_string()));
        }

        let socket_addr = address.parse().map_err(|e| {
            println!("Failed to parse address '{}': {}", address, e);
            RconError::ConnectionFailed(format!("Invalid address '{}': {}", address, e))
        })?;

        println!("Parsed socket address: {:?}", socket_addr);

        // Try connecting with a shorter timeout first
        let stream = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3)) {
            Ok(stream) => {
                println!("Successfully connected to RCON at {}", socket_addr);
                stream
            },
            Err(e) => {
                println!("Failed to connect to RCON at {}: {}", socket_addr, e);
                println!("Error kind: {:?}", e.kind());
                
                // Try again with a fallback approach
                match TcpStream::connect(socket_addr) {
                    Ok(stream) => {
                        println!("Fallback connection succeeded to {}", socket_addr);
                        stream
                    },
                    Err(e2) => {
                        println!("Fallback connection also failed: {}", e2);
                        return Err(RconError::ConnectionFailed(format!(
                            "Failed to connect to RCON at {}: {} (fallback: {})", 
                            socket_addr, e, e2
                        )));
                    }
                }
            }
        };

        // Set longer timeouts for better stability
        stream.set_read_timeout(Some(Duration::from_secs(60)))
            .map_err(|e| RconError::ConnectionFailed(e.to_string()))?;
        stream.set_write_timeout(Some(Duration::from_secs(60)))
            .map_err(|e| RconError::ConnectionFailed(e.to_string()))?;

        self.stream = Some(stream);
        self.connection_lost = false;
        
        // Authenticate immediately after connection
        self.authenticate()?;
        
        // Set initial heartbeat time
        self.last_heartbeat = Some(Instant::now());
        
        println!("RCON connection established and authenticated");
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(stream) = &mut self.stream {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        self.stream = None;
        self.authenticated = false;
        self.connection_lost = true;
        self.last_heartbeat = None;
        self.pending_commands.clear();
        println!("RCON connection closed");
    }

    pub fn reconnect(&mut self) -> Result<(), RconError> {
        println!("Attempting to reconnect RCON...");
        self.disconnect();
        self.connect()
    }

    fn authenticate(&mut self) -> Result<(), RconError> {
        if self.stream.is_none() {
            return Err(RconError::NotConnected);
        }

        println!("Authenticating with RCON server using password: '{}'", self.password);
        
        let auth_id = self.request_id;
        self.request_id += 1;

        println!("Sending authentication packet with ID: {}", auth_id);
        self.send_packet(auth_id, RCON_TYPE_LOGIN, &self.password.clone()).map_err(|e| {
            println!("Failed to send authentication packet: {}", e);
            self.connection_lost = true;
            e
        })?;
        
        println!("Waiting for authentication response...");
        let response = self.receive_packet().map_err(|e| {
            println!("Failed to receive authentication response: {}", e);
            self.connection_lost = true;
            e
        })?;

        println!("Received authentication response - ID: {}, expected: {}, type: {}", 
                 response.request_id, auth_id, response.packet_type);

        if response.request_id != auth_id {
            println!("Authentication failed: request ID mismatch");
            self.connection_lost = true;
            return Err(RconError::AuthenticationFailed);
        }

        if response.request_id == -1 {
            println!("Authentication failed: server rejected password");
            self.connection_lost = true;
            return Err(RconError::AuthenticationFailed);
        }

        self.authenticated = true;
        println!("RCON authentication successful");
        
        // Small delay to let the server stabilize the RCON connection
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, RconError> {
        if !self.is_connected() {
            return Err(RconError::NotConnected);
        }

        println!("Executing RCON command: {}", command);
        
        // Small delay before sending command to ensure connection is stable
        thread::sleep(Duration::from_millis(50));

        let cmd_id = self.request_id;
        self.request_id += 1;

        // Send the command
        match self.send_packet(cmd_id, RCON_TYPE_COMMAND, command) {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to send packet, marking connection as lost: {}", e);
                self.connection_lost = true;
                return Err(e);
            }
        }

        // Receive response with simple timeout
        let response = match self.receive_packet() {
            Ok(response) => response,
            Err(e) => {
                println!("Failed to receive packet, marking connection as lost: {}", e);
                self.connection_lost = true;
                return Err(e);
            }
        };

        // Validate response ID
        if response.request_id != cmd_id {
            println!("Received unexpected response ID: {} (expected: {})", response.request_id, cmd_id);
            
            // If this is a Keep Alive message, handle it and try again
            if response.payload.trim().to_lowercase() == "keep alive" || response.payload.trim().is_empty() {
                println!("📡 Received Keep Alive, trying to get actual response...");
                
                // Try to get the real response
                match self.receive_packet() {
                    Ok(real_response) => {
                        if real_response.request_id == cmd_id {
                            self.last_heartbeat = Some(Instant::now());
                            println!("RCON command response: {}", real_response.payload);
                            return Ok(real_response.payload);
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive real response after Keep Alive: {}", e);
                        return Err(e);
                    }
                }
            }
            
            return Err(RconError::InvalidResponse);
        }

        // Update heartbeat on successful command
        self.last_heartbeat = Some(Instant::now());
        
        println!("RCON command response: {}", response.payload);
        Ok(response.payload)
    }

    pub fn heartbeat(&mut self) -> Result<(), RconError> {
        // No longer using active heartbeat - server handles Keep Alive
        println!("RCON heartbeat: Using passive Keep Alive handling");
        Ok(())
    }

    pub fn handle_keep_alive(&mut self) -> Result<(), RconError> {
        // Simplified - just update heartbeat time to show we're alive
        if self.is_connected() {
            self.last_heartbeat = Some(Instant::now());
            Ok(())
        } else {
            Err(RconError::NotConnected)
        }
    }

    fn send_packet(&mut self, request_id: i32, packet_type: i32, payload: &str) -> Result<(), RconError> {
        let stream = self.stream.as_mut().ok_or(RconError::NotConnected)?;
        
        let payload_bytes = payload.as_bytes();
        let packet_size = 4 + 4 + payload_bytes.len() + 2; // id + type + payload + null terminators

        stream.write_i32::<LittleEndian>(packet_size as i32)
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        stream.write_i32::<LittleEndian>(request_id)
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        stream.write_i32::<LittleEndian>(packet_type)
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        stream.write_all(payload_bytes)
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        stream.write_u8(0) // null terminator for payload
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        stream.write_u8(0) // null terminator for packet
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;

        stream.flush()
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;

        Ok(())
    }

    fn receive_packet(&mut self) -> Result<RconPacket, RconError> {
        let stream = self.stream.as_mut().ok_or(RconError::NotConnected)?;
        
        let packet_size = stream.read_i32::<LittleEndian>()
            .map_err(|e| {
                self.connection_lost = true;
                
                // Classify the specific error
                let error_msg = e.to_string();
                if error_msg.contains("failed to fill whole buffer") {
                    RconError::BufferError(error_msg)
                } else if error_msg.contains("Connection reset") || error_msg.contains("Connection aborted") {
                    RconError::ServerClosedConnection
                } else if error_msg.contains("timed out") || error_msg.contains("timeout") {
                    RconError::NetworkTimeout
                } else {
                    RconError::CommandFailed(format!("Failed to read packet size: {}", e))
                }
            })?;

        if packet_size < 10 || packet_size > 4096 {
            self.connection_lost = true;
            return Err(RconError::InvalidResponse);
        }

        let request_id = stream.read_i32::<LittleEndian>()
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;
        let packet_type = stream.read_i32::<LittleEndian>()
            .map_err(|e| {
                self.connection_lost = true;
                RconError::CommandFailed(e.to_string())
            })?;

        let payload_size = packet_size - 8; // subtract id and type
        
        let mut payload_buffer = vec![0u8; payload_size as usize];
        
        // Use a more robust reading approach
        let mut bytes_read = 0;
        while bytes_read < payload_size as usize {
            match stream.read(&mut payload_buffer[bytes_read..]) {
                Ok(0) => {
                    self.connection_lost = true;
                    return Err(RconError::ServerClosedConnection);
                },
                Ok(n) => {
                    bytes_read += n;
                },
                Err(e) => {
                    self.connection_lost = true;
                    
                    // Classify the specific error
                    let error_msg = e.to_string();
                    if error_msg.contains("failed to fill whole buffer") {
                        return Err(RconError::BufferError(error_msg));
                    } else if error_msg.contains("Connection reset") || error_msg.contains("Connection aborted") {
                        return Err(RconError::ServerClosedConnection);
                    } else if error_msg.contains("timed out") || error_msg.contains("timeout") {
                        return Err(RconError::NetworkTimeout);
                    } else {
                        return Err(RconError::CommandFailed(format!("Failed to read payload: {}", e)));
                    }
                }
            }
        }

        // Remove null terminators
        while payload_buffer.last() == Some(&0) {
            payload_buffer.pop();
        }

        let payload = String::from_utf8_lossy(&payload_buffer).to_string();

        Ok(RconPacket {
            request_id,
            packet_type,
            payload,
        })
    }
}

impl Drop for RconConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}