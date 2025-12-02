//! Message types for inter-service communication.
//!
//! These types define the messages exchanged between Horizon, Atlas, and Maestro
//! for coordination and control.

use serde::{Deserialize, Serialize};

use crate::health::{HealthCheck, HealthCheckRequest};
use crate::player::{PlayerId, PlayerState, DisconnectReason};
use crate::server::{
    ServerHeartbeat, ServerInfo, ServerRegistration, ServerId,
    RegistrationResponse, SpawnServerRequest, SpawnServerResponse,
};
use crate::transfer::{TransferRequest, TransferToken};
use crate::spatial::WorldCoordinate;

/// Messages sent from Horizon to Atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HorizonMessage {
    /// Server registration request
    Register(ServerRegistration),
    
    /// Periodic heartbeat
    Heartbeat(ServerHeartbeat),
    
    /// Health check response
    HealthResponse(HealthCheck),
    
    /// Player connected notification
    PlayerConnected {
        player_id: PlayerId,
        position: WorldCoordinate,
    },
    
    /// Player disconnected notification
    PlayerDisconnected {
        player_id: PlayerId,
        reason: DisconnectReason,
    },
    
    /// Player position update
    PlayerPositionUpdate {
        player_id: PlayerId,
        position: WorldCoordinate,
        velocity: WorldCoordinate,
    },
    
    /// Request to transfer a player (player approaching boundary)
    TransferRequest(TransferRequest),
    
    /// Transfer completed successfully
    TransferComplete {
        player_id: PlayerId,
        success: bool,
        error: Option<String>,
    },
    
    /// Player accepted from transfer
    TransferAccepted {
        player_id: PlayerId,
        token_id: String,
    },
    
    /// Server shutting down
    Shutdown {
        server_id: ServerId,
        player_count: u32,
    },
}

/// Messages sent from Atlas to Horizon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AtlasMessage {
    /// Registration response
    RegistrationResponse(RegistrationResponse),
    
    /// Health check request
    HealthCheckRequest(HealthCheckRequest),
    
    /// Initiate player transfer
    InitiateTransfer {
        player_id: PlayerId,
        target_server: ServerInfo,
        token: TransferToken,
    },
    
    /// Accept incoming transfer
    AcceptTransfer {
        token: TransferToken,
        player_state: PlayerState,
    },
    
    /// Cancel pending transfer
    CancelTransfer {
        player_id: PlayerId,
        reason: String,
    },
    
    /// Prepare for shutdown
    PrepareShutdown {
        deadline_secs: u32,
    },
    
    /// Update adjacent servers list
    AdjacentServersUpdate {
        servers: Vec<ServerInfo>,
    },
    
    /// Configuration update
    ConfigUpdate {
        config: serde_json::Value,
    },
}

/// Messages sent from Atlas to Maestro.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AtlasToMaestroMessage {
    /// Request to spawn a new server instance
    SpawnServer(SpawnServerRequest),
    
    /// Request to stop a server instance
    StopServer {
        instance_id: String,
        graceful: bool,
    },
    
    /// Request server health/stats
    GetServerStats {
        instance_id: String,
    },
    
    /// Scale cluster to target count
    ScaleCluster {
        target_count: u32,
    },
}

/// Messages sent from Maestro to Atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MaestroMessage {
    /// Server spawn response
    SpawnResponse(SpawnServerResponse),
    
    /// Server stopped notification
    ServerStopped {
        instance_id: String,
        exit_code: Option<i32>,
    },
    
    /// Server stats response
    ServerStats {
        instance_id: String,
        cpu_percent: f32,
        memory_mb: u32,
        running: bool,
    },
    
    /// Cluster scaled
    ClusterScaled {
        current_count: u32,
        target_count: u32,
    },
    
    /// Error occurred
    Error {
        operation: String,
        message: String,
    },
}

/// Wrapper for all message types with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// Message ID for tracking
    pub id: String,
    /// Timestamp in milliseconds since epoch
    pub timestamp_ms: u64,
    /// Source service identifier
    pub source: String,
    /// Destination service identifier
    pub destination: String,
    /// The actual message
    pub message: T,
}

impl<T> Envelope<T> {
    /// Creates a new envelope with the given message.
    pub fn new(source: impl Into<String>, destination: impl Into<String>, message: T) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source: source.into(),
            destination: destination.into(),
            message,
        }
    }
}

/// Simple acknowledgment response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ack {
    /// Message ID being acknowledged
    pub message_id: String,
    /// Whether the message was processed successfully
    pub success: bool,
    /// Optional error message
    #[serde(default)]
    pub error: Option<String>,
}

impl Ack {
    /// Creates a successful acknowledgment.
    pub fn success(message_id: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            success: true,
            error: None,
        }
    }

    /// Creates a failed acknowledgment.
    pub fn failure(message_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            success: false,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizon_message_serialization() {
        let msg = HorizonMessage::PlayerConnected {
            player_id: PlayerId::new(),
            position: WorldCoordinate::new(100.0, 50.0, -200.0),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let restored: HorizonMessage = serde_json::from_str(&json).unwrap();
        match restored {
            HorizonMessage::PlayerConnected { position, .. } => {
                assert!((position.x - 100.0).abs() < 0.001);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_envelope_creation() {
        let msg = Ack::success("test-123");
        let envelope = Envelope::new("horizon-1", "atlas", msg);
        assert_eq!(envelope.source, "horizon-1");
        assert_eq!(envelope.destination, "atlas");
    }
}
