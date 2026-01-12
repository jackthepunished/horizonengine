//! Command Pattern for Input Actions
//!
//! This module implements the Command pattern to decouple physical inputs from
//! game actions. This enables input rebinding, replay recording, and undo/redo.
//!
//! # Design Principles
//!
//! - **Decoupling**: Physical keys map to logical actions, not direct behavior
//! - **Rebinding**: Players can customize controls at runtime
//! - **Undo/Redo**: Optional support for undoable commands (editor use)
//! - **Type Safety**: Actions are strongly typed via enums
//!
//! # Example
//!
//! ```ignore
//! // Setup input mapper with default bindings
//! let mut mapper = InputMapper::with_defaults();
//!
//! // Rebind a key
//! mapper.bind(KeyCode::ArrowUp, InputAction::MoveForward);
//!
//! // Query actions based on pressed keys
//! if let Some(action) = mapper.get_action(KeyCode::KeyW) {
//!     match action {
//!         InputAction::MoveForward => player.move_forward(delta),
//!         _ => {}
//!     }
//! }
//! ```

use rustc_hash::FxHashMap;
use winit::keyboard::KeyCode;

// ============================================================================
// Input Actions
// ============================================================================

/// Logical input actions that map to game behaviors.
///
/// These represent what the player wants to do, independent of how they
/// trigger it (keyboard, gamepad, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InputAction {
    // -------------------------------------------------------------------------
    // Movement
    // -------------------------------------------------------------------------
    /// Move forward
    MoveForward,
    /// Move backward
    MoveBackward,
    /// Strafe left
    MoveLeft,
    /// Strafe right
    MoveRight,
    /// Move up (fly mode, swim up)
    MoveUp,
    /// Move down (fly mode, swim down, crouch)
    MoveDown,

    // -------------------------------------------------------------------------
    // Actions
    // -------------------------------------------------------------------------
    /// Jump
    Jump,
    /// Sprint/run
    Sprint,
    /// Interact with objects
    Interact,
    /// Primary action (attack, use)
    PrimaryAction,
    /// Secondary action (aim, block)
    SecondaryAction,

    // -------------------------------------------------------------------------
    // Camera
    // -------------------------------------------------------------------------
    /// Look up
    LookUp,
    /// Look down
    LookDown,
    /// Look left
    LookLeft,
    /// Look right
    LookRight,

    // -------------------------------------------------------------------------
    // UI
    // -------------------------------------------------------------------------
    /// Pause/menu
    Pause,
    /// Open inventory
    Inventory,
    /// Confirm/select
    Confirm,
    /// Cancel/back
    Cancel,
}

// ============================================================================
// Input Mapper
// ============================================================================

/// Maps physical inputs to logical actions.
///
/// Supports runtime rebinding and querying of key-to-action mappings.
#[derive(Debug, Clone)]
pub struct InputMapper {
    /// Key to action bindings
    key_bindings: FxHashMap<KeyCode, InputAction>,
    /// Reverse lookup: action to keys (for displaying bindings in UI)
    action_keys: FxHashMap<InputAction, Vec<KeyCode>>,
}

impl InputMapper {
    /// Create an empty input mapper.
    #[must_use]
    pub fn new() -> Self {
        Self {
            key_bindings: FxHashMap::default(),
            action_keys: FxHashMap::default(),
        }
    }

    /// Create an input mapper with default WASD + common bindings.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut mapper = Self::new();

        // Movement (WASD)
        mapper.bind(KeyCode::KeyW, InputAction::MoveForward);
        mapper.bind(KeyCode::KeyS, InputAction::MoveBackward);
        mapper.bind(KeyCode::KeyA, InputAction::MoveLeft);
        mapper.bind(KeyCode::KeyD, InputAction::MoveRight);

        // Arrow key alternatives
        mapper.bind(KeyCode::ArrowUp, InputAction::MoveForward);
        mapper.bind(KeyCode::ArrowDown, InputAction::MoveBackward);
        mapper.bind(KeyCode::ArrowLeft, InputAction::MoveLeft);
        mapper.bind(KeyCode::ArrowRight, InputAction::MoveRight);

        // Vertical movement
        mapper.bind(KeyCode::Space, InputAction::Jump);
        mapper.bind(KeyCode::ControlLeft, InputAction::MoveDown);

        // Actions
        mapper.bind(KeyCode::ShiftLeft, InputAction::Sprint);
        mapper.bind(KeyCode::KeyE, InputAction::Interact);

        // UI
        mapper.bind(KeyCode::Escape, InputAction::Pause);
        mapper.bind(KeyCode::Tab, InputAction::Inventory);
        mapper.bind(KeyCode::Enter, InputAction::Confirm);
        mapper.bind(KeyCode::Backspace, InputAction::Cancel);

