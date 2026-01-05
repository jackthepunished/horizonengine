//! Entity hierarchy components
//!
//! Provides parent-child relationships between entities for transform propagation.

use glam::{Mat4, Quat, Vec3};
use hecs::Entity;
use smallvec::SmallVec;

/// Parent component - indicates this entity has a parent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Parent {
    /// Create a new parent reference
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Get the parent entity
    #[must_use]
    pub const fn entity(&self) -> Entity {
        self.0
    }
}

/// Children component - tracks all children of this entity
#[derive(Debug, Clone, Default)]
pub struct Children(pub SmallVec<[Entity; 8]>);

impl Children {
    /// Create an empty children list
    #[must_use]
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Create from a single child
    #[must_use]
    pub fn single(child: Entity) -> Self {
        let mut children = SmallVec::new();
        children.push(child);
        Self(children)
    }

    /// Add a child
    pub fn add(&mut self, child: Entity) {
        if !self.0.contains(&child) {
            self.0.push(child);
        }
    }

    /// Remove a child
    pub fn remove(&mut self, child: Entity) -> bool {
        if let Some(pos) = self.0.iter().position(|&e| e == child) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this entity has children
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of children
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterate over children
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

/// Global transform - computed world-space transform
#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform {
    /// World-space transformation matrix
    pub matrix: Mat4,
}

impl GlobalTransform {
    /// Create from a transformation matrix
    #[must_use]
    pub const fn new(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Create identity transform
    #[must_use]
    pub fn identity() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }

    /// Create from position, rotation, and scale
    #[must_use]
    pub fn from_components(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            matrix: Mat4::from_scale_rotation_translation(scale, rotation, position),
        }
    }

    /// Get world position
    #[must_use]
    pub fn position(&self) -> Vec3 {
        self.matrix.col(3).truncate()
    }

    /// Get world rotation (approximate, from matrix decomposition)
    #[must_use]
    pub fn rotation(&self) -> Quat {
        Quat::from_mat4(&self.matrix)
    }

    /// Get world scale (approximate)
    #[must_use]
    pub fn scale(&self) -> Vec3 {
        Vec3::new(
            self.matrix.col(0).truncate().length(),
            self.matrix.col(1).truncate().length(),
            self.matrix.col(2).truncate().length(),
        )
    }

    /// Transform a point from local to world space
    #[must_use]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix.transform_point3(point)
    }

    /// Transform a direction vector (ignores translation)
    #[must_use]
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.matrix.transform_vector3(direction)
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_children_add_remove() {
        let mut world = hecs::World::new();
        let entity1 = world.spawn(());
        let entity2 = world.spawn(());

        let mut children = Children::new();

        children.add(entity1);
        children.add(entity2);
        assert_eq!(children.len(), 2);

        // No duplicates
        children.add(entity1);
        assert_eq!(children.len(), 2);

        children.remove(entity1);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_global_transform() {
        let transform =
            GlobalTransform::from_components(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);

        let pos = transform.position();
        assert!((pos.x - 1.0).abs() < 0.001);
        assert!((pos.y - 2.0).abs() < 0.001);
        assert!((pos.z - 3.0).abs() < 0.001);
    }
}
