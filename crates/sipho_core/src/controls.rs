use bevy::input::ButtonState;
use bevy::time::Stopwatch;
use bevy::{prelude::*, utils::HashMap};

use std::hash::Hash;
/// Inputs are configured via an input map (TODO).
/// Mouse events are translated into InputActions.
/// Rays are cast to determine the target of the InputAction.
/// How can we determine what the target was?
use std::time::Duration;

use crate::prelude::*;

/// Plugin for input action events.
pub struct ControlActionPlugin;
impl Plugin for ControlActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlState>()
            .add_event::<ControlEvent>()
            .register_type::<ControlAction>()
            .add_systems(
                FixedUpdate,
                ControlEvent::update.in_set(FixedUpdateStage::Control),
            );
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

// Sends events at a given interval for events until the key is released OR a new raycast target is provided.
#[derive(Reflect, Default, Clone)]
pub struct HeldActionRepeater {
    pub timer: Timer,
    pub target: RaycastTarget,
}

#[derive(Default, Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct ControlState {
    pub mode: ControlMode,
    // For held controls, stores their recurring timers.
    pub held_actions: HashMap<ControlAction, HeldActionRepeater>,
    // For held controls, stores their raycast target.
    pub held_action_targets: HashMap<ControlAction, RaycastTarget>,
    // For pressed controls, stores their recurring timers.
    pub press_durations: HashMap<ControlAction, Stopwatch>,
    // Entity being hovered.
    pub hovered_entity: Option<Entity>,
    pub input_targets: HashMap<InputAction, RaycastTarget>,
}
impl ControlState {
    pub fn press_action(&mut self, action: ControlAction, target: RaycastTarget) {
        let duration = action.get_repeat_duration();
        if duration.as_nanos() > 0 {
            self.held_actions.insert(
                action,
                HeldActionRepeater {
                    timer: Timer::new(action.get_repeat_duration(), TimerMode::Repeating),
                    target,
                },
            );
        }
        self.press_durations.insert(action, Stopwatch::new());
    }

    pub fn tick(&mut self, delta: Duration) {
        for (_action, repeater) in self.held_actions.iter_mut() {
            repeater.timer.tick(delta);
        }
        for (_action, stopwatch) in self.press_durations.iter_mut() {
            stopwatch.tick(delta);
        }
    }

    pub fn release_action(&mut self, action: ControlAction) {
        self.press_durations.remove(&action);
        self.held_actions.remove(&action);
    }

    pub fn release_all(&mut self) {
        self.press_durations.clear();
        self.held_actions.clear();
    }

    pub fn get_duration(&mut self, action: ControlAction) -> Duration {
        if let Some(stopwatches) = self.press_durations.get(&action) {
            stopwatches.elapsed()
        } else {
            Duration::from_millis(0)
        }
    }

    pub fn get_repeat_events(
        &mut self,
        grid_spec: &GridSpec,
        raycast_event: &RaycastEvent,
    ) -> Vec<ControlEvent> {
        let mut events = Vec::new();
        let mut actions_to_release: Vec<ControlAction> = Vec::new();

        for (&action, repeater) in self.held_actions.iter() {
            // When the input goes from Minimap -> WorldGrid, release.
            if let (RaycastTarget::Minimap, RaycastTarget::WorldGrid) =
                (repeater.target, raycast_event.target)
            {
                events.push(ControlEvent {
                    action,
                    state: ButtonState::Released,
                    position: ControlEvent::compute_position(grid_spec, raycast_event),
                    entity: raycast_event.entity,
                    duration: Duration::from_millis(0),
                });
                actions_to_release.push(action);
                continue;
            }
            if repeater.timer.finished() {
                let mut raycast_event = raycast_event.clone();
                if action == ControlAction::Select {
                    raycast_event.target = RaycastTarget::WorldGrid;
                }
                events.push(ControlEvent {
                    action,
                    state: ButtonState::Pressed,
                    position: ControlEvent::compute_position(grid_spec, &raycast_event),
                    entity: raycast_event.entity,
                    duration: Duration::from_millis(0),
                });
            }
        }
        for action in actions_to_release {
            self.release_action(action);
        }
        events
    }
}

