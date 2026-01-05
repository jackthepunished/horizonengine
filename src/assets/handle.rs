//! Asset handle implementation
//!
//! Provides type-safe handles for referencing assets without owning them.

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

/// Global counter for generating unique asset IDs
static NEXT_ASSET_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique asset ID
fn next_id() -> u64 {
    NEXT_ASSET_ID.fetch_add(1, Ordering::Relaxed)
}

/// A strong handle to an asset of type `T`.
///
/// Assets are kept alive as long as at least one `AssetHandle` exists.
/// When all handles are dropped, the asset becomes eligible for cleanup.
#[derive(Debug)]
pub struct AssetHandle<T> {
    /// Unique identifier for this asset
    id: u64,
    /// Reference-counted pointer to the asset
    inner: Arc<T>,
}

impl<T> AssetHandle<T> {
    /// Create a new asset handle wrapping the given value
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            id: next_id(),
            inner: Arc::new(value),
        }
    }

    /// Get the unique ID of this asset
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Get a reference to the underlying asset
    #[must_use]
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Create a weak handle that doesn't keep the asset alive
    #[must_use]
    pub fn downgrade(&self) -> WeakAssetHandle<T> {
        WeakAssetHandle {
            id: self.id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Get the strong reference count
    #[must_use]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get the weak reference count
    #[must_use]
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.inner)
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> std::ops::Deref for AssetHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A weak handle to an asset that doesn't prevent cleanup.
///
/// Use `upgrade()` to attempt to get a strong handle.
#[derive(Debug)]
pub struct WeakAssetHandle<T> {
    /// Unique identifier for this asset
    id: u64,
    /// Weak reference to the asset
    inner: Weak<T>,
}

impl<T> WeakAssetHandle<T> {
    /// Get the unique ID of this asset
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Attempt to upgrade to a strong handle.
    ///
    /// Returns `None` if the asset has been dropped.
    #[must_use]
    pub fn upgrade(&self) -> Option<AssetHandle<T>> {
        self.inner
            .upgrade()
            .map(|inner| AssetHandle { id: self.id, inner })
    }

    /// Check if the asset is still alive
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.inner.strong_count() > 0
    }
}

impl<T> Clone for WeakAssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: Weak::clone(&self.inner),
        }
    }
}

impl<T> PartialEq for WeakAssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for WeakAssetHandle<T> {}

impl<T> Hash for WeakAssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_creation() {
        let handle = AssetHandle::new(42_i32);
        assert_eq!(*handle.get(), 42);
    }

    #[test]
    fn test_handle_clone() {
        let handle1 = AssetHandle::new("test".to_string());
        let handle2 = handle1.clone();
        assert_eq!(handle1.id(), handle2.id());
        assert_eq!(handle1.strong_count(), 2);
    }

    #[test]
    fn test_weak_upgrade() {
        let strong = AssetHandle::new(100_u32);
        let weak = strong.downgrade();

        assert!(weak.is_alive());
        let upgraded = weak.upgrade();
        assert!(upgraded.is_some());

        drop(strong);
        drop(upgraded);
        assert!(!weak.is_alive());
        assert!(weak.upgrade().is_none());
    }
}