        mapper
    }

    /// Bind a key to an action.
    ///
    /// If the key was previously bound, the old binding is replaced.
    pub fn bind(&mut self, key: KeyCode, action: InputAction) {
        // Remove old binding for this key
        if let Some(old_action) = self.key_bindings.get(&key)
            && let Some(keys) = self.action_keys.get_mut(old_action)
        {
            keys.retain(|k| *k != key);
        }

        // Add new binding
        self.key_bindings.insert(key, action);
        self.action_keys.entry(action).or_default().push(key);
    }

    /// Unbind a key.
    pub fn unbind(&mut self, key: KeyCode) {
        if let Some(action) = self.key_bindings.remove(&key)
            && let Some(keys) = self.action_keys.get_mut(&action)
        {
            keys.retain(|k| *k != key);
        }
    }

    /// Unbind all keys for an action.
    pub fn unbind_action(&mut self, action: InputAction) {
        if let Some(keys) = self.action_keys.remove(&action) {
            for key in keys {
                self.key_bindings.remove(&key);
            }
        }
    }

    /// Get the action for a key.
    #[must_use]
    pub fn get_action(&self, key: KeyCode) -> Option<InputAction> {
        self.key_bindings.get(&key).copied()
    }

    /// Get all keys bound to an action.
    #[must_use]
    pub fn get_keys(&self, action: InputAction) -> &[KeyCode] {
        self.action_keys
            .get(&action)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if a key is bound to any action.
    #[must_use]
    pub fn is_bound(&self, key: KeyCode) -> bool {
        self.key_bindings.contains_key(&key)
    }

    /// Get total number of bindings.
    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.key_bindings.len()
    }

    /// Clear all bindings.
    pub fn clear(&mut self) {
        self.key_bindings.clear();
        self.action_keys.clear();
    }

    /// Iterate over all bindings.
    pub fn iter(&self) -> impl Iterator<Item = (KeyCode, InputAction)> + '_ {
        self.key_bindings.iter().map(|(&k, &a)| (k, a))
    }
}

impl Default for InputMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Command Trait
// ============================================================================

/// A command that can be executed (and optionally undone).
///
/// Commands encapsulate actions that can be:
/// - Executed once
/// - Undone (for editor actions)
/// - Recorded for replay
///
/// # Note
///
/// For game input, prefer using `InputAction` with `InputMapper`.
/// The `Command` trait is primarily useful for:
/// - Editor operations with undo/redo
/// - Scripted sequences
/// - Network replays
pub trait Command: std::fmt::Debug {
    /// Execute the command.
    fn execute(&mut self);

    /// Undo the command (optional).
    ///
    /// Returns `true` if undo was successful.
    fn undo(&mut self) -> bool {
        false
    }

    /// Whether this command supports undo.
    fn is_undoable(&self) -> bool {
        false
    }

    /// Command name for debugging/logging.
    fn name(&self) -> &'static str;
}

// ============================================================================
// Command History
// ============================================================================

/// Undo/redo history for commands.
///
/// Maintains two stacks:
/// - Undo stack: commands that have been executed
/// - Redo stack: commands that have been undone
///
/// # Example
///
/// ```ignore
/// let mut history = CommandHistory::new();
///
/// history.execute(Box::new(MyCommand::new()));
/// history.undo();  // Reverts MyCommand
/// history.redo();  // Re-applies MyCommand
/// ```
#[derive(Debug, Default)]
pub struct CommandHistory {
    /// Commands that can be undone
    undo_stack: Vec<Box<dyn Command>>,
    /// Commands that can be redone
    redo_stack: Vec<Box<dyn Command>>,
    /// Maximum history size (0 = unlimited)
    max_size: usize,
}

