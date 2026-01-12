//! Dirty Flag Pattern for Cached Computations
//!
//! Provides wrappers that track when data changes and cache expensive computations
//! like matrix calculations. The cache is invalidated when the underlying data is mutated.
//!
//! # Design Principles
//!
//! - **Lazy Evaluation**: Expensive computations only run when needed
//! - **Automatic Invalidation**: Any mutation marks the cache as dirty
//! - **Zero-Cost When Clean**: Reading cached values has no overhead
//! - **Transparent API**: Works like the wrapped type with caching benefits
//!
//! # Example
//!
//! ```ignore
//! // Create a cached transform
//! let mut cached = CachedTransform::new();
//!
//! // First access computes the matrix
//! let matrix = cached.world_matrix();  // Computes and caches
//!
//! // Subsequent accesses use the cache
//! let matrix2 = cached.world_matrix();  // Returns cached value (fast!)
//!
//! // Mutations invalidate the cache
//! cached.set_position(Vec3::new(1.0, 0.0, 0.0));
//! let matrix3 = cached.world_matrix();  // Recomputes (position changed)
//! ```

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

use std::cell::Cell;

// ============================================================================
// Cached Transform
// ============================================================================

/// A transform with cached world matrix computation.
///
/// Wraps position, rotation, and scale with a dirty flag that tracks when
/// the cached world matrix needs to be recomputed. This avoids redundant
/// matrix calculations when the transform hasn't changed.
///
/// # Performance
///
/// | Operation       | Cost                        |
/// |-----------------|----------------------------|
/// | Read (clean)    | O(1) - returns cached      |
/// | Read (dirty)    | O(1) - recomputes matrix   |
/// | Write           | O(1) - marks dirty         |
///
/// # Note
///
/// Uses interior mutability (`Cell`) for the cache so that `world_matrix()`
/// can be called with a shared reference while still updating the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransform {
    /// Position in world space
    position: Vec3,
    /// Rotation as a quaternion
    rotation: Quat,
    /// Scale factor
    scale: Vec3,

    /// Cached world matrix (computed lazily)
    #[serde(skip)]
    cached_matrix: Cell<Mat4>,

    /// Whether the cache is valid
    #[serde(skip)]
    dirty: Cell<bool>,
}

impl CachedTransform {
    /// Create a new cached transform at the origin.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from position.
    #[must_use]
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create from position and rotation.
    #[must_use]
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    /// Create from position, rotation, and scale.
    #[must_use]
    pub fn from_parts(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            cached_matrix: Cell::new(Mat4::IDENTITY),
            dirty: Cell::new(true),
        }
    }

    // -------------------------------------------------------------------------
    // Getters (don't invalidate cache)
    // -------------------------------------------------------------------------

    /// Get the position.
    #[must_use]
    #[inline]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Get the rotation.
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    /// Get the scale.
    #[must_use]
    #[inline]
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    // -------------------------------------------------------------------------
    // Setters (invalidate cache)
    // -------------------------------------------------------------------------

    /// Set the position.
    #[inline]
    pub fn set_position(&mut self, position: Vec3) {
        if self.position != position {
            self.position = position;
            self.dirty.set(true);
        }
    }

    /// Set the rotation.
    #[inline]
    pub fn set_rotation(&mut self, rotation: Quat) {
        if self.rotation != rotation {
            self.rotation = rotation;
            self.dirty.set(true);
        }
    }

    /// Set the scale.
    #[inline]
    pub fn set_scale(&mut self, scale: Vec3) {
        if self.scale != scale {
            self.scale = scale;
            self.dirty.set(true);
        }
    }

    /// Set all transform components at once.
    #[inline]
    pub fn set(&mut self, position: Vec3, rotation: Quat, scale: Vec3) {
        self.position = position;
        self.rotation = rotation;
        self.scale = scale;
        self.dirty.set(true);
    }

    // -------------------------------------------------------------------------
    // Mutations (invalidate cache)
    // -------------------------------------------------------------------------

    /// Translate by a delta.
    #[inline]
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
        self.dirty.set(true);
    }

    /// Rotate by a quaternion.
    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
        self.dirty.set(true);
    }

    /// Rotate by euler angles (in radians).
    #[inline]
    pub fn rotate_euler(&mut self, euler: Vec3) {
        let rot = Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);
        self.rotation = rot * self.rotation;
        self.dirty.set(true);
    }

    /// Scale by a factor.
    #[inline]
    pub fn scale_by(&mut self, factor: Vec3) {
        self.scale *= factor;
        self.dirty.set(true);
    }

    /// Scale uniformly.
    #[inline]
    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale *= factor;
        self.dirty.set(true);
    }

    // -------------------------------------------------------------------------
    // Computed Properties (use cache)
    // -------------------------------------------------------------------------

    /// Get the world matrix, computing if dirty.
    ///
    /// This is the main benefit of caching - repeated calls return
    /// the cached value without recomputation.
    #[must_use]
    pub fn world_matrix(&self) -> Mat4 {
        if self.dirty.get() {
            let matrix =
                Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
            self.cached_matrix.set(matrix);
            self.dirty.set(false);
        }
        self.cached_matrix.get()
    }

    /// Get the forward direction (negative Z in local space).
    #[must_use]
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction (positive X in local space).
    #[must_use]
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y in local space).
    #[must_use]
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    // -------------------------------------------------------------------------
    // Cache State
    // -------------------------------------------------------------------------

    /// Check if the cache is dirty (needs recomputation).
    #[must_use]
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    /// Manually mark the cache as dirty.
    ///
    /// Useful when external factors require recalculation (e.g., parent changed).
    #[inline]
    pub fn mark_dirty(&self) {
        self.dirty.set(true);
    }

    /// Manually mark the cache as clean.
    ///
    /// Use with caution - only when you know the cached value is correct.
    #[inline]
    pub fn mark_clean(&self) {
        self.dirty.set(false);
    }
}

