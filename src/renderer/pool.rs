//! Object Pool for Zero-Allocation Reuse
//!
//! This module provides a generic object pool that eliminates allocations during
//! gameplay by pre-allocating and reusing objects. Particularly useful for
//! frequently created/destroyed objects like particles, projectiles, and audio sources.
//!
//! # Design Principles
//!
//! - **Zero Allocation**: After initial capacity is filled, no heap allocations occur
//! - **Cache Friendly**: Objects stored contiguously for good cache locality
//! - **Simple API**: Acquire, use, release - no complex lifetime management
//! - **Type Safe**: Generic over any type, with optional reset behavior
//!
//! # Example
//!
//! ```ignore
//! use crate::renderer::pool::Pool;
//!
//! // Create a pool with initial capacity
//! let mut pool: Pool<Particle> = Pool::with_capacity(1000);
//!
//! // Acquire an object (reuses existing or creates new)
//! let index = pool.acquire(Particle::default);
//!
//! // Use the object
//! if let Some(particle) = pool.get_mut(index) {
//!     particle.position = [1.0, 2.0, 3.0];
//! }
//!
//! // Release back to pool when done
//! pool.release(index);
//! ```

// ============================================================================
// Pool Index
// ============================================================================

/// Index into a pool, identifying a specific slot.
///
/// This is a simple index wrapper that ensures type safety.
/// The index remains valid until the object is released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolIndex(usize);

impl PoolIndex {
    /// Get the raw index value.
    #[must_use]
    #[inline]
    pub const fn raw(self) -> usize {
        self.0
    }
}

// ============================================================================
// Pool Slot
// ============================================================================

// ============================================================================
// Pool Slot
// ============================================================================

/// Status of a pool slot.
#[derive(Debug)]
enum SlotState {
    /// Slot is actively used
    Occupied,
    /// Slot is available for reuse, pointing to next free slot
    Vacant(usize),
}

/// Internal slot storage.
///
/// Keeps the object instance alive even when vacant, allowing for memory reuse
/// (e.g. avoiding buffer reallocations) via `acquire_with_reset`.
#[derive(Debug)]
struct Slot<T> {
    value: T,
    state: SlotState,
}

// ============================================================================
// Object Pool
// ============================================================================

/// A generic object pool for zero-allocation object reuse.
///
/// The pool maintains a free list for O(1) acquire and release operations.
/// Objects are stored contiguously for cache efficiency.
///
/// # Performance Characteristics
///
/// | Operation | Time Complexity |
/// |-----------|-----------------|
/// | `acquire` | O(1) amortized  |
/// | `release` | O(1)            |
/// | `get`     | O(1)            |
/// | `iter`    | O(n)            |
///
/// # Memory Layout
///
/// Objects are stored in a `Vec<Slot<T>>`. Unlike traditional pools that might
/// drop objects on release, this pool keeps the memory (and the `T` instance)
/// alive in "Vacant" slots. This allows `acquire_with_reset` to reuse allocated
/// resources (like vectors inside `T`) without dropping and reallocating them.
#[derive(Debug)]
pub struct Pool<T> {
    /// Storage for all slots
    slots: Vec<Slot<T>>,
    /// Head of the free list (index of first free slot, or usize::MAX if none)
    free_head: usize,
    /// Number of currently active objects
    active_count: usize,
}

impl<T> Pool<T> {
    /// Sentinel value indicating end of free list.
    const NONE: usize = usize::MAX;

