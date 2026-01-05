//! Asset and resource management system
//!
//! Provides handle-based resource management with:
//! - Type-safe asset handles
//! - Centralized asset storage
//! - Reference counting for automatic cleanup

mod handle;
mod storage;

pub use handle::{AssetHandle, WeakAssetHandle};
pub use storage::{AssetServer, Assets};
