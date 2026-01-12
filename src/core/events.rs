//! Event Queue System for Decoupled Communication
//!
//! This module provides a type-safe, double-buffered event queue that enables
//! loose coupling between engine systems. Events are written during one frame
//! and processed in the next, ensuring consistent behavior.
//!
//! # Design Principles
//!
//! - **Type Safety**: All events are strongly typed via the `GameEvent` enum
//! - **Double Buffering**: Events are frame-consistent (no mid-frame mutations)
//! - **Zero Allocation**: Uses pre-allocated `VecDeque` with reuse
//! - **Simplicity**: No complex pub/sub - just push and iterate
//!
//! # Example
//!
//! ```ignore
//! // In game update
//! ctx.events.push(GameEvent::EntityDamaged {
//!     entity,
//!     amount: 10.0,
//!     source: Some(attacker),
//! });
//!
//! // In audio system
//! for event in ctx.events.iter() {
//!     if let GameEvent::EntityDamaged { entity, .. } = event {
//!         play_hurt_sound(*entity);
//!     }
//! }
//! ```

use std::collections::VecDeque;

use glam::Vec3;
use hecs::Entity;

// ============================================================================
// Event Types
// ============================================================================

/// Game events for inter-system communication.
///
/// Events represent things that happened in the game world. They flow from
/// producers (gameplay systems) to consumers (audio, UI, effects) without
/// direct coupling.
///
/// # Extensibility
///
/// The `#[non_exhaustive]` attribute allows adding new variants without
/// breaking downstream code that uses wildcard patterns.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GameEvent {
    // -------------------------------------------------------------------------
    // Combat Events
    // -------------------------------------------------------------------------
    /// An entity took damage.
    EntityDamaged {
        /// The entity that was damaged
        entity: Entity,
        /// Amount of damage dealt
        amount: f32,
        /// Entity that caused the damage, if any
        source: Option<Entity>,
    },

    /// An entity was destroyed/killed.
    EntityDestroyed {
        /// The destroyed entity
        entity: Entity,
        /// Optional destroyer
        destroyer: Option<Entity>,
    },

    // -------------------------------------------------------------------------
    // Physics Events
    // -------------------------------------------------------------------------
    /// Two entities collided.
    Collision {
        /// First entity in collision
        entity_a: Entity,
        /// Second entity in collision
        entity_b: Entity,
        /// World-space contact point
        contact_point: Vec3,
        /// Contact normal (from A to B)
        normal: Vec3,
    },

    // -------------------------------------------------------------------------
    // Audio Events
    // -------------------------------------------------------------------------
    /// Request to play a sound effect.
    PlaySound {
        /// Sound asset name
        name: &'static str,
        /// Position for 3D audio (None for 2D)
        position: Option<Vec3>,
        /// Volume multiplier (0.0 to 1.0)
        volume: f32,
    },

    // -------------------------------------------------------------------------
    // UI Events
    // -------------------------------------------------------------------------
    /// A UI button was clicked.
    ButtonClicked {
        /// Button identifier
        id: &'static str,
    },

    /// A UI value changed.
    ValueChanged {
        /// Widget identifier
        id: &'static str,
        /// New value
        value: f32,
    },

    // -------------------------------------------------------------------------
    // Game State Events
    // -------------------------------------------------------------------------
    /// Player scored points.
    ScoreChanged {
        /// New score value
        score: u32,
    },

    /// Game state transition.
    StateChanged {
        /// New state name
        state: &'static str,
    },
}

// ============================================================================
// Event Queue
// ============================================================================

/// Double-buffered event queue for frame-consistent event processing.
///
/// Events pushed during frame N are available for reading during frame N+1.
/// This prevents issues where event order depends on system update order.
///
/// # Performance
///
/// - Push: O(1) amortized
/// - Iteration: O(n)
/// - Swap: O(1)
///
/// # Example
///
/// ```ignore
/// let mut queue = EventQueue::new();
///
/// // Frame N: Push events
/// queue.push(GameEvent::ScoreChanged { score: 100 });
///
/// // Frame N+1: Process events (after swap)
/// queue.swap();
/// for event in queue.iter() {
///     handle_event(event);
/// }
/// ```
#[derive(Debug)]
pub struct EventQueue {
    /// Events being written this frame
    pending: VecDeque<GameEvent>,
    /// Events from previous frame, ready for processing
    processing: VecDeque<GameEvent>,
}

impl EventQueue {
    /// Default initial capacity for event queues.
    const DEFAULT_CAPACITY: usize = 64;

