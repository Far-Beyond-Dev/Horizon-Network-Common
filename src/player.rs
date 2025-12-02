//! Player-related types shared across the Horizon ecosystem.
//!
//! These types represent players and their state as they move between
//! different Horizon instances managed by Atlas.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::spatial::WorldCoordinate;
use crate::server::ServerId;

/// Unique identifier for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub Uuid);

impl PlayerId {
    /// Creates a new random player ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a player ID from a string.
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
    }
}

impl Default for PlayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Authentication status of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationStatus {
    /// Player is not authenticated
    #[default]
    Unauthenticated,
    /// Player is in the process of authenticating
    Authenticating,
    /// Player is successfully authenticated
    Authenticated,
    /// Player authentication failed
    AuthenticationFailed,
}

/// Connection state of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// Player is connecting
    #[default]
    Connecting,
    /// Player is connected and active
    Connected,
    /// Player is being transferred to another server
    Transferring,
    /// Player is disconnecting
    Disconnecting,
    /// Player is disconnected
    Disconnected,
}

/// Basic player information tracked by Atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Unique player identifier
    pub id: PlayerId,
    /// Player display name
    pub name: String,
    /// Current authentication status
    pub auth_status: AuthenticationStatus,
    /// Current connection state
    pub connection_state: ConnectionState,
    /// Server the player is currently connected to
    pub current_server: Option<ServerId>,
    /// Last known position in world coordinates
    pub last_position: WorldCoordinate,
    /// Timestamp of last position update
    pub last_updated: DateTime<Utc>,
}

impl PlayerInfo {
    /// Creates new player info.
    pub fn new(id: PlayerId, name: String) -> Self {
        Self {
            id,
            name,
            auth_status: AuthenticationStatus::Unauthenticated,
            connection_state: ConnectionState::Connecting,
            current_server: None,
            last_position: WorldCoordinate::zero(),
            last_updated: Utc::now(),
        }
    }

    /// Updates the player's position.
    pub fn update_position(&mut self, position: WorldCoordinate) {
        self.last_position = position;
        self.last_updated = Utc::now();
    }
}

/// Player state that can be serialized for transfer between servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    /// Player information
    pub info: PlayerInfo,
    /// Velocity vector
    pub velocity: WorldCoordinate,
    /// Player health (0.0 to 1.0)
    pub health: f32,
    /// Custom state data (game-specific)
    #[serde(default)]
    pub custom_data: std::collections::HashMap<String, serde_json::Value>,
    /// Inventory or other persistent data
    #[serde(default)]
    pub persistent_data: serde_json::Value,
}

impl PlayerState {
    /// Creates a new player state from player info.
    pub fn new(info: PlayerInfo) -> Self {
        Self {
            info,
            velocity: WorldCoordinate::zero(),
            health: 1.0,
            custom_data: std::collections::HashMap::new(),
            persistent_data: serde_json::Value::Null,
        }
    }

    /// Serializes the player state to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes player state from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Movement data for player position prediction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct MovementData {
    /// Current velocity
    pub velocity: WorldCoordinate,
    /// Acceleration
    pub acceleration: WorldCoordinate,
    /// Timestamp of this movement data
    pub timestamp_ms: u64,
}

impl MovementData {
    /// Predicts position after the given time delta.
    pub fn predict_position(&self, current_pos: WorldCoordinate, delta_ms: u64) -> WorldCoordinate {
        let t = delta_ms as f64 / 1000.0;
        // Simple kinematic equation: p = p0 + v*t + 0.5*a*t^2
        WorldCoordinate::new(
            current_pos.x + self.velocity.x * t + 0.5 * self.acceleration.x * t * t,
            current_pos.y + self.velocity.y * t + 0.5 * self.acceleration.y * t * t,
            current_pos.z + self.velocity.z * t + 0.5 * self.acceleration.z * t * t,
        )
    }
}

/// Reasons for player disconnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisconnectReason {
    /// Player initiated disconnection
    ClientDisconnect,
    /// Connection timed out
    Timeout,
    /// Server is shutting down
    ServerShutdown,
    /// Player was kicked
    Kicked { reason: String },
    /// Player is being transferred
    Transfer { target_server: ServerId },
    /// An error occurred
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_state_serialization() {
        let info = PlayerInfo::new(PlayerId::new(), "TestPlayer".to_string());
        let state = PlayerState::new(info);
        let json = state.to_json().unwrap();
        let restored = PlayerState::from_json(&json).unwrap();
        assert_eq!(restored.info.name, "TestPlayer");
    }

    #[test]
    fn test_movement_prediction() {
        let movement = MovementData {
            velocity: WorldCoordinate::new(10.0, 0.0, 0.0),
            acceleration: WorldCoordinate::zero(),
            timestamp_ms: 0,
        };
        let pos = WorldCoordinate::zero();
        let predicted = movement.predict_position(pos, 1000); // 1 second
        assert!((predicted.x - 10.0).abs() < 0.001);
    }
}
