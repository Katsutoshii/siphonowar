use crate::prelude::*;

use super::ObjectAssets;

pub struct ElasticPlugin;
impl Plugin for ElasticPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Elastic>().add_systems(
            FixedUpdate,
            (Elastic::tie_together, Elastic::update)
                .chain()
                .in_set(SystemStage::PostCompute)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Elastic {
    entities: [Entity; 2],
}
impl Default for Elastic {
    fn default() -> Self {
        Self {
            entities: [Entity::PLACEHOLDER, Entity::PLACEHOLDER],
        }
    }
}

#[derive(Bundle, Default)]
pub struct ElasticBundle {
    pub elastic: Elastic,
    pub pbr: PbrBundle,
}
impl Elastic {
    pub fn tie_together(
        mut commands: Commands,
        mut control_events: EventReader<ControlEvent>,
        query: Query<((Entity, &Team), &Selected)>,
        assets: Res<ObjectAssets>,
    ) {
        let mut entity_set = vec![];
        for (entity, selected) in query.iter() {
            match selected {
                Selected::Selected { .. } => {
                    entity_set.push(entity);
                }
                Selected::Unselected => {}
            }
        }
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::TieWorkers) {
                for slice in entity_set.windows(2) {
                    if let [(a, &a_team), (b, &b_team)] = slice {
                        if a_team != b_team {
                            continue;
                        }
                        commands.spawn(ElasticBundle {
                            elastic: Elastic { entities: [*a, *b] },
                            pbr: PbrBundle {
                                mesh: assets.connector_mesh.clone(),
                                material: assets.get_team_material(a_team).background,
                                ..default()
                            },
                        });
                    }
                }
            }
        }
    }
    pub fn update(
        mut commands: Commands,
        mut elastic_query: Query<(Entity, &Elastic, &mut Transform)>,
        worker_query: Query<(Entity, &GlobalTransform)>,
        mut accel_query: Query<&mut Acceleration>,
    ) {
        for (entity, elastic, mut transform) in elastic_query.iter_mut() {
            if let (Ok((entity1, transform1)), Ok((entity2, transform2))) = (
                worker_query.get(elastic.entities[0]),
                worker_query.get(elastic.entities[1]),
            ) {
                let position1 = transform1.translation().xy();
                let position2 = transform2.translation().xy();

                let delta = position2 - position1;
                let direction = delta.normalize_or_zero();
                let magnitude = delta.length();
                let force = magnitude * magnitude * 0.0005;
                *accel_query.get_mut(entity1).unwrap() += Acceleration(direction * force);
                *accel_query.get_mut(entity2).unwrap() -= Acceleration(direction * force);

                // Set transform.
                let width = 4.;
                let depth = transform1.translation().z;
                transform.translation = ((position1 + position2) / 2.).extend(depth);
                transform.scale = Vec3::new(magnitude / 2., width, width);
                transform.rotation = Quat::from_axis_angle(Vec3::Z, delta.to_angle())
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
