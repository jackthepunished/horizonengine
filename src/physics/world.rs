//! Physics simulation using rapier3d

use glam::{Quat, Vec3};
use nalgebra::UnitQuaternion;
use rapier3d::prelude::*;

/// Handle to a rigid body in the physics world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RigidBodyHandle(pub rapier3d::dynamics::RigidBodyHandle);

/// Handle to a collider in the physics world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColliderHandle(pub rapier3d::geometry::ColliderHandle);

/// Convert glam Quat to rapier3d UnitQuaternion
fn quat_to_rapier(q: Quat) -> UnitQuaternion<f32> {
    UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(q.w, q.x, q.y, q.z))
}

/// Convert rapier3d UnitQuaternion to glam Quat
fn rapier_to_quat(uq: &UnitQuaternion<f32>) -> Quat {
    let q = uq.quaternion();
    Quat::from_xyzw(q.i, q.j, q.k, q.w)
}

/// Physics world manager
pub struct Physics {
    /// Gravity vector
    pub gravity: Vec3,
    /// Physics pipeline
    pipeline: PhysicsPipeline,
    /// Island manager
    island_manager: IslandManager,
    /// Broad phase
    broad_phase: DefaultBroadPhase,
    /// Narrow phase
    narrow_phase: NarrowPhase,
    /// Rigid body set
    rigid_body_set: RigidBodySet,
    /// Collider set
    collider_set: ColliderSet,
    /// Impulse joint set
    impulse_joint_set: ImpulseJointSet,
    /// Multibody joint set
    multibody_joint_set: MultibodyJointSet,
    /// CCD solver
    ccd_solver: CCDSolver,
    /// Query pipeline for raycasting
    query_pipeline: QueryPipeline,
    /// Integration parameters
    integration_parameters: IntegrationParameters,
}

impl Physics {
    /// Create a new physics world with default gravity
    pub fn new() -> Self {
        Self::with_gravity(Vec3::new(0.0, -9.81, 0.0))
    }

    /// Create a new physics world with custom gravity
    pub fn with_gravity(gravity: Vec3) -> Self {
        Self {
            gravity,
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
        }
    }

    /// Step the physics simulation
    pub fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;

        self.pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    /// Create a static rigid body (doesn't move)
    pub fn create_static_body(&mut self, position: Vec3, rotation: Quat) -> RigidBodyHandle {
        let isometry = Isometry::from_parts(
            nalgebra::Translation3::new(position.x, position.y, position.z),
            quat_to_rapier(rotation),
        );
        let body = RigidBodyBuilder::fixed().position(isometry).build();

        RigidBodyHandle(self.rigid_body_set.insert(body))
    }

    /// Create a dynamic rigid body (affected by forces)
    pub fn create_dynamic_body(&mut self, position: Vec3, rotation: Quat) -> RigidBodyHandle {
        let isometry = Isometry::from_parts(
            nalgebra::Translation3::new(position.x, position.y, position.z),
            quat_to_rapier(rotation),
        );
        let body = RigidBodyBuilder::dynamic().position(isometry).build();

        RigidBodyHandle(self.rigid_body_set.insert(body))
    }

    /// Create a kinematic rigid body (controlled directly)
    pub fn create_kinematic_body(&mut self, position: Vec3, rotation: Quat) -> RigidBodyHandle {
        let isometry = Isometry::from_parts(
            nalgebra::Translation3::new(position.x, position.y, position.z),
            quat_to_rapier(rotation),
        );
        let body = RigidBodyBuilder::kinematic_position_based()
            .position(isometry)
            .build();

        RigidBodyHandle(self.rigid_body_set.insert(body))
    }

    /// Add a box collider to a rigid body
    pub fn add_box_collider(
        &mut self,
        body: RigidBodyHandle,
        half_extents: Vec3,
        density: f32,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .density(density)
            .build();

        ColliderHandle(self.collider_set.insert_with_parent(
            collider,
            body.0,
            &mut self.rigid_body_set,
        ))
    }

