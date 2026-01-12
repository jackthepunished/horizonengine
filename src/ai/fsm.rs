//! Finite State Machine for AI Behavior
//!
//! Provides a generic, type-safe state machine for managing AI entity behavior.
//! States encapsulate behavior logic with clean enter/update/exit lifecycle hooks.
//!
//! # Design Principles
//!
//! - **Encapsulation**: Each state owns its behavior and transition logic
//! - **Type Safety**: States are strongly typed with compile-time checking
//! - **Simplicity**: Clean API with minimal boilerplate
//! - **Debuggability**: States have names for logging/debugging
//!
//! # Example
//!
//! ```ignore
//! // Define states
//! struct IdleState;
//! struct PatrolState { waypoints: Vec<Vec3>, current: usize }
//! struct ChaseState { target: Entity }
//!
//! impl State for IdleState {
//!     fn name(&self) -> &'static str { "Idle" }
//!
//!     fn update(&mut self, entity: Entity, ctx: &mut StateContext) -> Transition {
//!         if ctx.can_see_player {
//!             Transition::To(Box::new(ChaseState { target: ctx.player }))
//!         } else {
//!             Transition::None
//!         }
//!     }
//! }
//!
//! // Create and run FSM
//! let mut fsm = StateMachine::new(IdleState);
//! fsm.update(entity, &mut ctx);  // May transition to ChaseState
//! ```

use std::fmt;

// ============================================================================
// State Trait
// ============================================================================

/// A state in the finite state machine.
///
/// States define behavior for an AI entity and control when to transition
/// to other states. The lifecycle is:
///
/// 1. `enter()` - Called once when entering this state
/// 2. `update()` - Called each frame while in this state
/// 3. `exit()` - Called once when leaving this state
pub trait State<Ctx = ()>: fmt::Debug {
    /// State name for debugging and logging.
    fn name(&self) -> &'static str;

    /// Called when entering this state.
    ///
    /// Use this to initialize state-specific data or trigger one-time effects.
    fn enter(&mut self, _ctx: &mut Ctx) {}

    /// Called each frame while in this state.
    ///
    /// Returns a `Transition` to indicate whether to stay or change states.
    fn update(&mut self, ctx: &mut Ctx) -> Transition<Ctx>;

    /// Called when exiting this state.
    ///
    /// Use this to clean up state-specific resources.
    fn exit(&mut self, _ctx: &mut Ctx) {}
}

// ============================================================================
// Transition
// ============================================================================

/// Represents a state transition decision.
///
/// Returned from `State::update()` to indicate whether to stay in the
/// current state or transition to a new one.
pub enum Transition<Ctx = ()> {
    /// Stay in the current state.
    None,
    /// Transition to a new state.
    To(Box<dyn State<Ctx>>),
    /// Pop to parent state (for hierarchical FSM).
    Pop,
}

impl<Ctx> Transition<Ctx> {
    /// Create a transition to a new state.
    pub fn to<S: State<Ctx> + 'static>(state: S) -> Self {
        Transition::To(Box::new(state))
    }
}

impl<Ctx> fmt::Debug for Transition<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Transition::None => write!(f, "Transition::None"),
            Transition::To(state) => write!(f, "Transition::To({})", state.name()),
            Transition::Pop => write!(f, "Transition::Pop"),
        }
    }
}

// ============================================================================
// State Machine
// ============================================================================

/// A finite state machine that manages state transitions.
///
/// The FSM owns the current state and handles the lifecycle of entering,
/// updating, and exiting states.
///
/// # Type Parameters
///
/// - `Ctx`: Context type passed to state methods (e.g., game world, AI data)
pub struct StateMachine<Ctx = ()> {
    /// Current active state
    current: Box<dyn State<Ctx>>,
    /// Whether enter() has been called on current state
    entered: bool,
}

impl<Ctx> StateMachine<Ctx> {
    /// Create a new state machine with an initial state.
    ///
    /// The initial state's `enter()` will be called on the first `update()`.
    pub fn new<S: State<Ctx> + 'static>(initial: S) -> Self {
        Self {
            current: Box::new(initial),
            entered: false,
        }
    }

    /// Update the state machine.
    ///
    /// Calls `enter()` on first update, then `update()` each frame.
    /// Handles transitions by calling `exit()` on old state and `enter()` on new.
    pub fn update(&mut self, ctx: &mut Ctx) {
        // Enter current state if not yet entered
        if !self.entered {
            self.current.enter(ctx);
            self.entered = true;
        }

        // Update and check for transition
        let transition = self.current.update(ctx);

        if let Transition::To(mut new_state) = transition {
            // Exit current state
            self.current.exit(ctx);

            // Enter new state
            new_state.enter(ctx);

            // Replace current state
            self.current = new_state;
            self.entered = true;
        }
    }

    /// Force a transition to a new state.
    ///
    /// Immediately exits the current state and enters the new one.
    pub fn transition<S: State<Ctx> + 'static>(&mut self, ctx: &mut Ctx, new_state: S) {
        if self.entered {
            self.current.exit(ctx);
        }

        self.current = Box::new(new_state);
        self.current.enter(ctx);
        self.entered = true;
    }

    /// Get the name of the current state.
    #[must_use]
    pub fn current_state_name(&self) -> &'static str {
        self.current.name()
    }

    /// Check if the FSM is in a state with the given name.
    #[must_use]
    pub fn is_in_state(&self, name: &str) -> bool {
        self.current.name() == name
    }
}

