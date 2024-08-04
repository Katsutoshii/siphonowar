use crate::prelude::*;
use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};
use bevy_bundletree::*;

use super::{neighbors::NeighborsBundle, object_tree::ObjectTree};

#[derive(Default, Debug)]
pub struct ObjectSpec {
    pub object: Object,
    pub position: Vec2,
    pub team: Team,
    pub velocity: Option<Velocity>,
    pub objectives: Objectives,
    pub selected: bool,
}

#[derive(Bundle, Default)]
pub struct ObjectBundle {
    pub object: Object,
    pub team: Team,
    pub physics: PhysicsBundle,
    pub objectives: Objectives,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub health: Health,
    pub neighbors: NeighborsBundle,
    pub attached_to: AttachedTo,
    pub path: PathToHead,
    pub carried_by: CarriedBy,
    pub grid_raycast_target: GridRaycastTarget,
    pub name: Name,
    pub fog_entity: FogEntity,
    pub selectable: Selectable,
}
impl ObjectBundle {
    /// Returns random value [0, 1.)
    fn random_offset(time: &Time) -> f32 {
        time.elapsed().as_secs_f32() / time.wrap_period().as_secs_f32()
    }
    pub fn new(config: &ObjectConfig, spec: ObjectSpec, time: &Time) -> Self {
        let name: &'static str = spec.object.into();
        Self {
            object: spec.object,
            team: spec.team,
            objectives: spec.objectives,
            transform: Transform {
                scale: Vec3::splat(config.radius),
                translation: spec
                    .position
                    .extend(spec.object.zindex() + 0.1 * Self::random_offset(time)),
                ..default()
            },
            physics: PhysicsBundle {
                material: config.physics_material.clone(),
                position: Position(spec.position),
                velocity: spec
                    .velocity
                    .unwrap_or(Velocity(Vec2::ONE) * config.spawn_velocity),
                mass: Mass(1.0),
                ..default()
            },
            health: Health::new(config.health),
            name: Name::new(name),
            neighbors: NeighborsBundle {
                grid_entity: GridEntity {
                    publish_events: true,
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct ObjectCommands<'w, 's> {
    assets: Res<'w, ObjectAssets>,
    pub commands: Commands<'w, 's>,
    configs: Res<'w, ObjectConfigs>,
    parents: Query<'w, 's, &'static Children, Without<Parent>>,
    children: Query<'w, 's, &'static Object, With<Parent>>,
    despawn_events: EventWriter<'w, DespawnEvent>,
    time: Res<'w, Time>,
    obstacles: Res<'w, Grid2<Obstacle>>,
    consumers: Query<'w, 's, &'static mut Consumer>,
}
impl ObjectCommands<'_, '_> {
    pub fn try_consume(&mut self, entity: Entity, n: usize) -> Result<(), Error> {
        if let Ok(mut consumer) = self.consumers.get_mut(entity) {
            if consumer.food_consumed() >= n {
                consumer.spend_food(n, &mut self.commands);
            } else {
                return Err(Error::Default);
            }
        }
        Ok(())
    }
    pub fn spawn(&mut self, spec: ObjectSpec) -> Option<EntityCommands> {
        let config = &self.configs[&spec.object];
        let team_material = self.assets.get_team_material(spec.team);
        if let Some(rowcol) = self.obstacles.to_rowcol(spec.position) {
            if self.obstacles[rowcol] != Obstacle::Empty {
                return None;
            }
        }
        let primary_material = if spec.object == Object::Gem {
            self.assets.crystal_material.clone()
        } else {
            team_material.primary.clone()
        };
        let mesh = self.assets.object_meshes[&spec.object].clone();
        let bundle_tree = ObjectTree::new(
            spec,
            mesh,
            team_material.background,
            primary_material,
            config,
            &self.time,
        );
        Some(self.commands.spawn_tree(bundle_tree))
    }

    /// Queue a despawn event for this entity.
    pub fn deferred_despawn(&mut self, entity: Entity) {
        if let Ok(children) = self.parents.get(entity) {
            for &child in children {
                if self.children.get(child).is_ok() {
                    self.commands.entity(child).remove_parent_in_place();
                }
            }
        }
        self.despawn_events.send(DespawnEvent(entity));
    }
}
