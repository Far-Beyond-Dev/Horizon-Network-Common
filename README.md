# Horizon Network Common

Shared network types and protocols for the Horizon game server ecosystem.

## Overview

This crate provides the common data structures and communication protocols used by:

- **[Horizon](https://github.com/Far-Beyond-Dev/Horizon)** - Game server instances (regions)
- **[Horizon-Atlas](https://github.com/Far-Beyond-Dev/Horizon-Atlas)** - Server meshing proxy and load balancer
- **[Horizon-Maestro](https://github.com/Far-Beyond-Dev/Horizon-Maestro)** - Container orchestration and deployment

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
horizon_network_common = "0.1"
```

Or with git:

```toml
[dependencies]
horizon_network_common = { git = "https://github.com/Far-Beyond-Dev/Horizon-Network-Common" }
```

## Modules

| Module | Description |
|--------|-------------|
| `spatial` | `WorldCoordinate`, `RegionCoordinate`, `RegionBounds` |
| `server` | `ServerId`, `ServerInfo`, `ServerRegistration`, `ServerHeartbeat` |
| `player` | `PlayerId`, `PlayerInfo`, `PlayerState`, `MovementData` |
| `transfer` | `TransferToken`, `TransferRequest`, `TransferResult` |
| `health` | `HealthStatus`, `HealthCheck`, `ClusterHealth` |
| `messages` | `HorizonMessage`, `AtlasMessage`, `MaestroMessage` |

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                         Maestro                                  │
│              (Container Orchestration)                           │
└─────────────────────────┬───────────────────────────────────────┘
                          │ SpawnServerRequest / SpawnServerResponse
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Atlas                                   │
│        (Server Meshing Proxy + Player Routing)                   │
└────────┬────────────────┬───────────────────┬───────────────────┘
         │                │                   │
         │ Registration   │ Heartbeat         │ Transfer
         ▼                ▼                   ▼
┌────────────────┐ ┌────────────────┐ ┌────────────────┐
│   Horizon      │ │   Horizon      │ │   Horizon      │
│  Region(0,0,0) │ │  Region(1,0,0) │ │  Region(0,1,0) │
└────────────────┘ └────────────────┘ └────────────────┘
```

## Usage

```rust
use horizon_network_common::{
    WorldCoordinate, RegionCoordinate, RegionBounds,
    ServerInfo, ServerRegistration,
    PlayerId, PlayerInfo,
};

// Create a region coordinate
let region = RegionCoordinate::new(1, 0, 0);

// Convert to world position
let world_pos = region.to_world_center(1000.0);

// Check if a position is in bounds
let bounds = RegionBounds::from_center(world_pos, 500.0);
assert!(bounds.contains(&world_pos));
```

## Key Types

### Spatial Types

- `WorldCoordinate` - 3D position with f64 precision
- `RegionCoordinate` - Discrete 3D grid position (i64)
- `RegionBounds` - AABB bounding box for regions

### Server Types

- `ServerId` - Unique server identifier (UUID)
- `ServerInfo` - Server metadata and capabilities
- `ServerRegistration` - Registration request from Horizon to Atlas
- `ServerHeartbeat` - Periodic health/load updates

### Player Types

- `PlayerId` - Unique player identifier (UUID)
- `PlayerInfo` - Player metadata and state
- `PlayerState` - Complete player state for transfers
- `MovementData` - Velocity/acceleration for prediction

### Transfer Types

- `TransferToken` - Cryptographically signed transfer authorization
- `TransferRequest` - Request to move player between servers
- `TransferResult` - Outcome of a transfer operation

### Message Types

- `HorizonMessage` - Messages from Horizon to Atlas
- `AtlasMessage` - Messages from Atlas to Horizon
- `MaestroMessage` - Messages from Maestro to Atlas
- `Envelope<T>` - Message wrapper with metadata

## License

MIT