impl<Ctx> fmt::Debug for StateMachine<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachine")
            .field("current", &self.current.name())
            .field("entered", &self.entered)
            .finish()
    }
}

// ============================================================================
// Example AI States
// ============================================================================

/// Simple context for AI state machines.
///
/// This is a minimal example context. Real games would include
/// entity data, world access, and sensor information.
#[derive(Debug, Default)]
pub struct AiContext {
    /// Delta time for this frame
    pub delta_time: f32,
    /// Whether the AI can see its target
    pub can_see_target: bool,
    /// Distance to target
    pub target_distance: f32,
    /// Time spent in current state
    pub state_time: f32,
    /// Whether an attack was performed this frame (for verification)
    pub attack_performed: bool,
}

/// Idle state - waiting for something to happen.
#[derive(Debug, Default)]
pub struct IdleState {
    /// Time spent idle
    pub idle_time: f32,
    /// Maximum idle time before transitioning
    pub max_idle_time: f32,
}

impl IdleState {
    /// Create idle state with maximum idle time.
    #[must_use]
    pub fn new(max_idle_time: f32) -> Self {
        Self {
            idle_time: 0.0,
            max_idle_time,
        }
    }
}

impl State<AiContext> for IdleState {
    fn name(&self) -> &'static str {
        "Idle"
    }

    fn enter(&mut self, _ctx: &mut AiContext) {
        self.idle_time = 0.0;
    }

    fn update(&mut self, ctx: &mut AiContext) -> Transition<AiContext> {
        self.idle_time += ctx.delta_time;

        // Transition to patrol after idle time
        if self.idle_time >= self.max_idle_time {
            return Transition::to(PatrolState::default());
        }

        // Transition to chase if target visible
        if ctx.can_see_target {
            return Transition::to(ChaseState::default());
        }

        Transition::None
    }
}

/// Patrol state - moving between waypoints.
#[derive(Debug, Default)]
pub struct PatrolState {
    /// Current waypoint index
    pub waypoint_index: usize,
    /// Number of waypoints visited
    pub waypoints_visited: usize,
}

impl State<AiContext> for PatrolState {
    fn name(&self) -> &'static str {
        "Patrol"
    }

    fn enter(&mut self, _ctx: &mut AiContext) {
        self.waypoints_visited = 0;
    }

    fn update(&mut self, ctx: &mut AiContext) -> Transition<AiContext> {
        // Transition to chase if target visible
        if ctx.can_see_target {
            return Transition::to(ChaseState::default());
        }

        // Simulate reaching waypoint
        self.waypoints_visited += 1;
        self.waypoint_index = (self.waypoint_index + 1) % 4;

        Transition::None
    }
}

/// Chase state - pursuing a target.
#[derive(Debug, Default)]
pub struct ChaseState {
    /// Time spent chasing
    pub chase_time: f32,
}

impl State<AiContext> for ChaseState {
    fn name(&self) -> &'static str {
        "Chase"
    }

    fn enter(&mut self, _ctx: &mut AiContext) {
        self.chase_time = 0.0;
    }

    fn update(&mut self, ctx: &mut AiContext) -> Transition<AiContext> {
        self.chase_time += ctx.delta_time;

        // Return to idle if target lost
        if !ctx.can_see_target {
            return Transition::to(IdleState::new(2.0));
        }

        // Transition to attack if close enough
        if ctx.target_distance < 2.0 {
            return Transition::to(AttackState::default());
        }

        Transition::None
    }
}

/// Attack state - attacking the target.
#[derive(Debug)]
pub struct AttackState {
    /// Time remaining before next attack
    pub cooldown: f32,
    /// Time between attacks
    pub attack_rate: f32,
}

impl AttackState {
    pub fn new(attack_rate: f32) -> Self {
        Self {
            cooldown: 0.0,
            attack_rate,
        }
    }
}

