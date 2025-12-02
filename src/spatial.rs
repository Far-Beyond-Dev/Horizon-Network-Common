//! Spatial types for 3D world coordinates and region boundaries.
//!
//! These types are used across all Horizon ecosystem components to represent
//! positions in the game world and define region boundaries.

use serde::{Deserialize, Serialize};

/// 3D world coordinates using f64 for precision.
///
/// This type represents a point in the game world with double-precision
/// floating point values for maximum accuracy in large worlds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl WorldCoordinate {
    /// Creates a new world coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a zero coordinate (origin).
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Calculate 3D Euclidean distance to another coordinate.
    pub fn distance_to(&self, other: &WorldCoordinate) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate 3D vector to another coordinate.
    pub fn vector_to(&self, other: &WorldCoordinate) -> WorldCoordinate {
        WorldCoordinate {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z,
        }
    }

    /// Calculate magnitude (length) of this coordinate as a vector.
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize this coordinate as a unit vector.
    pub fn normalized(&self) -> WorldCoordinate {
        let mag = self.magnitude();
        if mag == 0.0 {
            WorldCoordinate::zero()
        } else {
            WorldCoordinate {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        }
    }

    /// Add another coordinate (vector addition).
    pub fn add(&self, other: &WorldCoordinate) -> WorldCoordinate {
        WorldCoordinate {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Scale coordinate by a factor.
    pub fn scale(&self, factor: f64) -> WorldCoordinate {
        WorldCoordinate {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    /// Creates from environment variables (HORIZON_CENTER_X/Y/Z).
    pub fn from_env() -> Self {
        let x = std::env::var("HORIZON_CENTER_X")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let y = std::env::var("HORIZON_CENTER_Y")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let z = std::env::var("HORIZON_CENTER_Z")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        Self { x, y, z }
    }
}

/// Server region coordinates (i64 for grid-based regions).
///
/// This type represents a region's position in a discrete 3D grid,
/// where each cell can contain one server instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct RegionCoordinate {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl RegionCoordinate {
    /// Creates a new region coordinate.
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Center region coordinate (0, 0, 0).
    pub fn center() -> Self {
        Self::new(0, 0, 0)
    }

    /// Calculate Manhattan distance to another region.
    pub fn manhattan_distance(&self, other: &RegionCoordinate) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    /// Get adjacent region coordinates (6 directions in 3D).
    pub fn adjacent_regions(&self) -> Vec<RegionCoordinate> {
        vec![
            RegionCoordinate::new(self.x + 1, self.y, self.z),
            RegionCoordinate::new(self.x - 1, self.y, self.z),
            RegionCoordinate::new(self.x, self.y + 1, self.z),
            RegionCoordinate::new(self.x, self.y - 1, self.z),
            RegionCoordinate::new(self.x, self.y, self.z + 1),
            RegionCoordinate::new(self.x, self.y, self.z - 1),
        ]
    }

    /// Convert region coordinate to world coordinate center.
    ///
    /// Uses the region size to calculate the center point of this region.
    pub fn to_world_center(&self, region_size: f64) -> WorldCoordinate {
        WorldCoordinate::new(
            self.x as f64 * region_size,
            self.y as f64 * region_size,
            self.z as f64 * region_size,
        )
    }

    /// Calculate which region a world coordinate belongs to.
    pub fn from_world_coordinate(coord: &WorldCoordinate, region_size: f64) -> Self {
        Self {
            x: (coord.x / region_size).floor() as i64,
            y: (coord.y / region_size).floor() as i64,
            z: (coord.z / region_size).floor() as i64,
        }
    }

    /// Creates from environment variables (HORIZON_REGION_X/Y/Z).
    pub fn from_env() -> Self {
        let x = std::env::var("HORIZON_REGION_X")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let y = std::env::var("HORIZON_REGION_Y")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let z = std::env::var("HORIZON_REGION_Z")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Self { x, y, z }
    }
}

/// Defines the spatial boundaries of a game region.
///
/// This structure defines a 3D axis-aligned bounding box (AABB) that encompasses
/// all the space within a game region. Compatible with both Horizon and Atlas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegionBounds {
    /// Minimum X coordinate (western boundary)
    pub min_x: f64,
    /// Maximum X coordinate (eastern boundary)
    pub max_x: f64,
    /// Minimum Y coordinate (bottom boundary)
    pub min_y: f64,
    /// Maximum Y coordinate (top boundary)
    pub max_y: f64,
    /// Minimum Z coordinate (southern boundary)
    pub min_z: f64,
    /// Maximum Z coordinate (northern boundary)
    pub max_z: f64,
}

impl Default for RegionBounds {
    fn default() -> Self {
        Self {
            min_x: -1000.0,
            max_x: 1000.0,
            min_y: -1000.0,
            max_y: 1000.0,
            min_z: -1000.0,
            max_z: 1000.0,
        }
    }
}

impl RegionBounds {
    /// Creates a new region bounds from min/max values.
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64) -> Self {
        Self { min_x, max_x, min_y, max_y, min_z, max_z }
    }

    /// Creates region bounds from center point and half-extents.
    pub fn from_center(center: WorldCoordinate, half_extent: f64) -> Self {
        Self {
            min_x: center.x - half_extent,
            max_x: center.x + half_extent,
            min_y: center.y - half_extent,
            max_y: center.y + half_extent,
            min_z: center.z - half_extent,
            max_z: center.z + half_extent,
        }
    }

    /// Get the center point of this region.
    pub fn center(&self) -> WorldCoordinate {
        WorldCoordinate::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            (self.min_z + self.max_z) / 2.0,
        )
    }

    /// Get the half-extent (assuming cubic region).
    pub fn half_extent(&self) -> f64 {
        (self.max_x - self.min_x) / 2.0
    }

    /// Check if a world coordinate is within these bounds.
    pub fn contains(&self, coord: &WorldCoordinate) -> bool {
        coord.x >= self.min_x && coord.x <= self.max_x &&
        coord.y >= self.min_y && coord.y <= self.max_y &&
        coord.z >= self.min_z && coord.z <= self.max_z
    }

    /// Calculate the distance from a point to the nearest boundary.
    /// Returns negative if inside, positive if outside.
    pub fn distance_to_boundary(&self, coord: &WorldCoordinate) -> f64 {
        let dx = (coord.x - self.min_x).min(self.max_x - coord.x);
        let dy = (coord.y - self.min_y).min(self.max_y - coord.y);
        let dz = (coord.z - self.min_z).min(self.max_z - coord.z);
        dx.min(dy).min(dz)
    }

    /// Check if this region overlaps with another.
    pub fn overlaps(&self, other: &RegionBounds) -> bool {
        self.min_x <= other.max_x && self.max_x >= other.min_x &&
        self.min_y <= other.max_y && self.max_y >= other.min_y &&
        self.min_z <= other.max_z && self.max_z >= other.min_z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_coordinate_distance() {
        let a = WorldCoordinate::new(0.0, 0.0, 0.0);
        let b = WorldCoordinate::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_region_bounds_contains() {
        let bounds = RegionBounds::from_center(WorldCoordinate::zero(), 100.0);
        assert!(bounds.contains(&WorldCoordinate::new(0.0, 0.0, 0.0)));
        assert!(bounds.contains(&WorldCoordinate::new(99.0, 0.0, 0.0)));
        assert!(!bounds.contains(&WorldCoordinate::new(101.0, 0.0, 0.0)));
    }

    #[test]
    fn test_region_coordinate_conversion() {
        let world = WorldCoordinate::new(150.0, 50.0, -25.0);
        let region = RegionCoordinate::from_world_coordinate(&world, 100.0);
        assert_eq!(region, RegionCoordinate::new(1, 0, -1));
    }
}
