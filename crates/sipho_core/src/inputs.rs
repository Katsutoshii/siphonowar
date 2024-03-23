use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::{prelude::*, utils::HashMap};

use std::hash::Hash;
/// Inputs are configured via an input map (TODO).
/// Mouse events are translated into InputActions.
/// Rays are cast to determine the target of the InputAction.
/// How can we determine what the target was?
use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use crate::prelude::*;

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
            .insert_resource(InputConfig::default())
            .add_event::<ControlEvent>()
            .add_event::<InputEvent>()
            .add_systems(
                Update,
                (InputEvent::update, ControlEvent::update, pan_camera)
                    .chain()
                    .in_set(SystemStage::Input),
            );
    }
}

/// Describes an action input by the user.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum InputAction {
    Primary,
    Secondary,
    PanCamera,
    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
    SpawnPlankton,
    SpawnFood,
    PauseMenu,
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
    pub fn update(
        mut inputs: EventWriter<Self>,
        mut keyboard_inputs: EventReader<KeyboardInput>,
        mut mouse_inputs: EventReader<MouseButtonInput>,
        config: Res<InputConfig>,
    ) {
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

#[derive(Resource, Deref, DerefMut)]
pub struct ControlActions {
    #[deref]
    pub input: ButtonInput<ControlAction>,
    pub position: RaycastEvent,
}
impl ControlActions {}

/// Describes an input action and the worldspace position where it occurred.
#[derive(Event, Debug)]
pub struct ControlEvent {
    pub action: ControlAction,
    pub state: ButtonState,
    pub position: Vec2,
}
impl ControlEvent {
    pub fn is_pressed(&self, action: ControlAction) -> bool {
        self.action == action && self.state == ButtonState::Pressed
    }
    pub fn is_released(&self, action: ControlAction) -> bool {
        self.action == action && self.state == ButtonState::Released
    }
    pub fn update(
        raycast: RaycastCommands,
        mut input_events: EventReader<InputEvent>,
        mut control_events: EventWriter<ControlEvent>,
        cursor: Query<&GlobalTransform, With<Cursor>>,
        grid_spec: Option<Res<GridSpec>>,
        mut timers: Local<ControlTimers>,
        time: Res<Time>,
    ) {
        let grid_spec = if let Some(grid_spec) = grid_spec {
            grid_spec
        } else {
            return;
        };
        let mut raycast_event = None;
        for event in input_events.read() {
            if raycast_event.is_none() {
                raycast_event = raycast.raycast(Cursor::ray3d(cursor.single()))
            }
            if let Some(raycast_event) = &raycast_event {
                let action = ControlAction::from((raycast_event.target, event.action));

                // Skip this action if the timer isn't ready.
                if let Some(timer) = timers.get_mut(&action) {
                    match event.state {
                        ButtonState::Pressed => {
                            timer.unpause();
                            timer.reset();
                        }
                        ButtonState::Released => {
                            timer.pause();
                        }
                    }
                }

                let event = ControlEvent {
                    action,
                    state: event.state,
                    position: match raycast_event.target {
                        RaycastTarget::Minimap => grid_spec.local_to_world_position(
                            raycast_event.position * Vec2 { x: 1., y: -1. },
                        ),
                        RaycastTarget::WorldGrid => raycast_event.world_position,
                        RaycastTarget::None => raycast_event.position,
                    },
                };
                control_events.send(event);
            }
        }
        // Tick all active timers.
        for (&action, timer) in timers.iter_mut() {
            if timer.paused() {
                continue;
            }

            if raycast_event.is_none() {
                raycast_event = raycast.raycast(Cursor::ray3d(cursor.single()))
            }
            timer.tick(time.delta());
            if timer.finished() {
                timer.reset();
                if let Some(raycast_event) = &raycast_event {
                    let event = ControlEvent {
                        action,
                        state: ButtonState::Pressed,
                        position: match raycast_event.target {
                            RaycastTarget::Minimap => grid_spec.local_to_world_position(
                                raycast_event.position * Vec2 { x: 1., y: -1. },
                            ),
                            RaycastTarget::WorldGrid => raycast_event.world_position,
                            RaycastTarget::None => raycast_event.position,
                        },
                    };
                    info!("Held!");
                    dbg!(&event);
                    control_events.send(event);
                }
            }
        }
    }
}

/// Describes an action input by the user.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ControlAction {
    #[default]
    None,
    Select,
    Move,
    PanCamera,

    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
    SpawnPlankton,
    SpawnFood,

    PauseMenu,
}
impl From<(RaycastTarget, InputAction)> for ControlAction {
    fn from(value: (RaycastTarget, InputAction)) -> Self {
        match value {
            (RaycastTarget::Minimap, InputAction::Primary) => Self::PanCamera,
            (RaycastTarget::Minimap, InputAction::PanCamera) => Self::PanCamera,
            (RaycastTarget::Minimap, InputAction::Secondary) => Self::Move,
            (RaycastTarget::WorldGrid, InputAction::Primary) => Self::Select,
            (RaycastTarget::WorldGrid, InputAction::Secondary) => Self::Move,
            (RaycastTarget::WorldGrid, InputAction::PanCamera) => Self::PanCamera,
            (RaycastTarget::WorldGrid, InputAction::SpawnHead) => Self::SpawnHead,
            (RaycastTarget::WorldGrid, InputAction::SpawnZooid) => Self::SpawnZooid,
            (RaycastTarget::WorldGrid, InputAction::SpawnRed) => Self::SpawnRed,
            (RaycastTarget::WorldGrid, InputAction::SpawnBlue) => Self::SpawnBlue,
            (RaycastTarget::WorldGrid, InputAction::SpawnPlankton) => Self::SpawnPlankton,
            (RaycastTarget::WorldGrid, InputAction::SpawnFood) => Self::SpawnFood,
            (_, InputAction::PauseMenu) => Self::PauseMenu,
            (RaycastTarget::None, _) => Self::None,
            _ => Self::None,
        }
    }
}

/// Collection of timers to prevent input action spam.
#[derive(Deref, DerefMut)]
pub struct ControlTimers(HashMap<ControlAction, Timer>);
impl Default for ControlTimers {
    fn default() -> Self {
        let mut timers = Self(HashMap::default());
        timers.insert(
            ControlAction::Move,
            Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        );
        timers.insert(
            ControlAction::Select,
            Timer::new(Duration::from_millis(5), TimerMode::Repeating),
        );
        for (_action, timer) in timers.iter_mut() {
            timer.pause();
        }
        timers
    }
}
impl Index<ControlAction> for ControlTimers {
    type Output = Timer;
    fn index(&self, i: ControlAction) -> &Self::Output {
        self.get(&i).unwrap()
    }
}
impl IndexMut<ControlAction> for ControlTimers {
    fn index_mut(&mut self, i: ControlAction) -> &mut Self::Output {
        self.get_mut(&i).unwrap()
    }
}

pub fn pan_camera(
    mut control_events: EventReader<ControlEvent>,
    mut camera: Query<(&CameraController, &mut Transform), With<MainCamera>>,
) {
    for &ControlEvent {
        action,
        state: _,
        position,
    } in control_events.read()
    {
        if action != ControlAction::PanCamera {
            continue;
        }
        let (controller, mut camera_transform) = camera.single_mut();
        controller.set_position(&mut camera_transform, position);
    }
}