    /// Add a sphere collider to a rigid body
    pub fn add_sphere_collider(
        &mut self,
        body: RigidBodyHandle,
        radius: f32,
        density: f32,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::ball(radius).density(density).build();

        ColliderHandle(self.collider_set.insert_with_parent(
            collider,
            body.0,
            &mut self.rigid_body_set,
        ))
    }

    /// Add a capsule collider to a rigid body
    pub fn add_capsule_collider(
        &mut self,
        body: RigidBodyHandle,
        half_height: f32,
        radius: f32,
        density: f32,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::capsule_y(half_height, radius)
            .density(density)
            .build();

        ColliderHandle(self.collider_set.insert_with_parent(
            collider,
            body.0,
            &mut self.rigid_body_set,
        ))
    }

    /// Add a ground plane collider
    pub fn add_ground_plane(&mut self, body: RigidBodyHandle) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();

        ColliderHandle(self.collider_set.insert_with_parent(
            collider,
            body.0,
            &mut self.rigid_body_set,
        ))
    }

    /// Get the position of a rigid body
    pub fn get_position(&self, body: RigidBodyHandle) -> Option<Vec3> {
        self.rigid_body_set.get(body.0).map(|rb| {
            let pos = rb.translation();
            Vec3::new(pos.x, pos.y, pos.z)
        })
    }

    /// Get the rotation of a rigid body
    pub fn get_rotation(&self, body: RigidBodyHandle) -> Option<Quat> {
        self.rigid_body_set
            .get(body.0)
            .map(|rb| rapier_to_quat(rb.rotation()))
    }

    /// Set the position of a kinematic body
    pub fn set_kinematic_position(&mut self, body: RigidBodyHandle, position: Vec3) {
        if let Some(rb) = self.rigid_body_set.get_mut(body.0) {
            rb.set_next_kinematic_translation(vector![position.x, position.y, position.z]);
        }
    }

    /// Apply a force to a dynamic body
    pub fn apply_force(&mut self, body: RigidBodyHandle, force: Vec3) {
        if let Some(rb) = self.rigid_body_set.get_mut(body.0) {
            rb.add_force(vector![force.x, force.y, force.z], true);
        }
    }

    /// Apply an impulse to a dynamic body
    pub fn apply_impulse(&mut self, body: RigidBodyHandle, impulse: Vec3) {
        if let Some(rb) = self.rigid_body_set.get_mut(body.0) {
            rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
        }
    }

    /// Set the linear velocity of a body
    pub fn set_linear_velocity(&mut self, body: RigidBodyHandle, velocity: Vec3) {
        if let Some(rb) = self.rigid_body_set.get_mut(body.0) {
            rb.set_linvel(vector![velocity.x, velocity.y, velocity.z], true);
        }
    }

    /// Get the linear velocity of a body
    pub fn get_linear_velocity(&self, body: RigidBodyHandle) -> Option<Vec3> {
        self.rigid_body_set.get(body.0).map(|rb| {
            let vel = rb.linvel();
            Vec3::new(vel.x, vel.y, vel.z)
        })
    }

    /// Cast a ray and return the first hit
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        self.query_pipeline
            .cast_ray(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_distance,
                true,
                QueryFilter::default(),
            )
            .map(|(handle, distance)| {
                let point = ray.point_at(distance);
                RaycastHit {
                    collider: ColliderHandle(handle),
                    point: Vec3::new(point.x, point.y, point.z),
                    distance,
                }
            })
    }

    /// Remove a rigid body and its colliders
    pub fn remove_body(&mut self, body: RigidBodyHandle) {
        self.rigid_body_set.remove(
            body.0,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a raycast
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// The collider that was hit
    pub collider: ColliderHandle,
    /// The point of intersection
    pub point: Vec3,
    /// Distance from ray origin
    pub distance: f32,
}
