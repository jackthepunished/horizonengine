//! World wrapper around hecs

use hecs::Entity;

/// Game world containing all entities and components
pub struct World {
    /// The underlying hecs world
    pub inner: hecs::World,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }

    /// Spawn an entity with the given components
    pub fn spawn(&mut self, components: impl hecs::DynamicBundle) -> Entity {
        self.inner.spawn(components)
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) -> Result<(), hecs::NoSuchEntity> {
        self.inner.despawn(entity)
    }

    /// Get a reference to a component
    pub fn get<T: hecs::Component>(
        &self,
        entity: Entity,
    ) -> Result<hecs::Ref<'_, T>, hecs::ComponentError> {
        self.inner.get::<&T>(entity)
    }

    /// Get a mutable reference to a component
    pub fn get_mut<T: hecs::Component>(
        &mut self,
        entity: Entity,
    ) -> Result<hecs::RefMut<'_, T>, hecs::ComponentError> {
        self.inner.get::<&mut T>(entity)
    }

    /// Check if an entity exists
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    /// Get the number of entities
    pub fn len(&self) -> u32 {
        self.inner.len()
    }

    /// Check if the world is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all entities from the world
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Query for entities with specific components
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Query for entities with specific components (mutable)
    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryMut<'_, Q> {
        self.inner.query_mut::<Q>()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
