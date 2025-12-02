//! Server registration and status types.
//!
//! These types are used for Horizon instances to register with Atlas
//! and report their status and availability.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::spatial::{RegionBounds, RegionCoordinate, WorldCoordinate};

/// Unique identifier for a Horizon server instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerId(pub Uuid);

impl ServerId {
    /// Creates a new random server ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a server ID from a string.
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
    }
}

impl Default for ServerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Current status of a Horizon server instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    /// Server is starting up
    Starting,
    /// Server is running and accepting connections
    Running,
    /// Server is draining connections (preparing to shutdown)
    Draining,
    /// Server is stopped
    Stopped,
    /// Server encountered an error
    Error,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Starting
    }
}

/// Basic server information for registration and discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Unique server identifier
    pub id: ServerId,
    /// Human-readable server name
    pub name: String,
    /// Server address (host:port)
    pub address: String,
    /// Region coordinate in the world grid
    pub region_coord: RegionCoordinate,
    /// Spatial bounds of this server's region
    pub bounds: RegionBounds,
    /// Center point of the region in world coordinates
    pub center: WorldCoordinate,
    /// Maximum number of connections this server can handle
    pub capacity: u32,
    /// Server version string
    pub version: String,
}

impl ServerInfo {
    /// Creates new server info with the given parameters.
    pub fn new(
        name: String,
        address: String,
        region_coord: RegionCoordinate,
        bounds: RegionBounds,
        capacity: u32,
    ) -> Self {
        Self {
            id: ServerId::new(),
            name,
            address,
            region_coord,
            bounds,
            center: bounds.center(),
            capacity,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Server registration request sent from Horizon to Atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistration {
    /// Server information
    pub server: ServerInfo,
    /// Current server status
    pub status: ServerStatus,
    /// Timestamp of registration
    pub registered_at: DateTime<Utc>,
    /// Optional metadata for custom properties
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ServerRegistration {
    /// Creates a new server registration.
    pub fn new(server: ServerInfo) -> Self {
        Self {
            server,
            status: ServerStatus::Starting,
            registered_at: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Server heartbeat sent periodically from Horizon to Atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHeartbeat {
    /// Server ID
    pub server_id: ServerId,
    /// Current server status
    pub status: ServerStatus,
    /// Current number of connected players
    pub current_connections: u32,
    /// Server load (0.0 to 1.0)
    pub load: f32,
    /// Timestamp of this heartbeat
    pub timestamp: DateTime<Utc>,
    /// Average tick time in milliseconds
    #[serde(default)]
    pub avg_tick_ms: f64,
    /// Memory usage in bytes
    #[serde(default)]
    pub memory_bytes: u64,
}

impl ServerHeartbeat {
    /// Creates a new heartbeat with current metrics.
    pub fn new(
        server_id: ServerId,
        status: ServerStatus,
        current_connections: u32,
        capacity: u32,
    ) -> Self {
        let load = if capacity > 0 {
            current_connections as f32 / capacity as f32
        } else {
            0.0
        };

        Self {
            server_id,
            status,
            current_connections,
            load,
            timestamp: Utc::now(),
            avg_tick_ms: 0.0,
            memory_bytes: 0,
        }
    }
}

/// Response from Atlas when a server registers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    /// Whether registration was successful
    pub success: bool,
    /// Assigned server ID (may differ from requested if conflict)
    pub server_id: ServerId,
    /// Message describing the result
    pub message: String,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u32,
    /// List of adjacent servers for cross-region communication
    #[serde(default)]
    pub adjacent_servers: Vec<ServerInfo>,
}

/// Request from Atlas to Maestro to spawn a new Horizon instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnServerRequest {
    /// Requested region coordinate
    pub region_coord: RegionCoordinate,
    /// Region bounds for the new server
    pub bounds: RegionBounds,
    /// Optional preferred name
    pub name: Option<String>,
    /// Environment variables to pass to the container
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
}

/// Response from Maestro after spawning a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnServerResponse {
    /// Whether spawn was successful
    pub success: bool,
    /// Container/instance ID
    pub instance_id: String,
    /// Server address once running
    pub address: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_registration() {
        let info = ServerInfo::new(
            "test-server".to_string(),
            "127.0.0.1:8080".to_string(),
            RegionCoordinate::center(),
            RegionBounds::default(),
            100,
        );
        let reg = ServerRegistration::new(info);
        assert_eq!(reg.status, ServerStatus::Starting);
    }

    #[test]
    fn test_heartbeat_load() {
        let heartbeat = ServerHeartbeat::new(
            ServerId::new(),
            ServerStatus::Running,
            50,
            100,
        );
        assert!((heartbeat.load - 0.5).abs() < 0.001);
    }
}
