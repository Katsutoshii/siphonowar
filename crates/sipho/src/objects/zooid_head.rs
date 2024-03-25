use crate::prelude::*;
use bevy::utils::{Entry, HashMap};

use super::zooid_worker::ZooidWorker;
use super::Object;
use super::{ObjectCommands, ObjectSpec, Team};
pub struct ZooidHeadPlugin;
impl Plugin for ZooidHeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                (ZooidHead::spawn, ZooidHead::spawn_zooids).in_set(SystemStage::Spawn),
                NearestZooidHead::update.in_set(SystemStage::PreCompute),
            )
                .in_set(GameStateSet::Running),
        );
    }
}

/// State for a head.
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct ZooidHead;
impl ZooidHead {
    pub fn spawn(
        mut commands: ObjectCommands,
        config: Res<TeamConfig>,
        obj_config: Res<ObjectConfigs>,
        mut control_events: EventReader<ControlEvent>,
        query: Query<(&ZooidWorker, Entity, &GridEntity, &Selected)>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::SpawnHead) {
                commands.spawn(ObjectSpec {
                    object: Object::Head,
                    position: control_event.position,
                    team: config.player_team,
                    ..default()
                });
            }
            if control_event.is_pressed(ControlAction::Fuse) {
                info!("Fusing!");
                let mut killable = vec![];
                for (_, entity, grid_entity, selected) in query.iter() {
                    if selected.is_selected() {
                        killable.push((entity, grid_entity));
                    }
                    if killable.len() >= obj_config.get(&Object::Head).unwrap().spawn_cost as usize
                    {
                        commands.spawn(ObjectSpec {
                            object: Object::Head,
                            position: control_event.position,
                            team: config.player_team,
                            ..default()
                        });
                        for (entity, grid_entity) in killable.iter() {
                            commands.despawn(*entity, **grid_entity);
                        }
                        break;
                    }
                }
            }
        }
    }

    /// System to spawn zooids on Z key.
    pub fn spawn_zooids(
        mut query: Query<(
            &Self,
            Entity,
            &GlobalTransform,
            &Velocity,
            &Team,
            &Selected,
            &mut Consumer,
        )>,
        mut commands: ObjectCommands,
        configs: Res<ObjectConfigs>,
        mut control_events: EventReader<ControlEvent>,
    ) {
        let config = configs.get(&Object::Worker).unwrap();
        for control_event in control_events.read() {
            if !control_event.is_pressed(ControlAction::SpawnZooid) {
                continue;
            }
            for (_head, head_id, transform, velocity, team, selected, mut consumer) in
                query.iter_mut()
            {
                if !selected.is_selected() {
                    continue;
                }
                if consumer.consumed > 0 {
                    consumer.consumed -= 1;
                    let zindex = zindex::ZOOIDS_MIN + zindex::ZOOIDS_MAX - zindex::ZOOIDS_MIN;
                    let velocity: Vec2 = Vec2::Y * config.spawn_velocity + velocity.0;
                    commands.spawn(ObjectSpec {
                        position: transform.translation().xy() + velocity,
                        velocity: Some(Velocity(velocity)),
                        team: *team,
                        zindex,
                        objectives: Objectives::new(Objective::FollowEntity(head_id)),
                        ..default()
                    });
                }
            }
            if control_event.is_pressed(ControlAction::SpawnZooid) {
                for (_head, head_id, transform, velocity, team, _, _) in query.iter() {
                    let num_zooids = 1;
                    for i in 1..=num_zooids {
                        let zindex = zindex::ZOOIDS_MIN
                            + (i as f32) * 0.00001 * (zindex::ZOOIDS_MAX - zindex::ZOOIDS_MIN);
                        let velocity: Vec2 = Vec2::Y * config.spawn_velocity + velocity.0;
                        commands.spawn(ObjectSpec {
                            position: transform.translation().xy() + velocity,
                            velocity: Some(Velocity(velocity)),
                            team: *team,
                            zindex,
                            objectives: Objectives::new(Objective::FollowEntity(head_id)),
                            ..default()
                        });
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct NearestZooidHead {
    pub entity: Option<Entity>,
}
impl NearestZooidHead {
    /// Each worker tracks its nearest head.
    pub fn update(
        mut query: Query<(&mut Self, &Team, &GlobalTransform), Without<ZooidHead>>,
        heads: Query<(Entity, &Team, &GlobalTransform), With<ZooidHead>>,
    ) {
        let mut team_heads: HashMap<Team, HashMap<Entity, Vec2>> = HashMap::default();
        for (entity, team, transform) in &heads {
            let entry = match team_heads.entry(*team) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(HashMap::default()),
            };
            entry.insert(entity, transform.translation().xy());
        }
        for (mut nearest_head, team, transform) in &mut query {
            if let Some(heads) = team_heads.get(team) {
                if let Some(entity) = nearest_head.entity {
                    if !heads.contains_key(&entity) {
                        nearest_head.entity = None;
                    }
                } else {
                    let position = transform.translation().xy();
                    let (entity, _) = heads
                        .iter()
                        .max_by(|(_, p1), (_, p2)| {
                            let d1 = position.distance_squared(**p1);
                            let d2 = position.distance_squared(**p2);
                            d1.partial_cmp(&d2).unwrap()
                        })
                        .unwrap();
                    nearest_head.entity = Some(*entity);
                }
            }
        }
    }
}