    /// Create a new event queue with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Create a new event queue with specified initial capacity.
    ///
    /// Use a larger capacity if you expect many events per frame to
    /// avoid reallocations.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pending: VecDeque::with_capacity(capacity),
            processing: VecDeque::with_capacity(capacity),
        }
    }

    /// Push an event to be processed next frame.
    ///
    /// Events are not immediately visible to iterators. Call `swap()`
    /// at the frame boundary to make them available.
    #[inline]
    pub fn push(&mut self, event: GameEvent) {
        self.pending.push_back(event);
    }

    /// Swap the pending and processing queues.
    ///
    /// Call this once per frame, typically at the start of the update loop.
    /// After swapping:
    /// - `iter()` returns events from the previous frame
    /// - `push()` writes to the new pending queue
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.pending, &mut self.processing);
        self.pending.clear();
    }

    /// Iterate over events from the previous frame.
    ///
    /// Returns an iterator over references to events. The events remain
    /// in the queue until the next `swap()` call.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &GameEvent> {
        self.processing.iter()
    }

    /// Drain all events from the previous frame.
    ///
    /// Similar to `iter()` but takes ownership of the events.
    /// Useful when you need to move events elsewhere.
    #[inline]
    pub fn drain(&mut self) -> impl Iterator<Item = GameEvent> + '_ {
        self.processing.drain(..)
    }

    /// Check if there are any pending events to process.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.processing.is_empty()
    }

    /// Get the number of events ready for processing.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.processing.len()
    }

    /// Get the number of events pending for next frame.
    #[must_use]
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Clear all events (both pending and processing).
    ///
    /// Useful for scene transitions or resetting game state.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.processing.clear();
    }
}

impl Default for EventQueue {
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

    /// Helper to create a test entity
    fn test_entity() -> Entity {
        // Create a temporary world to get a valid entity
        let mut world = hecs::World::new();
        world.spawn(())
    }

    #[test]
    fn test_event_queue_push_and_swap() {
        let mut queue = EventQueue::new();

        // Push event - should not be visible yet
        queue.push(GameEvent::ScoreChanged { score: 100 });
        assert!(queue.is_empty(), "Events should not be visible before swap");

        // Swap - now event should be visible
        queue.swap();
        assert_eq!(queue.len(), 1);

        // Verify event content
        let events: Vec<_> = queue.iter().collect();
        assert!(matches!(events[0], GameEvent::ScoreChanged { score: 100 }));
    }

    #[test]
    fn test_event_queue_double_buffer_isolation() {
        let mut queue = EventQueue::new();

        // Frame 1: Push event A
        queue.push(GameEvent::ScoreChanged { score: 1 });
        queue.swap();

        // Frame 2: Push event B while A is being processed
        queue.push(GameEvent::ScoreChanged { score: 2 });

        // Should only see event A
        let events: Vec<_> = queue.iter().collect();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], GameEvent::ScoreChanged { score: 1 }));

        // Frame 3: Now we see event B
        queue.swap();
        let events: Vec<_> = queue.iter().collect();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], GameEvent::ScoreChanged { score: 2 }));
    }

    #[test]
    fn test_event_queue_drain() {
        let mut queue = EventQueue::new();

        queue.push(GameEvent::StateChanged { state: "playing" });
        queue.push(GameEvent::StateChanged { state: "paused" });
        queue.swap();

        // Drain should consume events
        let events: Vec<_> = queue.drain().collect();
        assert_eq!(events.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_event_queue_clear() {
        let mut queue = EventQueue::new();

        queue.push(GameEvent::ScoreChanged { score: 50 });
        queue.swap();
        queue.push(GameEvent::ScoreChanged { score: 100 });

        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_entity_damaged_event() {
        let entity = test_entity();
        let source = test_entity();

        let event = GameEvent::EntityDamaged {
            entity,
            amount: 25.5,
            source: Some(source),
        };

        // Verify fields via pattern matching
        if let GameEvent::EntityDamaged { amount, source, .. } = event {
            assert!((amount - 25.5).abs() < f32::EPSILON);
            assert!(source.is_some());
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_collision_event() {
        let entity_a = test_entity();
        let entity_b = test_entity();

        let event = GameEvent::Collision {
            entity_a,
            entity_b,
            contact_point: Vec3::new(1.0, 2.0, 3.0),
            normal: Vec3::Y,
        };

        if let GameEvent::Collision {
            contact_point,
            normal,
            ..
        } = event
        {
            assert_eq!(contact_point, Vec3::new(1.0, 2.0, 3.0));
            assert_eq!(normal, Vec3::Y);
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_play_sound_event() {
        let event = GameEvent::PlaySound {
            name: "explosion",
            position: Some(Vec3::ZERO),
            volume: 0.8,
        };

        if let GameEvent::PlaySound {
            name,
            position,
            volume,
        } = event
        {
            assert_eq!(name, "explosion");
            assert!(position.is_some());
            assert!((volume - 0.8).abs() < f32::EPSILON);
        } else {
            panic!("Wrong event type");
        }
    }
}
