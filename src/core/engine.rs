//! Core Engine struct and main game loop

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::core::Time;
use crate::core::debug::DebugInfo;
use crate::ecs::World;
use crate::input::Input;
use crate::renderer::Renderer;

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Window title
    pub title: String,
    /// Initial window width
    pub width: u32,
    /// Initial window height
    pub height: u32,
    /// Target frames per second (0 for unlimited)
    pub target_fps: u32,
    /// Enable VSync
    pub vsync: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: String::from("Engine"),
            width: 1280,
            height: 720,
            target_fps: 60,
            vsync: true,
        }
    }
}

impl EngineConfig {
    /// Create a new config with a title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window dimensions
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set target FPS
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    /// Enable or disable VSync
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

/// Game trait that users implement
pub trait Game: 'static {
    /// Called once when the engine starts
    fn init(&mut self, engine: &mut EngineContext);

    /// Called every frame for game logic updates
    fn update(&mut self, engine: &mut EngineContext);

    /// Called every frame for rendering
    fn render(&mut self, engine: &mut EngineContext);

    /// Called when the window is resized
    fn on_resize(&mut self, _engine: &mut EngineContext, _width: u32, _height: u32) {}

    /// Called when the game is shutting down
    fn shutdown(&mut self, _engine: &mut EngineContext) {}
}

/// Context passed to game callbacks
pub struct EngineContext {
    /// Time tracking
    pub time: Time,
    /// Input state
    pub input: Input,
    /// ECS world
    pub world: World,
    /// Debug information and stats
    pub debug: DebugInfo,
    /// Renderer (available after initialization)
    renderer: Option<Renderer>,
    /// Window size
    window_size: PhysicalSize<u32>,
    /// Should the engine quit
    should_quit: bool,
}

impl EngineContext {
    fn new(width: u32, height: u32) -> Self {
        Self {
            time: Time::new(),
            input: Input::new(),
            world: World::new(),
            debug: DebugInfo::new(),
            renderer: None,
            window_size: PhysicalSize::new(width, height),
            should_quit: false,
        }
    }

    /// Get the renderer
    pub fn renderer(&self) -> &Renderer {
        self.renderer.as_ref().expect("Renderer not initialized")
    }

    /// Get the renderer mutably
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        self.renderer.as_mut().expect("Renderer not initialized")
    }

    /// Check if renderer is available
    pub fn has_renderer(&self) -> bool {
        self.renderer.is_some()
    }

    /// Get window width
    pub fn width(&self) -> u32 {
        self.window_size.width
    }

    /// Get window height
    pub fn height(&self) -> u32 {
        self.window_size.height
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.window_size.width as f32 / self.window_size.height.max(1) as f32
    }

    /// Request engine shutdown
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Check if engine should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

/// Main engine struct
pub struct Engine<G: Game> {
    config: EngineConfig,
    game: G,
    context: EngineContext,
    window: Option<Arc<Window>>,
    initialized: bool,
}

impl<G: Game> Engine<G> {
    /// Create a new engine with the given game
    pub fn new(config: EngineConfig, game: G) -> Self {
        let context = EngineContext::new(config.width, config.height);
        Self {
            config,
            game,
            context,
            window: None,
            initialized: false,
        }
    }

    /// Run the engine
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init();
        log::info!("Starting engine: {}", self.config.title);

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;

        Ok(())
    }
}

impl<G: Game> ApplicationHandler for Engine<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        // Initialize renderer
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window), self.config.vsync));

        self.context.renderer = Some(renderer);
        self.window = Some(window);

        // Initialize game
        if !self.initialized {
            self.game.init(&mut self.context);
            self.initialized = true;
            log::info!("Engine initialized successfully");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, shutting down");
                self.game.shutdown(&mut self.context);
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    self.context.window_size = new_size;
                    if let Some(renderer) = &mut self.context.renderer {
                        renderer.resize(new_size.width, new_size.height);
                    }
                    self.game
                        .on_resize(&mut self.context, new_size.width, new_size.height);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                    self.context.input.process_keyboard(key_code, event.state);
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.context.input.process_mouse_button(button, state);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.context
                    .input
                    .process_mouse_motion(glam::Vec2::new(position.x as f32, position.y as f32));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => glam::Vec2::new(x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        glam::Vec2::new(pos.x as f32, pos.y as f32)
                    }
                };
                self.context.input.process_scroll(scroll);
            }

            WindowEvent::RedrawRequested => {
                // Update time
                self.context.time.update();

                // Update debug stats
                self.context.debug.record_frame(self.context.time.delta());

                // Update game logic
                self.game.update(&mut self.context);

                // Check if should quit
                if self.context.should_quit() {
                    self.game.shutdown(&mut self.context);
                    event_loop.exit();
                    return;
                }

                // Render
                self.game.render(&mut self.context);

                // Clear per-frame input state
                self.context.input.update();

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
