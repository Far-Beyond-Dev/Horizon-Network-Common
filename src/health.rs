//! Health check types for monitoring Horizon instances.
//!
//! These types are used by Atlas to monitor the health of Horizon servers
//! and by Maestro to check container status.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::server::ServerId;

/// Overall health status of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Service is healthy and operating normally
    Healthy,
    /// Service is experiencing issues but still functional
    Degraded,
    /// Service is not responding or critically failed
    Unhealthy,
    /// Health status is unknown (no recent check)
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl HealthStatus {
    /// Returns true if the service is operational (healthy or degraded).
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// Detailed health check response from a Horizon server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Server identifier
    pub server_id: ServerId,
    /// Overall health status
    pub status: HealthStatus,
    /// Timestamp of this health check
    pub timestamp: DateTime<Utc>,
    /// Current player count
    pub player_count: u32,
    /// Maximum player capacity
    pub capacity: u32,
    /// Server uptime in seconds
    pub uptime_secs: u64,
    /// Average tick rate (ticks per second)
    pub tick_rate: f32,
    /// Memory usage in megabytes
    pub memory_mb: u32,
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Individual component checks
    pub components: Vec<ComponentHealth>,
    /// Optional message
    #[serde(default)]
    pub message: Option<String>,
}

impl HealthCheck {
    /// Creates a healthy status response.
    pub fn healthy(server_id: ServerId, player_count: u32, capacity: u32) -> Self {
        Self {
            server_id,
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            player_count,
            capacity,
            uptime_secs: 0,
            tick_rate: 60.0,
            memory_mb: 0,
            cpu_percent: 0.0,
            components: Vec::new(),
            message: None,
        }
    }

    /// Creates an unhealthy status response.
    pub fn unhealthy(server_id: ServerId, message: String) -> Self {
        Self {
            server_id,
            status: HealthStatus::Unhealthy,
            timestamp: Utc::now(),
            player_count: 0,
            capacity: 0,
            uptime_secs: 0,
            tick_rate: 0.0,
            memory_mb: 0,
            cpu_percent: 0.0,
            components: Vec::new(),
            message: Some(message),
        }
    }

    /// Calculates load factor (0.0 to 1.0).
    pub fn load_factor(&self) -> f32 {
        if self.capacity == 0 {
            0.0
        } else {
            self.player_count as f32 / self.capacity as f32
        }
    }
}

/// Health status of an individual component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component health status
    pub status: HealthStatus,
    /// Optional details
    #[serde(default)]
    pub details: Option<String>,
    /// Response time in milliseconds (if applicable)
    #[serde(default)]
    pub response_time_ms: Option<u64>,
}

impl ComponentHealth {
    /// Creates a healthy component status.
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            details: None,
            response_time_ms: None,
        }
    }

    /// Creates an unhealthy component status.
    pub fn unhealthy(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            details: Some(details.into()),
            response_time_ms: None,
        }
    }
}

/// Health check request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    /// Whether to include detailed component checks
    #[serde(default)]
    pub include_components: bool,
    /// Whether to include system metrics
    #[serde(default)]
    pub include_metrics: bool,
}

impl Default for HealthCheckRequest {
    fn default() -> Self {
        Self {
            include_components: false,
            include_metrics: false,
        }
    }
}

/// Aggregated health status for all servers (used by Atlas).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealth {
    /// Overall cluster status
    pub status: HealthStatus,
    /// Number of healthy servers
    pub healthy_servers: u32,
    /// Number of degraded servers
    pub degraded_servers: u32,
    /// Number of unhealthy servers
    pub unhealthy_servers: u32,
    /// Total players across all servers
    pub total_players: u32,
    /// Total capacity across all servers
    pub total_capacity: u32,
    /// Timestamp of this aggregation
    pub timestamp: DateTime<Utc>,
}

impl ClusterHealth {
    /// Creates a new cluster health summary.
    pub fn new(checks: &[HealthCheck]) -> Self {
        let mut healthy = 0u32;
        let mut degraded = 0u32;
        let mut unhealthy = 0u32;
        let mut total_players = 0u32;
        let mut total_capacity = 0u32;

        for check in checks {
            match check.status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unhealthy | HealthStatus::Unknown => unhealthy += 1,
            }
            total_players += check.player_count;
            total_capacity += check.capacity;
        }

        let status = if unhealthy > 0 && healthy == 0 {
            HealthStatus::Unhealthy
        } else if degraded > 0 || unhealthy > 0 {
            HealthStatus::Degraded
        } else if healthy > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        Self {
            status,
            healthy_servers: healthy,
            degraded_servers: degraded,
            unhealthy_servers: unhealthy,
            total_players,
            total_capacity,
            timestamp: Utc::now(),
        }
    }

    /// Calculates overall load factor.
    pub fn load_factor(&self) -> f32 {
        if self.total_capacity == 0 {
            0.0
        } else {
            self.total_players as f32 / self.total_capacity as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_operational() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
        assert!(!HealthStatus::Unknown.is_operational());
    }

    #[test]
    fn test_cluster_health_aggregation() {
        let server_id = ServerId::new();
        let checks = vec![
            HealthCheck::healthy(server_id, 50, 100),
            HealthCheck::healthy(server_id, 30, 100),
        ];
        let cluster = ClusterHealth::new(&checks);
        assert_eq!(cluster.healthy_servers, 2);
        assert_eq!(cluster.total_players, 80);
        assert_eq!(cluster.total_capacity, 200);
    }
}