impl Default for CachedTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            cached_matrix: Cell::new(Mat4::IDENTITY),
            dirty: Cell::new(true),
        }
    }
}

// ============================================================================
// Conversions
// ============================================================================

impl From<super::components::Transform> for CachedTransform {
    fn from(t: super::components::Transform) -> Self {
        Self::from_parts(t.position, t.rotation, t.scale)
    }
}

impl From<CachedTransform> for super::components::Transform {
    fn from(ct: CachedTransform) -> Self {
        Self {
            position: ct.position,
            rotation: ct.rotation,
            scale: ct.scale,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_transform_default() {
        let ct = CachedTransform::new();

        assert_eq!(ct.position(), Vec3::ZERO);
        assert_eq!(ct.rotation(), Quat::IDENTITY);
        assert_eq!(ct.scale(), Vec3::ONE);
        assert!(ct.is_dirty());
    }

    #[test]
    fn test_cached_transform_world_matrix() {
        let ct = CachedTransform::from_position(Vec3::new(1.0, 2.0, 3.0));

        let matrix = ct.world_matrix();

        // Matrix should contain the translation
        assert_eq!(matrix.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));

        // Should no longer be dirty
        assert!(!ct.is_dirty());
    }

    #[test]
    fn test_cached_transform_cache_invalidation() {
        let mut ct = CachedTransform::new();

        // Compute initial matrix
        let _ = ct.world_matrix();
        assert!(!ct.is_dirty());

        // Set position - should invalidate cache
        ct.set_position(Vec3::new(5.0, 0.0, 0.0));
        assert!(ct.is_dirty());

        // Get matrix - should recompute
        let matrix = ct.world_matrix();
        assert_eq!(matrix.w_axis.truncate(), Vec3::new(5.0, 0.0, 0.0));
        assert!(!ct.is_dirty());
    }

    #[test]
    fn test_cached_transform_no_change_no_invalidate() {
        let mut ct = CachedTransform::from_position(Vec3::new(1.0, 2.0, 3.0));

        // Compute matrix
        let _ = ct.world_matrix();
        assert!(!ct.is_dirty());

        // Set same position - should NOT invalidate
        ct.set_position(Vec3::new(1.0, 2.0, 3.0));
        assert!(!ct.is_dirty());
    }

    #[test]
    fn test_cached_transform_translate() {
        let mut ct = CachedTransform::new();
        let _ = ct.world_matrix();

        ct.translate(Vec3::new(1.0, 0.0, 0.0));

        assert!(ct.is_dirty());
        assert_eq!(ct.position(), Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_cached_transform_rotate() {
        let mut ct = CachedTransform::new();
        let _ = ct.world_matrix();

        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        ct.rotate(rotation);

        assert!(ct.is_dirty());
    }

    #[test]
    fn test_cached_transform_scale() {
        let mut ct = CachedTransform::new();
        let _ = ct.world_matrix();

        ct.scale_uniform(2.0);

        assert!(ct.is_dirty());
        assert_eq!(ct.scale(), Vec3::splat(2.0));
    }

    #[test]
    fn test_cached_transform_directions() {
        let ct = CachedTransform::new();

        // Default orientation: forward is -Z, right is X, up is Y
        assert!((ct.forward() - Vec3::NEG_Z).length() < 0.001);
        assert!((ct.right() - Vec3::X).length() < 0.001);
        assert!((ct.up() - Vec3::Y).length() < 0.001);
    }

    #[test]
    fn test_cached_transform_conversion() {
        let original =
            super::super::components::Transform::from_position(Vec3::new(5.0, 10.0, 15.0));

        // Convert to cached
        let cached = CachedTransform::from(original);
        assert_eq!(cached.position(), Vec3::new(5.0, 10.0, 15.0));

        // Convert back
        let restored: super::super::components::Transform = cached.into();
        assert_eq!(restored.position, Vec3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_cached_transform_manual_dirty() {
        let ct = CachedTransform::new();

        // Compute to clean
        let _ = ct.world_matrix();
        assert!(!ct.is_dirty());

        // Manual dirty
        ct.mark_dirty();
        assert!(ct.is_dirty());

        // Recompute
        let _ = ct.world_matrix();
        assert!(!ct.is_dirty());
    }
}