impl Default for AttackState {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl State<AiContext> for AttackState {
    fn name(&self) -> &'static str {
        "Attack"
    }

    fn update(&mut self, ctx: &mut AiContext) -> Transition<AiContext> {
        self.cooldown -= ctx.delta_time;
        ctx.attack_performed = false;

        // Return to chase if target moved away
        if ctx.target_distance > 3.0 {
            return Transition::to(ChaseState::default());
        }

        // Return to idle if target lost
        if !ctx.can_see_target {
            return Transition::to(IdleState::new(1.0));
        }

        // Perform attack if cooldown ready
        if self.cooldown <= 0.0 {
            ctx.attack_performed = true;
            // Reset cooldown
            self.cooldown = self.attack_rate;
        }

        Transition::None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsm_initial_state() {
        let fsm: StateMachine<AiContext> = StateMachine::new(IdleState::new(5.0));
        assert_eq!(fsm.current_state_name(), "Idle");
    }

    #[test]
    fn test_fsm_enter_called_on_first_update() {
        let mut fsm = StateMachine::new(IdleState::new(5.0));
        let mut ctx = AiContext::default();

        // Before update, enter hasn't been called
        assert!(!fsm.entered);

        // After update, enter should have been called
        fsm.update(&mut ctx);
        assert!(fsm.entered);
    }

    #[test]
    fn test_fsm_transition_on_condition() {
        let mut fsm = StateMachine::new(IdleState::new(5.0));
        let mut ctx = AiContext {
            can_see_target: true,
            ..Default::default()
        };

        fsm.update(&mut ctx);

        // Should have transitioned to Chase
        assert_eq!(fsm.current_state_name(), "Chase");
    }

    #[test]
    fn test_fsm_transition_on_timeout() {
        let mut fsm = StateMachine::new(IdleState::new(1.0));
        let mut ctx = AiContext {
            delta_time: 1.5,
            ..Default::default()
        };

        fsm.update(&mut ctx);

        // Should have transitioned to Patrol
        assert_eq!(fsm.current_state_name(), "Patrol");
    }

    #[test]
    fn test_fsm_chase_to_attack() {
        let mut fsm = StateMachine::new(ChaseState::default());
        let mut ctx = AiContext {
            can_see_target: true,
            target_distance: 1.5,
            ..Default::default()
        };

        fsm.update(&mut ctx);

        // Should transition to Attack (close enough)
        assert_eq!(fsm.current_state_name(), "Attack");
    }

    #[test]
    fn test_fsm_chase_returns_to_idle() {
        let mut fsm = StateMachine::new(ChaseState::default());
        let mut ctx = AiContext {
            can_see_target: false,
            target_distance: 10.0,
            ..Default::default()
        };

        fsm.update(&mut ctx);

        // Should return to Idle (lost target)
        assert_eq!(fsm.current_state_name(), "Idle");
    }

    #[test]
    fn test_fsm_forced_transition() {
        let mut fsm = StateMachine::new(IdleState::new(5.0));
        let mut ctx = AiContext::default();

        fsm.transition(&mut ctx, AttackState::default());

        assert_eq!(fsm.current_state_name(), "Attack");
    }

    #[test]
    fn test_fsm_is_in_state() {
        let fsm: StateMachine<AiContext> = StateMachine::new(PatrolState::default());

        assert!(fsm.is_in_state("Patrol"));
        assert!(!fsm.is_in_state("Idle"));
    }

    #[test]
    fn test_attack_to_chase_distance() {
        let mut fsm = StateMachine::new(AttackState::default());
        let mut ctx = AiContext {
            can_see_target: true,
            target_distance: 5.0, // Too far to attack
            ..Default::default()
        };

        fsm.update(&mut ctx);

        // Should return to Chase
        assert_eq!(fsm.current_state_name(), "Chase");
    }

    #[test]
    fn test_attack_cooldown() {
        // Attack logic: attacks immediately if cooldown <= 0, then resets to attack_rate
        let mut fsm = StateMachine::new(AttackState::new(1.0));
        let mut ctx = AiContext {
            can_see_target: true,
            target_distance: 1.0,
            delta_time: 0.1,
            ..Default::default()
        };

        // First update: should attack immediately (initial cooldown is 0)
        fsm.update(&mut ctx);
        assert!(
            ctx.attack_performed,
            "Should attack immediately on first update"
        );

        // Second update: cooldown should be ~0.9, no attack
        fsm.update(&mut ctx);
        assert!(
            !ctx.attack_performed,
            "Should not attack while cooldown is active"
        );

        // Fast forward 1.0s
        ctx.delta_time = 1.0;
        fsm.update(&mut ctx);
        assert!(ctx.attack_performed, "Should attack after cooldown expires");
    }
}
