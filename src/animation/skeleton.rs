//! Skeleton and bone system for skeletal animation
//!
//! Provides bone hierarchy and skinning data for GPU.

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A single bone in a skeleton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    /// Bone name
    pub name: String,
    /// Parent bone index (None for root)
    pub parent: Option<usize>,
    /// Children bone indices
    pub children: Vec<usize>,
    /// Local translation
    pub translation: Vec3,
    /// Local rotation
    pub rotation: Quat,
    /// Local scale
    pub scale: Vec3,
    /// Inverse bind matrix (for skinning)
    pub inverse_bind_matrix: Mat4,
}

impl Bone {
    /// Create a new bone
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            children: Vec::new(),
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            inverse_bind_matrix: Mat4::IDENTITY,
        }
    }

    /// Get the local transform matrix
    #[must_use]
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Bone {
    fn default() -> Self {
        Self::new("bone")
    }
}

/// A skeleton containing a hierarchy of bones
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Skeleton {
    /// All bones in the skeleton
    pub bones: Vec<Bone>,
    /// Root bone indices
    pub roots: Vec<usize>,
}

impl Skeleton {
    /// Create a new empty skeleton
    #[must_use]
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            roots: Vec::new(),
        }
    }

    /// Add a bone and return its index
    pub fn add_bone(&mut self, bone: Bone) -> usize {
        let index = self.bones.len();
        if bone.parent.is_none() {
            self.roots.push(index);
        }
        self.bones.push(bone);
        index
    }

    /// Set parent-child relationship
    pub fn set_parent(&mut self, child: usize, parent: usize) {
        if child == parent || child >= self.bones.len() || parent >= self.bones.len() {
            return;
        }

        // Check for cycles (walk up hierarchy)
        let mut curr = parent;
        while let Some(p) = self.bones[curr].parent {
            if p == child {
                return;
            }
            curr = p;
        }

        // Remove from old parent if needed
        if let Some(old_parent) = self.bones[child].parent {
            if old_parent != parent {
                if let Some(old) = self.bones.get_mut(old_parent) {
                    old.children.retain(|&c| c != child);
                }
            } else {
                return;
            }
        } else {
            // Was root
            self.roots.retain(|&r| r != child);
        }

        self.bones[child].parent = Some(parent);
        if !self.bones[parent].children.contains(&child) {
            self.bones[parent].children.push(child);
        }
    }

    /// Get the number of bones
    #[must_use]
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Get a bone by index
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Bone> {
        self.bones.get(index)
    }

    /// Get a mutable bone by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Bone> {
        self.bones.get_mut(index)
    }

    /// Find bone by name
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }

    /// Compute world matrices for all bones using hierarchy traversal
    ///
    /// This ensures correct calculation regardless of bone storage order.
    #[must_use]
    pub fn compute_world_matrices(&self) -> Vec<Mat4> {
        let mut world_matrices = vec![Mat4::IDENTITY; self.bones.len()];
        let mut stack = Vec::with_capacity(self.bones.len());

        // Start with roots
        for &root in &self.roots {
            stack.push((root, Mat4::IDENTITY));
        }

        // Traverse hierarchy
        while let Some((index, parent_world)) = stack.pop() {
            let bone = &self.bones[index];
            let local = bone.local_matrix();
            let world = parent_world * local;
            world_matrices[index] = world;

            // Add children to stack
            for &child in &bone.children {
                stack.push((child, world));
            }
        }

        world_matrices
    }

    /// Compute skinning matrices (joint matrices)
    #[must_use]
    pub fn compute_skinning_matrices(&self) -> Vec<Mat4> {
        let world_matrices = self.compute_world_matrices();

        self.bones
            .iter()
            .zip(world_matrices.iter())
            .map(|(bone, world)| *world * bone.inverse_bind_matrix)
            .collect()
    }
}

/// Skinning data for GPU upload
#[derive(Debug, Clone)]
pub struct SkinningData {
    /// Joint matrices (one per bone)
    pub joint_matrices: Vec<Mat4>,
}

impl SkinningData {
    /// Create from skeleton
    #[must_use]
    pub fn from_skeleton(skeleton: &Skeleton) -> Self {
        Self {
            joint_matrices: skeleton.compute_skinning_matrices(),
        }
    }

    /// Get the data as bytes for GPU upload
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.joint_matrices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_hierarchy() {
        let mut skeleton = Skeleton::new();

        let root = skeleton.add_bone(Bone::new("root"));
        let child = skeleton.add_bone(Bone::new("child"));

        skeleton.set_parent(child, root);

        assert_eq!(skeleton.bone_count(), 2);
        assert_eq!(skeleton.roots.len(), 1);
        assert_eq!(skeleton.bones[child].parent, Some(root));
        assert!(skeleton.bones[root].children.contains(&child));
    }

    #[test]
    fn test_world_matrix_computation() {
        let mut skeleton = Skeleton::new();

        let mut root = Bone::new("root");
        root.translation = Vec3::new(1.0, 0.0, 0.0);
        skeleton.add_bone(root);

        let mut child = Bone::new("child");
        child.translation = Vec3::new(2.0, 0.0, 0.0);
        let child_idx = skeleton.add_bone(child);
        skeleton.set_parent(child_idx, 0);

        let world_matrices = skeleton.compute_world_matrices();

        // Child world position should be parent + child local
        let child_pos = world_matrices[child_idx].w_axis.truncate();
        assert!((child_pos.x - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_find_by_name() {
        let mut skeleton = Skeleton::new();
        skeleton.add_bone(Bone::new("hip"));
        skeleton.add_bone(Bone::new("spine"));
        skeleton.add_bone(Bone::new("head"));

        assert_eq!(skeleton.find_by_name("spine"), Some(1));
        assert_eq!(skeleton.find_by_name("head"), Some(2));
        assert_eq!(skeleton.find_by_name("missing"), None);
    }

    #[test]
    fn test_out_of_order_bones() {
        let mut skeleton = Skeleton::new();

        // Add child first (index 0)
        let child = Bone::new("child");
        let child_idx = skeleton.add_bone(child);

        // Add parent second (index 1)
        let mut parent = Bone::new("parent");
        parent.translation = Vec3::new(10.0, 0.0, 0.0);
        let parent_idx = skeleton.add_bone(parent);

        // Set parent: Child -> Parent
        // This makes Child (0) depend on Parent (1)
        skeleton.set_parent(child_idx, parent_idx);

        let world_matrices = skeleton.compute_world_matrices();
        let child_pos = world_matrices[child_idx].w_axis.truncate();

        // Should be at 10.0 (Parent pos) + 0.0 (Child local)
        assert!((child_pos.x - 10.0).abs() < 0.001);
    }
}