/// Describes an input action and the worldspace position where it occurred.
#[derive(Event, Debug)]
pub struct ControlEvent {
    pub action: ControlAction,
    pub state: ButtonState,
    pub position: Vec2,
    pub entity: Entity,
    pub duration: Duration,
}
impl ControlEvent {
    pub fn compute_position(grid_spec: &GridSpec, raycast: &RaycastEvent) -> Vec2 {
        match raycast.target {
            RaycastTarget::Minimap => grid_spec.uv_to_world_position(raycast.position),
            RaycastTarget::WorldGrid => raycast.world_position,
            RaycastTarget::None => raycast.position,
            RaycastTarget::GridEntity => raycast.world_position,
        }
    }
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
        mut raycast_events: EventReader<RaycastEvent>,
        mut input_events: EventReader<InputEvent>,
        mut control_events: EventWriter<ControlEvent>,
        grid_spec: Option<Res<GridSpec>>,
        time: Res<Time>,
        mut state: ResMut<ControlState>,
    ) {
        let grid_spec = if let Some(grid_spec) = grid_spec {
            grid_spec
        } else {
            return;
        };

        let raycast_event = raycast_events.read().next();

        // If no inputs, send hover.
        if input_events.is_empty() {
            if let Some(raycast_event) = raycast_event {
                if raycast_event.target == RaycastTarget::GridEntity {
                    let new_hover = if let Some(hovered_entity) = state.hovered_entity {
                        hovered_entity != raycast_event.entity
                    } else {
                        true
                    };
                    if new_hover {
                        state.hovered_entity = Some(raycast_event.entity);
                        control_events.send(ControlEvent {
                            action: ControlAction::SelectHover,
                            state: ButtonState::Pressed,
                            entity: raycast_event.entity,
                            position: ControlEvent::compute_position(&grid_spec, raycast_event),
                            duration: Duration::default(),
                        });
                    }
                } else {
                    if let Some(hovered_entity) = state.hovered_entity {
                        control_events.send(ControlEvent {
                            action: ControlAction::SelectHover,
                            state: ButtonState::Released,
                            entity: hovered_entity,
                            position: ControlEvent::compute_position(&grid_spec, raycast_event),
                            duration: Duration::default(),
                        });
                    }
                    state.hovered_entity = None;
                }
            }
        }

        // Process inputs
        for event in input_events.read() {
            if let Some(raycast_event) = &raycast_event {
                let action = ControlAction::from((raycast_event.target, state.mode, event.action));
                if let ControlAction::AttackMove = action {
                    state.mode = ControlMode::Normal;
                }
                if event.state == ButtonState::Pressed {
                    if action == ControlAction::Select {
                        let held_actions: Vec<ControlAction> =
                            state.held_actions.keys().copied().collect();

                        for action in held_actions {
                            control_events.send(ControlEvent {
                                action,
                                state: ButtonState::Released,
                                entity: raycast_event.entity,
                                position: ControlEvent::compute_position(&grid_spec, raycast_event),
                                duration: state.get_duration(action),
                            });
                            state.release_action(action);
                        }
                    }
                    state.press_action(action, raycast_event.target);
                } else if event.state == ButtonState::Released
                    && !state.press_durations.contains_key(&action)
                {
                    continue;
                }

                control_events.send(ControlEvent {
                    action,
                    state: event.state,
                    entity: raycast_event.entity,
                    position: ControlEvent::compute_position(&grid_spec, raycast_event),
                    duration: state.get_duration(action),
                });

                if event.state == ButtonState::Released {
                    state.release_action(action);
                }
            }
        }

        state.tick(time.delta());
        if let Some(raycast_event) = &raycast_event {
            for event in state.get_repeat_events(&grid_spec, raycast_event) {
                control_events.send(event);
            }
        }
    }
}

/// Describes an action input by the user.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum ControlAction {
    #[default]
    None,
    Select,
    SelectHover,
    Move,
    AttackMove,
    Attack,
    PanCamera,
    DragCamera,
    Head,
    Grow,
    SpawnShocker,
    SpawnRed,
    SpawnBlue,
    Plankton,
    TieAll,
    Tie,
    Fuse,
    PauseMenu,
    // Build
    Worker,
    Armor,
    Shocker,
}
impl ControlAction {
    pub fn get_repeat_duration(self) -> Duration {
        match self {
            Self::Move => Duration::from_millis(100),
            Self::Select => Duration::from_millis(5),
            Self::DragCamera => Duration::from_millis(5),
            Self::PanCamera => Duration::from_millis(5),
            Self::Worker | Self::Armor | Self::Shocker => Duration::from_millis(1),
            _ => Duration::from_millis(0),
        }
    }
}
impl From<(RaycastTarget, ControlMode, InputAction)> for ControlAction {
    fn from(value: (RaycastTarget, ControlMode, InputAction)) -> Self {
        match value {
            (RaycastTarget::Minimap, _, InputAction::Primary) => Self::PanCamera,
            (RaycastTarget::Minimap, _, InputAction::Secondary) => Self::Move,
            (
                RaycastTarget::WorldGrid | RaycastTarget::GridEntity,
                ControlMode::Normal,
                InputAction::Primary,
            ) => Self::Select,
            (
                RaycastTarget::WorldGrid | RaycastTarget::GridEntity,
                ControlMode::Attack,
                InputAction::Primary,
            ) => Self::AttackMove,
            (RaycastTarget::WorldGrid, _, InputAction::Secondary) => Self::Move,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnShocker) => Self::SpawnShocker,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnRed) => Self::SpawnRed,
            (RaycastTarget::WorldGrid, _, InputAction::SpawnBlue) => Self::SpawnBlue,
            (RaycastTarget::WorldGrid, _, InputAction::Fuse) => Self::Fuse,
            (RaycastTarget::WorldGrid, _, InputAction::DragCamera) => Self::DragCamera,
            (_, _, InputAction::PauseMenu) => Self::PauseMenu,
            (RaycastTarget::None, _, _) => Self::None,
            _ => Self::None,
        }
    }
}
