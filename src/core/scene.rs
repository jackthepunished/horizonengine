//! Scene serialization and deserialization
//!
//! Supports saving and loading scenes in RON (Rusty Object Notation) format.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ecs::{Transform, Velocity};

/// A serializable entity with its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// Optional entity name
    pub name: Option<String>,
    /// Transform component
    pub transform: Option<Transform>,
    /// Velocity component
    pub velocity: Option<Velocity>,
    /// Parent entity index (if any)
    pub parent_index: Option<usize>,
    /// Child entity indices
    pub children_indices: Vec<usize>,
    /// Custom data as key-value pairs
    #[serde(default)]
    pub custom_data: std::collections::HashMap<String, String>,
}

impl Default for SerializedEntity {
    fn default() -> Self {
        Self {
            name: None,
            transform: Some(Transform::default()),
            velocity: None,
            parent_index: None,
            children_indices: Vec::new(),
            custom_data: std::collections::HashMap::new(),
        }
    }
}

/// A serializable scene containing multiple entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Scene name
    pub name: String,
    /// Scene version for compatibility
    pub version: u32,
    /// All entities in the scene
    pub entities: Vec<SerializedEntity>,
}

impl Scene {
    /// Create a new empty scene
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            entities: Vec::new(),
        }
    }

    /// Add an entity to the scene
    pub fn add_entity(&mut self, entity: SerializedEntity) -> usize {
        let index = self.entities.len();
        self.entities.push(entity);
        index
    }

    /// Save the scene to a RON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or serialization fails
    pub fn save_ron(&self, path: impl AsRef<Path>) -> Result<(), SceneError> {
        let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| SceneError::SerializeError(e.to_string()))?;
        fs::write(path, ron_string).map_err(|e| SceneError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Load a scene from a RON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or deserialization fails
    pub fn load_ron(path: impl AsRef<Path>) -> Result<Self, SceneError> {
        let content = fs::read_to_string(path).map_err(|e| SceneError::IoError(e.to_string()))?;
        let scene: Scene =
            ron::from_str(&content).map_err(|e| SceneError::DeserializeError(e.to_string()))?;
        Ok(scene)
    }

    /// Save the scene to a JSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or serialization fails
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), SceneError> {
        let json_string = serde_json::to_string_pretty(self)
            .map_err(|e| SceneError::SerializeError(e.to_string()))?;
        fs::write(path, json_string).map_err(|e| SceneError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Load a scene from a JSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or deserialization fails
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, SceneError> {
        let content = fs::read_to_string(path).map_err(|e| SceneError::IoError(e.to_string()))?;
        let scene: Scene = serde_json::from_str(&content)
            .map_err(|e| SceneError::DeserializeError(e.to_string()))?;
        Ok(scene)
    }

    /// Get the number of entities
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Check if the scene is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

/// Errors that can occur during scene operations
#[derive(Debug, Clone)]
pub enum SceneError {
    /// IO error
    IoError(String),
    /// Serialization error
    SerializeError(String),
    /// Deserialization error
    DeserializeError(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::SerializeError(e) => write!(f, "Serialization error: {e}"),
            Self::DeserializeError(e) => write!(f, "Deserialization error: {e}"),
        }
    }
}

impl std::error::Error for SceneError {}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_scene_serialization_ron() {
        let mut scene = Scene::new("Test Scene");

        let entity = SerializedEntity {
            name: Some("Player".to_string()),
            transform: Some(Transform::from_position(Vec3::new(1.0, 2.0, 3.0))),
            ..Default::default()
        };

        scene.add_entity(entity);

        // Serialize to RON string
        let ron_str =
            ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron_str.contains("Player"));

        // Deserialize back
        let loaded: Scene = ron::from_str(&ron_str).unwrap();
        assert_eq!(loaded.name, "Test Scene");
        assert_eq!(loaded.entities.len(), 1);
        assert_eq!(loaded.entities[0].name, Some("Player".to_string()));
    }

    #[test]
    fn test_scene_serialization_json() {
        let mut scene = Scene::new("JSON Test");

        let entity = SerializedEntity {
            name: Some("Enemy".to_string()),
            transform: Some(Transform::default()),
            velocity: Some(Velocity {
                linear: Vec3::X,
                angular: Vec3::ZERO,
            }),
            ..Default::default()
        };

        scene.add_entity(entity);

        // Serialize to JSON
        let json_str = serde_json::to_string(&scene).unwrap();

        // Deserialize back
        let loaded: Scene = serde_json::from_str(&json_str).unwrap();
        assert_eq!(loaded.name, "JSON Test");
        assert!(loaded.entities[0].velocity.is_some());
    }
}