    /// Create a new empty pool.
    ///
    /// The pool will allocate as objects are acquired.
    /// Use `with_capacity` to pre-allocate if you know the expected size.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: Self::NONE,
            active_count: 0,
        }
    }

    /// Create a pool with pre-allocated capacity.
    ///
    /// This is more efficient if you know the approximate number of
    /// objects you'll need, as it avoids reallocations.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: Self::NONE,
            active_count: 0,
        }
    }

    /// Acquire an object from the pool.
    ///
    /// If a free slot exists, it will be reused. The existing object in that slot
    /// is **replaced** (overwritten) by the new one created by `init`.
    ///
    /// If you want to reuse the internal memory of the existing object (e.g. reusing
    /// a Vec's capacity), use `acquire_with_reset` instead.
    ///
    /// # Arguments
    ///
    /// * `init` - Function to create a new object
    ///
    /// # Returns
    ///
    /// Index to the acquired object
    pub fn acquire(&mut self, init: impl FnOnce() -> T) -> PoolIndex {
        self.active_count += 1;

        if self.free_head != Self::NONE {
            // Reuse a free slot
            let index = self.free_head;
            let slot = &mut self.slots[index];

            // Update free list head
            if let SlotState::Vacant(next) = slot.state {
                self.free_head = next;
            } else {
                // Should be unreachable if logic is correct
                self.free_head = Self::NONE;
            }

            // Overwrite object and mark active
            slot.value = init();
            slot.state = SlotState::Occupied;

            PoolIndex(index)
        } else {
            // Allocate new slot
            let index = self.slots.len();
            self.slots.push(Slot {
                value: init(),
                state: SlotState::Occupied,
            });
            PoolIndex(index)
        }
    }

    /// Acquire an object, reusing the existing one if available.
    ///
    /// This variant resets the existing object in a free slot using `reset`.
    /// If no free slot exists, `init` is called to create a new one.
    ///
    /// # Arguments
    ///
    /// * `init` - Create brand new object (if pool needs to grow)
    /// * `reset` - Reset existing object (if reusing a slot)
    ///
    /// # Returns
    ///
    /// Index to the acquired object
    pub fn acquire_with_reset(
        &mut self,
        init: impl FnOnce() -> T,
        reset: impl FnOnce(&mut T),
    ) -> PoolIndex {
        self.active_count += 1;

        if self.free_head != Self::NONE {
            // Reuse existing slot
            let index = self.free_head;
            let slot = &mut self.slots[index];

            // Update free list
            if let SlotState::Vacant(next) = slot.state {
                self.free_head = next;
            } else {
                self.free_head = Self::NONE;
            }

            // Reset existing object and mark active
            reset(&mut slot.value);
            slot.state = SlotState::Occupied;

            PoolIndex(index)
        } else {
            // Grow pool
            let index = self.slots.len();
            self.slots.push(Slot {
                value: init(),
                state: SlotState::Occupied,
            });
            PoolIndex(index)
        }
    }

    /// Release an object back to the pool.
    ///
    /// The slot becomes available for future `acquire` calls.
    /// The object data is **preserved** in the slot (not dropped), allowing
    /// future reuse via `acquire_with_reset`.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the object to release
    ///
    /// # Returns
    ///
    /// `true` if the object was released, `false` if the index was invalid
    pub fn release(&mut self, index: PoolIndex) -> bool {
        let idx = index.0;

        if idx >= self.slots.len() {
            return false;
        }

        let slot = &mut self.slots[idx];

        // Check if already vacant
        if let SlotState::Vacant(_) = slot.state {
            return false;
        }

        // Add to free list
        slot.state = SlotState::Vacant(self.free_head);
        self.free_head = idx;
        self.active_count -= 1;

        true
    }

    /// Get a reference to an object by index.
    ///
    /// Returns `None` if the index is invalid or the slot is vacant.
    #[must_use]
    #[inline]
    pub fn get(&self, index: PoolIndex) -> Option<&T> {
        self.slots.get(index.0).and_then(|slot| match slot.state {
            SlotState::Occupied => Some(&slot.value),
            SlotState::Vacant(_) => None,
        })
    }

    /// Get a mutable reference to an object by index.
    ///
    /// Returns `None` if the index is invalid or the slot is vacant.
    #[inline]
    pub fn get_mut(&mut self, index: PoolIndex) -> Option<&mut T> {
        self.slots
            .get_mut(index.0)
            .and_then(|slot| match slot.state {
                SlotState::Occupied => Some(&mut slot.value),
                SlotState::Vacant(_) => None,
            })
    }

    /// Check if an index refers to an active object.
    #[must_use]
    #[inline]
    pub fn is_active(&self, index: PoolIndex) -> bool {
        self.slots
            .get(index.0)
            .is_some_and(|slot| matches!(slot.state, SlotState::Occupied))
    }

    /// Get the number of currently active objects.
    #[must_use]
    #[inline]
    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    /// Get the total capacity (active + free slots).
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Check if the pool has no active objects.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    /// Iterate over all active objects.
    ///
    /// The iterator yields references to active objects only.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(|slot| match slot.state {
            SlotState::Occupied => Some(&slot.value),
            SlotState::Vacant(_) => None,
        })
    }

    /// Iterate mutably over all active objects.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|slot| match slot.state {
            SlotState::Occupied => Some(&mut slot.value),
            SlotState::Vacant(_) => None,
        })
    }

    /// Iterate over active objects with their indices.
    ///
    /// Useful when you need to potentially release objects during iteration.
    pub fn iter_with_index(&self) -> impl Iterator<Item = (PoolIndex, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| match slot.state {
                SlotState::Occupied => Some((PoolIndex(idx), &slot.value)),
                SlotState::Vacant(_) => None,
            })
    }

    /// Clear all objects from the pool.
    ///
    /// This drops all objects and resets the pool to empty state.
    /// Allocated memory is retained for future use.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_head = Self::NONE;
        self.active_count = 0;
    }

    /// Collect active objects into a contiguous slice for GPU upload.
    ///
    /// This is useful when you need to pass pool data to a GPU buffer.
    /// Returns a reference to the internal staging buffer.
    ///
    /// # Note
    ///
    /// This method requires `T: Copy` to efficiently collect objects.
    pub fn collect_active(&self, buffer: &mut Vec<T>)
    where
        T: Copy,
    {
        buffer.clear();
        buffer.extend(self.iter().copied());
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestObject {
        value: i32,
    }

    impl TestObject {
        fn new(value: i32) -> Self {
            Self { value }
        }
    }

    #[test]
    fn test_pool_acquire_and_release() {
        let mut pool: Pool<TestObject> = Pool::new();

        // Acquire some objects
        let idx1 = pool.acquire(|| TestObject::new(1));
        let idx2 = pool.acquire(|| TestObject::new(2));
        let idx3 = pool.acquire(|| TestObject::new(3));

        assert_eq!(pool.active_count(), 3);
        assert_eq!(pool.get(idx1).unwrap().value, 1);
        assert_eq!(pool.get(idx2).unwrap().value, 2);
        assert_eq!(pool.get(idx3).unwrap().value, 3);

        // Release middle object
        assert!(pool.release(idx2));
        assert_eq!(pool.active_count(), 2);
        assert!(pool.get(idx2).is_none());

        // idx1 and idx3 should still be valid
        assert_eq!(pool.get(idx1).unwrap().value, 1);
        assert_eq!(pool.get(idx3).unwrap().value, 3);
    }

    #[test]
    fn test_pool_reuse_slot() {
        let mut pool: Pool<TestObject> = Pool::new();

        // Acquire and release
        let idx1 = pool.acquire(|| TestObject::new(100));
        pool.release(idx1);

        // Acquire again - should reuse the same slot
        let idx2 = pool.acquire(|| TestObject::new(200));
        assert_eq!(idx1.raw(), idx2.raw(), "Should reuse the same slot");
        assert_eq!(pool.get(idx2).unwrap().value, 200);
    }

    #[test]
    fn test_pool_free_list_order() {
        let mut pool: Pool<TestObject> = Pool::new();

        // Acquire 3 objects
        let idx0 = pool.acquire(|| TestObject::new(0));
        let idx1 = pool.acquire(|| TestObject::new(1));
        let idx2 = pool.acquire(|| TestObject::new(2));

        // Release in order: 1, 0, 2
        pool.release(idx1);
        pool.release(idx0);
        pool.release(idx2);

        // Acquire should get them in LIFO order: 2, 0, 1
        let new_idx1 = pool.acquire(|| TestObject::new(10));
        let new_idx2 = pool.acquire(|| TestObject::new(20));
        let new_idx3 = pool.acquire(|| TestObject::new(30));

        assert_eq!(new_idx1.raw(), 2);
        assert_eq!(new_idx2.raw(), 0);
        assert_eq!(new_idx3.raw(), 1);
    }

    #[test]
    fn test_pool_iteration() {
        let mut pool: Pool<TestObject> = Pool::new();

        pool.acquire(|| TestObject::new(1));
        let idx2 = pool.acquire(|| TestObject::new(2));
        pool.acquire(|| TestObject::new(3));

        // Release middle
        pool.release(idx2);

        // Iteration should skip the vacant slot
        let values: Vec<i32> = pool.iter().map(|obj| obj.value).collect();
        assert_eq!(values, vec![1, 3]);
    }

    #[test]
    fn test_pool_get_mut() {
        let mut pool: Pool<TestObject> = Pool::new();

        let idx = pool.acquire(|| TestObject::new(42));

        // Modify via get_mut
        if let Some(obj) = pool.get_mut(idx) {
            obj.value = 100;
        }

        assert_eq!(pool.get(idx).unwrap().value, 100);
    }

    #[test]
    fn test_pool_double_release() {
        let mut pool: Pool<TestObject> = Pool::new();

        let idx = pool.acquire(|| TestObject::new(1));
        assert!(pool.release(idx));
        assert!(!pool.release(idx), "Double release should return false");
    }

    #[test]
    fn test_pool_invalid_index() {
        let mut pool: Pool<TestObject> = Pool::new();

        let invalid = PoolIndex(999);
        assert!(pool.get(invalid).is_none());
        assert!(!pool.release(invalid));
    }

    #[test]
    fn test_pool_clear() {
        let mut pool: Pool<TestObject> = Pool::new();

        pool.acquire(|| TestObject::new(1));
        pool.acquire(|| TestObject::new(2));
        pool.acquire(|| TestObject::new(3));

        pool.clear();

        assert_eq!(pool.active_count(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_collect_active() {
        let mut pool: Pool<i32> = Pool::new();

        pool.acquire(|| 10);
        let idx2 = pool.acquire(|| 20);
        pool.acquire(|| 30);

        pool.release(idx2);

        let mut buffer = Vec::new();
        pool.collect_active(&mut buffer);

        assert_eq!(buffer, vec![10, 30]);
    }

    #[test]
    fn test_pool_with_capacity() {
        let pool: Pool<TestObject> = Pool::with_capacity(100);

        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.capacity(), 0); // No slots allocated yet, just reserved
    }

    #[test]
    fn test_pool_is_active() {
        let mut pool: Pool<TestObject> = Pool::new();

        let idx = pool.acquire(|| TestObject::new(1));
        assert!(pool.is_active(idx));

        pool.release(idx);
        assert!(!pool.is_active(idx));
    }

    #[test]
    fn test_pool_acquire_with_reset_reuse() {
        let mut pool: Pool<TestObject> = Pool::new();
        let idx1 = pool.acquire(|| TestObject::new(10));
        pool.release(idx1);

        // Should reuse the slot and NOT call init
        let idx2 = pool.acquire_with_reset(
            || panic!("Should not call init when reusing!"),
            |obj| {
                // Verify the old value is still there (memory persisted)
                assert_eq!(obj.value, 10);
                obj.value = 20;
            },
        );

        // Index should be reused (LIFO)
        assert_eq!(idx1, idx2);
        assert_eq!(pool.get(idx2).unwrap().value, 20);
    }
}
