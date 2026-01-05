//! Example game demonstrating engine features

use engine::prelude::*;

/// Demo game with rotating cubes and physics
struct DemoGame {
    camera: Camera,
    light: Light,
    cube_mesh: Option<Mesh>,
    ground_mesh: Option<Mesh>,
    cube_model: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    ground_model: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    physics: Physics,
    cube_body: Option<RigidBodyHandle>,
    camera_yaw: f32,
    camera_pitch: f32,
}

impl DemoGame {
    fn new() -> Self {
        Self {
            camera: Camera::look_at(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y),
            light: Light::new(Vec3::new(5.0, 10.0, 5.0)),
            cube_mesh: None,
            ground_mesh: None,
            cube_model: None,
            ground_model: None,
            physics: Physics::new(),
            cube_body: None,
            camera_yaw: 0.0,
            camera_pitch: 0.3,
        }
    }
}

impl Game for DemoGame {
    fn init(&mut self, ctx: &mut EngineContext) {
        log::info!("Initializing demo game");

        // Create meshes
        let mut cube = Mesh::cube();
        let mut ground = Mesh::plane(20.0);

        // Upload to GPU
        ctx.renderer_mut().upload_mesh(&mut cube);
        ctx.renderer_mut().upload_mesh(&mut ground);

        // Create model bind groups
        let cube_transform = Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0));
        self.cube_model = Some(ctx.renderer().create_model_bind_group(cube_transform));

        let ground_transform = Mat4::IDENTITY;
        self.ground_model = Some(ctx.renderer().create_model_bind_group(ground_transform));

        self.cube_mesh = Some(cube);
        self.ground_mesh = Some(ground);

        // Setup physics
        let ground_body = self.physics.create_static_body(Vec3::ZERO, Quat::IDENTITY);
        self.physics.add_ground_plane(ground_body);

        let cube_body = self
            .physics
            .create_dynamic_body(Vec3::new(0.0, 5.0, 0.0), Quat::IDENTITY);
        self.physics
            .add_box_collider(cube_body, Vec3::splat(0.5), 1.0);
        self.cube_body = Some(cube_body);

        // Set camera aspect ratio
        self.camera.set_aspect(ctx.width(), ctx.height());

        log::info!("Demo game initialized");
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.time.delta_seconds();

        // Handle input
        if ctx.input.is_key_pressed(KeyCode::Escape) {
            ctx.quit();
            return;
        }

        // Camera rotation with arrow keys
        let rotation_speed = 2.0;
        if ctx.input.is_key_pressed(KeyCode::ArrowLeft) {
            self.camera_yaw -= rotation_speed * dt;
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowRight) {
            self.camera_yaw += rotation_speed * dt;
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowUp) {
            self.camera_pitch -= rotation_speed * dt;
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowDown) {
            self.camera_pitch += rotation_speed * dt;
        }

        // Clamp pitch
        self.camera_pitch = self.camera_pitch.clamp(-1.4, 1.4);

        // Update camera position (orbit around origin)
        let distance = 12.0;
        self.camera.position = Vec3::new(
            distance * self.camera_yaw.cos() * self.camera_pitch.cos(),
            distance * self.camera_pitch.sin() + 3.0,
            distance * self.camera_yaw.sin() * self.camera_pitch.cos(),
        );
        self.camera.direction = (Vec3::ZERO - self.camera.position).normalize();

        // Reset cube with Space
        if ctx.input.is_key_just_pressed(KeyCode::Space) {
            // Remove old body and create new one
            if let Some(body) = self.cube_body {
                self.physics.remove_body(body);
            }
            let cube_body = self
                .physics
                .create_dynamic_body(Vec3::new(0.0, 5.0, 0.0), Quat::IDENTITY);
            self.physics
                .add_box_collider(cube_body, Vec3::splat(0.5), 1.0);
            self.cube_body = Some(cube_body);
        }

        // Apply force with WASD
        let force_strength = 10.0;
        let mut force = Vec3::ZERO;
        if ctx.input.is_key_pressed(KeyCode::KeyW) {
            force.z -= force_strength;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyS) {
            force.z += force_strength;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyA) {
            force.x -= force_strength;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            force.x += force_strength;
        }

        // Apply force if cube exists and force is non-zero
        if let Some(body) = self.cube_body.filter(|_| force != Vec3::ZERO) {
            self.physics.apply_force(body, force);
        }

        // Step physics
        self.physics.step(dt);

        // Update cube transform from physics
        if let (Some(body), Some((buffer, _))) = (self.cube_body, &self.cube_model) {
            let pos = self.physics.get_position(body);
            let rot = self.physics.get_rotation(body);
            if let (Some(pos), Some(rot)) = (pos, rot) {
                let transform = Mat4::from_rotation_translation(rot, pos);
                ctx.renderer().update_model_buffer(buffer, transform);
            }
        }
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        // Update camera and light
        ctx.renderer_mut().update_camera(&self.camera);
        ctx.renderer_mut().update_light(&self.light);

        // Begin frame
        let Some(mut frame) = ctx.renderer().begin_frame() else {
            return;
        };

        {
            let mut render_pass = ctx.renderer().begin_render_pass(&mut frame);

            // Draw ground
            if let (Some(mesh), Some((_, bind_group))) = (&self.ground_mesh, &self.ground_model) {
                ctx.renderer().draw_mesh(&mut render_pass, mesh, bind_group);
            }

            // Draw cube
            if let (Some(mesh), Some((_, bind_group))) = (&self.cube_mesh, &self.cube_model) {
                ctx.renderer().draw_mesh(&mut render_pass, mesh, bind_group);
            }
        }

        ctx.renderer().end_frame(frame);
    }

    fn on_resize(&mut self, _ctx: &mut EngineContext, width: u32, height: u32) {
        self.camera.set_aspect(width, height);
    }
}

fn main() {
    let config = EngineConfig::default()
        .with_title("Engine Demo")
        .with_size(1280, 720)
        .with_vsync(true);

    let game = DemoGame::new();
    let engine = Engine::new(config, game);

    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
    }
}
