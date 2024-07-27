use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::{ButtonState, InputSystem};
use bevy::{prelude::*, utils::HashMap};

/// Inputs are configured via an input map (TODO).
/// Mouse events are translated into InputActions.
/// Rays are cast to determine the target of the InputAction.
/// How can we determine what the target was?
use crate::prelude::*;
use std::hash::Hash;

/// Plugin for input action events.
pub struct InputActionPlugin;
impl Plugin for InputActionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KeyCode>()
            .register_type::<MouseButton>()
            .register_type::<InputAction>()
            .register_type::<HashMap<MouseButton, InputAction>>()
            .register_type::<HashMap<KeyCode, InputAction>>()
            .register_type::<InputConfig>()
            .init_resource::<InputConfig>()
            .add_event::<InputEvent>()
            .add_event::<RaycastEvent>()
            .add_systems(
                PreUpdate,
                (InputEvent::update)
                    .after(InputSystem)
                    .run_if(in_state(DebugState::NoDebug))
                    .run_if(not(in_state(GameState::PrepareWindow))),
            );
    }
}

/// Describes an action input by the user.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect, Default)]
pub enum InputAction {
    #[default]
    Primary,
    Secondary,
    DragCamera,
    AttackMode,
    SpawnShocker,
    SpawnRed,
    SpawnBlue,
    Fuse,
    PauseMenu,

    // Control groups
    Control1,
    Control2,
    Control3,
    Control4,

    // Grid controls
    Grid11,
    Grid12,
    Grid13,
    Grid14,
    Grid21,
    Grid22,
    Grid23,
    Grid24,
    Grid31,
    Grid32,
    Grid33,
    Grid34,
}

/// Specifies input mapping.
#[derive(Resource, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct InputConfig {
    pub keyboard: HashMap<KeyCode, InputAction>,
    pub mouse: HashMap<MouseButton, InputAction>,
}

#[derive(Event)]
pub struct InputEvent {
    pub action: InputAction,
    pub state: ButtonState,
}
impl InputEvent {
    // Convert direct keyboard/mouse input events into generalized InputEvents
    // with hold durations.
    pub fn update(
        mut inputs: EventWriter<Self>,
        mut keyboard_inputs: EventReader<KeyboardInput>,
        mut mouse_inputs: EventReader<MouseButtonInput>,
        config: Res<InputConfig>,
        cursor: CursorParam,
        raycast: RaycastCommands,
        mut raycasts: EventWriter<RaycastEvent>,
    ) {
        if let Some(ray) = cursor.ray3d() {
            if let Some(event) = raycast.raycast(ray) {
                raycasts.send(event);
            }
        }

        for event in keyboard_inputs.read() {
            let KeyboardInput {
                key_code, state, ..
            } = event;
            if let Some(&action) = config.keyboard.get(key_code) {
                inputs.send(Self {
                    action,
                    state: *state,
                });
            }
        }
        for event in mouse_inputs.read() {
            let MouseButtonInput { button, state, .. } = event;
            if let Some(&action) = config.mouse.get(button) {
                inputs.send(Self {
                    action,
                    state: *state,
                });
            }
        }
    }
}