impl CommandHistory {
    /// Create a new command history with unlimited size.
    #[must_use]
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 0,
        }
    }

    /// Create a new command history with a maximum size.
    ///
    /// When the limit is reached, oldest commands are discarded.
    #[must_use]
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_size.min(100)),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Execute a command and add it to the history.
    ///
    /// Clears the redo stack (can't redo after new actions).
    pub fn execute(&mut self, mut cmd: Box<dyn Command>) {
        cmd.execute();

        if cmd.is_undoable() {
            self.undo_stack.push(cmd);
            self.redo_stack.clear();

            // Enforce max size
            if self.max_size > 0 && self.undo_stack.len() > self.max_size {
                self.undo_stack.remove(0);
            }
        }
    }

    /// Undo the last command.
    ///
    /// Returns `true` if a command was undone.
    pub fn undo(&mut self) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop() {
            if cmd.undo() {
                self.redo_stack.push(cmd);
                return true;
            }
            // If undo failed, push it back
            self.undo_stack.push(cmd);
        }
        false
    }

    /// Redo the last undone command.
    ///
    /// Returns `true` if a command was redone.
    pub fn redo(&mut self) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute();
            self.undo_stack.push(cmd);
            return true;
        }
        false
    }

    /// Check if undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of commands in the undo stack.
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack.
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_mapper_defaults() {
        let mapper = InputMapper::with_defaults();

        assert_eq!(
            mapper.get_action(KeyCode::KeyW),
            Some(InputAction::MoveForward)
        );
        assert_eq!(mapper.get_action(KeyCode::Space), Some(InputAction::Jump));
        assert_eq!(mapper.get_action(KeyCode::Escape), Some(InputAction::Pause));
    }

    #[test]
    fn test_input_mapper_bind() {
        let mut mapper = InputMapper::new();

        mapper.bind(KeyCode::KeyZ, InputAction::Jump);
        assert_eq!(mapper.get_action(KeyCode::KeyZ), Some(InputAction::Jump));
    }

    #[test]
    fn test_input_mapper_rebind() {
        let mut mapper = InputMapper::with_defaults();

        // W is MoveForward by default
        assert_eq!(
            mapper.get_action(KeyCode::KeyW),
            Some(InputAction::MoveForward)
        );

        // Rebind W to Jump
        mapper.bind(KeyCode::KeyW, InputAction::Jump);
        assert_eq!(mapper.get_action(KeyCode::KeyW), Some(InputAction::Jump));

        // MoveForward should no longer have W
        assert!(
            !mapper
                .get_keys(InputAction::MoveForward)
                .contains(&KeyCode::KeyW)
        );
    }

    #[test]
    fn test_input_mapper_unbind() {
        let mut mapper = InputMapper::with_defaults();

        mapper.unbind(KeyCode::KeyW);
        assert!(mapper.get_action(KeyCode::KeyW).is_none());
    }

    #[test]
    fn test_input_mapper_get_keys() {
        let mapper = InputMapper::with_defaults();

        // MoveForward should have both W and ArrowUp
        let keys = mapper.get_keys(InputAction::MoveForward);
        assert!(keys.contains(&KeyCode::KeyW));
        assert!(keys.contains(&KeyCode::ArrowUp));
    }

    #[test]
    fn test_input_mapper_unbind_action() {
        let mut mapper = InputMapper::with_defaults();

        mapper.unbind_action(InputAction::MoveForward);

        assert!(mapper.get_action(KeyCode::KeyW).is_none());
        assert!(mapper.get_action(KeyCode::ArrowUp).is_none());
        assert!(mapper.get_keys(InputAction::MoveForward).is_empty());
    }

    // Test command with simple state
    #[derive(Debug)]
    struct TestCommand {
        #[allow(dead_code)]
        value: i32,
        executed: bool,
    }

    impl TestCommand {
        fn new(value: i32) -> Self {
            Self {
                value,
                executed: false,
            }
        }
    }

    impl Command for TestCommand {
        fn execute(&mut self) {
            self.executed = true;
        }

        fn undo(&mut self) -> bool {
            self.executed = false;
            true
        }

        fn is_undoable(&self) -> bool {
            true
        }

        fn name(&self) -> &'static str {
            "TestCommand"
        }
    }

    #[test]
    fn test_command_history_execute() {
        let mut history = CommandHistory::new();

        history.execute(Box::new(TestCommand::new(42)));

        assert_eq!(history.undo_count(), 1);
        assert!(history.can_undo());
    }

    #[test]
    fn test_command_history_undo_redo() {
        let mut history = CommandHistory::new();

        history.execute(Box::new(TestCommand::new(1)));
        history.execute(Box::new(TestCommand::new(2)));

        assert_eq!(history.undo_count(), 2);

        // Undo
        assert!(history.undo());
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);

        // Redo
        assert!(history.redo());
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_command_history_max_size() {
        let mut history = CommandHistory::with_max_size(2);

        history.execute(Box::new(TestCommand::new(1)));
        history.execute(Box::new(TestCommand::new(2)));
        history.execute(Box::new(TestCommand::new(3)));

        // Should only keep 2 commands
        assert_eq!(history.undo_count(), 2);
    }

    #[test]
    fn test_command_history_clear_redo_on_new_execute() {
        let mut history = CommandHistory::new();

        history.execute(Box::new(TestCommand::new(1)));
        history.undo();
        assert!(history.can_redo());

        // New execution clears redo stack
        history.execute(Box::new(TestCommand::new(2)));
        assert!(!history.can_redo());
    }
}
