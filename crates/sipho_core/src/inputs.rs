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
            .init_resource::<InputConfig>()
            .init_resource::<ControlState>()
            .add_event::<ControlEvent>()
            .add_event::<InputEvent>()
            .add_systems(
                Update,
                (InputEvent::update, ControlEvent::update)
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
    DragCamera,
    AttackMode,
    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
    SpawnPlankton,
    SpawnFood,
    TieWorkers,
    PauseMenu,
    Fuse,
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

#[derive(Debug, Default, Reflect, Clone, Copy, PartialEq)]
pub enum ControlMode {
    #[default]
    Normal,
    Attack,
}
impl From<InputAction> for ControlMode {
    fn from(action: InputAction) -> Self {
        match action {
            InputAction::AttackMode => ControlMode::Attack,
            _ => ControlMode::Normal,
        }
    }
}

#[derive(Default, Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct ControlState {
    pub mode: ControlMode,
}

/// Describes an input action and the worldspace position where it occurred.
#[derive(Event, Debug)]
pub struct ControlEvent {
    pub action: ControlAction,
    pub state: ButtonState,
    pub mode: ControlMode,
    pub position: Vec2,
}
impl ControlEvent {
    pub fn is_pressed(&self, action: ControlAction) -> bool {
        self.action == action && self.state == ButtonState::Pressed
    }
    pub fn is_released(&self, action: ControlAction) -> bool {
        self.action == action && self.state == ButtonState::Released
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        raycast: RaycastCommands,
        mut input_events: EventReader<InputEvent>,
        mut control_events: EventWriter<ControlEvent>,
        cursor: CursorParam,
        grid_spec: Option<Res<GridSpec>>,
        mut timers: Local<ControlTimers>,
        time: Res<Time>,
        mut state: ResMut<ControlState>,
    ) {
        let grid_spec = if let Some(grid_spec) = grid_spec {
            grid_spec
        } else {
            return;
        };
        let mut raycast_event = None;
        for event in input_events.read() {
            if raycast_event.is_none() {
                if let Some(ray) = cursor.ray3d() {
                    raycast_event = raycast.raycast(ray);
                }
            }
            if let Some(raycast_event) = &raycast_event {
                dbg!(raycast_event);
                let action = ControlAction::from((raycast_event.target, state.mode, event.action));

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

                control_events.send(ControlEvent {
                    action,
                    state: event.state,
                    mode: state.mode,
                    position: match raycast_event.target {
                        RaycastTarget::Minimap => grid_spec.local_to_world_position(
                            raycast_event.position * Vec2 { x: 1., y: -1. },
                        ),
                        RaycastTarget::WorldGrid => raycast_event.world_position,
                        RaycastTarget::None => raycast_event.position,
                    },
                });

                state.mode = ControlMode::from(event.action);
            } else {
                warn!("No raycast");
            }
        }
        // Tick all active timers.
        for (&action, timer) in timers.iter_mut() {
            if timer.paused() {
                continue;
            }

            if raycast_event.is_none() {
                if let Some(ray) = cursor.ray3d() {
                    raycast_event = raycast.raycast(ray);
                }
            }
            timer.tick(time.delta());
            if timer.finished() {
                timer.reset();
                if let Some(raycast_event) = &raycast_event {
                    let event = ControlEvent {
                        action,
                        state: ButtonState::Pressed,
                        mode: ControlMode::Normal,
                        position: match raycast_event.target {
                            RaycastTarget::Minimap => grid_spec.local_to_world_position(
                                raycast_event.position * Vec2 { x: 1., y: -1. },
                            ),
                            RaycastTarget::WorldGrid => raycast_event.world_position,
                            RaycastTarget::None => raycast_event.position,
                        },
                    };
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
    AttackMove,
    PanCamera,
    DragCamera,
    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
    SpawnPlankton,
    SpawnFood,
    TieWorkers,
    Fuse,
    PauseMenu,
}
impl From<(RaycastTarget, ControlMode, InputAction)> for ControlAction {
    fn from(value: (RaycastTarget, ControlMode, InputAction)) -> Self {
        match value {
            (RaycastTarget::Minimap, _, InputAction::Primary) => Self::PanCamera,
            (RaycastTarget::Minimap, _, InputAction::Secondary) => Self::Move,
            (RaycastTarget::WorldGrid, ControlMode::Normal, InputAction::Primary) => Self::Select,
            (RaycastTarget::WorldGrid, ControlMode::Attack, InputAction::Primary) => {
                Self::AttackMove
            }
            (RaycastTarget::WorldGrid, _, InputAction::Secondary) => Self::Move,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnHead) => Self::SpawnHead,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnZooid) => Self::SpawnZooid,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnRed) => Self::SpawnRed,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnBlue) => Self::SpawnBlue,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnPlankton) => Self::SpawnPlankton,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnFood) => Self::SpawnFood,
            (RaycastTarget::WorldGrid, _, InputAction::Fuse) => Self::Fuse,
            (RaycastTarget::WorldGrid, _, InputAction::TieWorkers) => Self::TieWorkers,
            (RaycastTarget::WorldGrid, _, InputAction::DragCamera) => Self::DragCamera,
            (_, _, InputAction::PauseMenu) => Self::PauseMenu,
            (RaycastTarget::None, _, _) => Self::None,

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
        timers.insert(
            ControlAction::DragCamera,
            Timer::new(Duration::from_millis(5), TimerMode::Repeating),
        );
        timers.insert(
            ControlAction::PanCamera,
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
