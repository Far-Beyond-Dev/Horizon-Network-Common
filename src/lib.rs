//! # Horizon Network Common
//!
//! Shared types and protocols for the Horizon ecosystem:
//! - **Horizon** - Game server instances (regions)
//! - **Horizon-Atlas** - Server meshing proxy and load balancer
//! - **Horizon-Maestro** - Container orchestration and deployment
//!
//! This crate provides the common data structures and communication protocols
//! that enable these three systems to work together for infinite world scaling.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         Maestro                                  │
//! │              (Container Orchestration)                           │
//! └─────────────────────────┬───────────────────────────────────────┘
//!                           │ SpawnServerRequest / SpawnServerResponse
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                          Atlas                                   │
//! │        (Server Meshing Proxy + Player Routing)                   │
//! └────────┬────────────────┬───────────────────┬───────────────────┘
//!          │                │                   │
//!          │ Registration   │ Heartbeat         │ Transfer
//!          ▼                ▼                   ▼
//! ┌────────────────┐ ┌────────────────┐ ┌────────────────┐
//! │   Horizon      │ │   Horizon      │ │   Horizon      │
//! │  Region(0,0,0) │ │  Region(1,0,0) │ │  Region(0,1,0) │
//! └────────────────┘ └────────────────┘ └────────────────┘
//! ```

pub mod spatial;
pub mod server;
pub mod player;
pub mod transfer;
pub mod health;
pub mod messages;

// Re-export commonly used types

// Spatial types
pub use spatial::{WorldCoordinate, RegionCoordinate, RegionBounds};

// Server types (full structured types)
pub use server::{
    ServerId, ServerInfo, ServerStatus, ServerRegistration, ServerHeartbeat, 
    RegistrationResponse, SpawnServerRequest, SpawnServerResponse,
};

// API-compatible types (flat structures for REST APIs)
pub use server::{
    ApiServerRegistration, ApiRegistrationResponse, AdjacentServerInfo,
    ApiServerHeartbeat, ApiHeartbeatResponse, ServerCommand,
};

// Player types  
pub use player::{PlayerId, PlayerInfo, PlayerState, AuthenticationStatus, ConnectionState, MovementData, DisconnectReason};

// Transfer types
pub use transfer::{TransferToken, TransferRequest, TransferResult, TransferReason, TransferError, TransferNotification};

// Health check types
pub use health::{HealthStatus, HealthCheck, HealthCheckRequest, ComponentHealth, ClusterHealth};

// Inter-service message types
pub use messages::{HorizonMessage, AtlasMessage, AtlasToMaestroMessage, MaestroMessage, Envelope, Ack};
