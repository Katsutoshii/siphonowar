use crate::prelude::*;
use bevy::utils::HashMap;
use enum_iterator::all;
use sipho_core::inputs::InputAction;

pub struct ObjectConfigPlugin;
impl Plugin for ObjectConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Vec2>()
            .register_type::<ObjectiveConfig>()
            .register_type::<InteractionConfig>()
            .register_type::<ObjectControlConfig>()
            .register_type::<HashMap<InputAction, ControlAction>>()
            .register_type::<HashMap<Object, ObjectConfig>>()
            .register_type::<HashMap<Object, InteractionConfig>>()
            .register_type::<ObjectConfig>()
            .register_type::<ObjectConfigs>()
            .register_type::<InteractionConfigs>()
            .add_systems(OnExit(GameState::Loading), ObjectConfigs::setup)
            .insert_resource(ObjectConfigs::default());
    }
}

#[derive(Clone, Default, Deref, DerefMut, Reflect, Debug)]
pub struct ObjectControlConfig(pub HashMap<InputAction, ControlAction>);

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub struct InteractionConfigs(pub HashMap<Object, InteractionConfig>);
/// Describes interactions between two objects
#[derive(Clone, Reflect, Debug)]
pub struct InteractionConfig {
    pub separation_radius: f32,
    pub separation_force: f32,
    pub cohesion_force: f32,
    pub alignment_factor: f32,
    pub damage_amount: i32,
}
impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            separation_radius: 1.0,
            separation_force: 0.0,
            cohesion_force: 0.0,
            alignment_factor: 0.0,
            damage_amount: 0,
        }
    }
}

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub struct ObjectConfigs(pub HashMap<Object, ObjectConfig>);

#[derive(Clone, Reflect, Debug)]
/// Specifies stats per object type.
pub struct ObjectConfig {
    pub physics_material: PhysicsMaterial,
    pub neighbor_radius: f32,
    pub nav_flow_factor: f32,
    pub attack_velocity: f32,
    pub attack_radius: f32,
    pub spawn_velocity: f32,
    pub objective: ObjectiveConfig,
    pub radius: f32,
    pub health: i32,
    pub idle_speed: f32,
    pub spawn_cost: i32,
    pub interactions: InteractionConfigs,
    pub controls: ObjectControlConfig,
}
impl Default for ObjectConfig {
    fn default() -> Self {
        Self {
            physics_material: PhysicsMaterial::default(),
            neighbor_radius: 256.0,
            nav_flow_factor: 1.,
            attack_velocity: 10.,
            attack_radius: 256.,
            spawn_velocity: 2.0,
            objective: ObjectiveConfig::default(),
            radius: 10.0,
            health: 1,
            idle_speed: 0.5,
            spawn_cost: 4,
            interactions: InteractionConfigs({
                let mut interactions = HashMap::new();
                for object in all::<Object>() {
                    interactions.insert(object, InteractionConfig::default());
                }
                interactions
            }),
            controls: ObjectControlConfig::default(),
        }
    }
}
impl ObjectConfig {
    pub fn is_colliding(&self, other: &Self, distance_squared: f32) -> bool {
        distance_squared <= self.radius * self.radius + other.radius * other.radius
    }
    pub fn in_radius(&self, distance_squared: f32) -> bool {
        distance_squared <= self.neighbor_radius * self.neighbor_radius
    }
}

impl ObjectConfigs {
    /// Setup object config.
    pub fn setup(mut configs: ResMut<ObjectConfigs>) {
        // Initialize defaults
        for object in all::<Object>() {
            let config = configs.entry(object).or_insert(default());
            for other in all::<Object>() {
                config.interactions.entry(other).or_insert(default());
            }
        }
    }
}
