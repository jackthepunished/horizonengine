//! Asset storage and management
//!
//! Provides centralized storage for assets with path-based lookup.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::handle::AssetHandle;

/// Type-erased asset entry
struct AssetEntry {
    /// The asset data (type-erased)
    data: Box<dyn Any + Send + Sync>,
    /// Original path this asset was loaded from
    path: Option<PathBuf>,
}

/// Centralized storage for all assets of a specific type
pub struct Assets<T: Send + Sync + 'static> {
    /// Assets indexed by their handle ID
    assets: HashMap<u64, AssetEntry>,
    /// Path to handle ID mapping for deduplication
    path_to_id: HashMap<PathBuf, u64>,
    /// Phantom data for type safety
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> Assets<T> {
    /// Create a new empty asset storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_to_id: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Add an asset and return a handle to it
    pub fn add(&mut self, asset: T) -> AssetHandle<T> {
        let handle = AssetHandle::new(asset);
        let id = handle.id();

        // Store a clone of the Arc's content
        self.assets.insert(
            id,
            AssetEntry {
                data: Box::new(handle.clone()),
                path: None,
            },
        );

        handle
    }

    /// Add an asset with an associated path for deduplication
    pub fn add_with_path(&mut self, asset: T, path: impl AsRef<Path>) -> AssetHandle<T> {
        let path = path.as_ref().to_path_buf();

        // Check if already loaded
        if let Some(&id) = self.path_to_id.get(&path)
            && let Some(entry) = self.assets.get(&id)
            && let Some(handle) = entry.data.downcast_ref::<AssetHandle<T>>()
        {
            return handle.clone();
        }

        let handle = AssetHandle::new(asset);
        let id = handle.id();

        self.path_to_id.insert(path.clone(), id);
        self.assets.insert(
            id,
            AssetEntry {
                data: Box::new(handle.clone()),
                path: Some(path),
            },
        );

        handle
    }

    /// Get an asset by its handle ID
    #[must_use]
    pub fn get(&self, id: u64) -> Option<AssetHandle<T>> {
        self.assets
            .get(&id)
            .and_then(|entry| entry.data.downcast_ref::<AssetHandle<T>>())
            .cloned()
    }

    /// Get an asset by its path
    #[must_use]
    pub fn get_by_path(&self, path: impl AsRef<Path>) -> Option<AssetHandle<T>> {
        let path = path.as_ref();
        self.path_to_id.get(path).and_then(|&id| self.get(id))
    }

    /// Check if an asset exists by path
    #[must_use]
    pub fn contains_path(&self, path: impl AsRef<Path>) -> bool {
        self.path_to_id.contains_key(path.as_ref())
    }

    /// Remove an asset by ID
    ///
    /// Returns true if the asset was removed
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(entry) = self.assets.remove(&id) {
            if let Some(path) = entry.path {
                self.path_to_id.remove(&path);
            }
            true
        } else {
            false
        }
    }

    /// Get the number of stored assets
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Check if storage is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Clear all assets
    pub fn clear(&mut self) {
        self.assets.clear();
        self.path_to_id.clear();
    }

    /// Iterate over all asset handles
    pub fn iter(&self) -> impl Iterator<Item = AssetHandle<T>> + '_ {
        self.assets
            .values()
            .filter_map(|entry| entry.data.downcast_ref::<AssetHandle<T>>().cloned())
    }
}

impl<T: Send + Sync + 'static> Default for Assets<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Global asset server for managing all asset types
pub struct AssetServer {
    /// Type-erased storage for each asset type
    storages: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AssetServer {
    /// Create a new asset server
    #[must_use]
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Get or create storage for a specific asset type
    pub fn get_storage<T: Send + Sync + 'static>(&mut self) -> &mut Assets<T> {
        let type_id = TypeId::of::<T>();

        self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(Assets::<T>::new()))
            .downcast_mut::<Assets<T>>()
            .expect("Type mismatch in asset storage")
    }

    /// Add an asset and return a handle
    pub fn add<T: Send + Sync + 'static>(&mut self, asset: T) -> AssetHandle<T> {
        self.get_storage::<T>().add(asset)
    }

    /// Add an asset with path
    pub fn add_with_path<T: Send + Sync + 'static>(
        &mut self,
        asset: T,
        path: impl AsRef<Path>,
    ) -> AssetHandle<T> {
        self.get_storage::<T>().add_with_path(asset, path)
    }

    /// Get an asset by path
    #[must_use]
    pub fn get_by_path<T: Send + Sync + 'static>(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Option<AssetHandle<T>> {
        self.get_storage::<T>().get_by_path(path)
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut assets = Assets::<String>::new();
        let handle = assets.add("hello".to_string());

        let retrieved = assets.get(handle.id());
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap().get(), "hello");
    }

    #[test]
    fn test_path_deduplication() {
        let mut assets = Assets::<i32>::new();
        let handle1 = assets.add_with_path(42, "test/asset.txt");
        let handle2 = assets.add_with_path(100, "test/asset.txt");

        // Should return same handle (deduplication)
        assert_eq!(handle1.id(), handle2.id());
        assert_eq!(*handle1.get(), 42);
    }

    #[test]
    fn test_asset_server() {
        let mut server = AssetServer::new();

        let str_handle = server.add("test".to_string());
        let int_handle = server.add(42_i32);

        assert_eq!(*str_handle.get(), "test");
        assert_eq!(*int_handle.get(), 42);
    }
}
