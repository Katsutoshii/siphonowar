use bevy::utils::smallvec::SmallVec;
use sipho_core::grid::fog::FogConfig;

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

#[derive(Component, Debug, Default, DerefMut, Deref)]
pub struct AttachedTo(pub SmallVec<[Entity; 8]>);

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct Elastic((Entity, Entity));
impl Default for Elastic {
    fn default() -> Self {
        Self((Entity::PLACEHOLDER, Entity::PLACEHOLDER))
    }
}
impl Elastic {
    pub fn first(&self) -> Entity {
        self.0 .0
    }
    pub fn second(&self) -> Entity {
        self.0 .1
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
        mut query: Query<(Entity, &Selected, &mut AttachedTo)>,
        assets: Res<ObjectAssets>,
        config: Res<FogConfig>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::TieWorkers) {
                // Collect entities to tie together.
                let mut entities = vec![];
                let mut attachments = vec![];
                for (entity, selected, attached_to) in query.iter_mut() {
                    match selected {
                        Selected::Selected { .. } => {
                            entities.push(entity);
                            attachments.push(attached_to);
                        }
                        Selected::Unselected => {}
                    }
                }
                if entities.is_empty() {
                    return;
                }
                for i in 0..entities.len() - 1 {
                    let pair = (entities[i], entities[i + 1]);
                    if attachments[i].contains(&pair.1) {
                        continue;
                    }
                    commands.spawn(ElasticBundle {
                        elastic: Elastic(pair),
                        pbr: PbrBundle {
                            mesh: assets.connector_mesh.clone(),
                            material: assets.get_team_material(config.player_team).background,
                            ..default()
                        },
                    });
                    attachments[i].push(pair.1);
                    attachments[i + 1].push(pair.0);
                }
                break;
            }
        }
    }
    pub fn update(
        mut commands: Commands,
        mut elastic_query: Query<(Entity, &Elastic, &mut Transform)>,
        worker_query: Query<(Entity, &GlobalTransform)>,
        mut accel_query: Query<&mut Acceleration>,
        mut attachments: Query<&mut AttachedTo>,
    ) {
        for (entity, elastic, mut transform) in elastic_query.iter_mut() {
            if let (Ok((entity1, transform1)), Ok((entity2, transform2))) = (
                worker_query.get(elastic.first()),
                worker_query.get(elastic.second()),
            ) {
                let position1 = transform1.translation().xy();
                let position2 = transform2.translation().xy();

                let delta = position2 - position1;
                let direction = delta.normalize_or_zero();
                let magnitude = delta.length();
                let force = magnitude * magnitude * 0.0001;
                *accel_query.get_mut(entity1).unwrap() += Acceleration(direction * force);
                *accel_query.get_mut(entity2).unwrap() -= Acceleration(direction * force);

                // Set transform.
                let width = 4.;
                let depth = transform1.translation().z;
                transform.translation = ((position1 + position2) / 2.).extend(depth);
                transform.scale = Vec3::new(magnitude / 2., width, width);
                transform.rotation = Quat::from_axis_angle(Vec3::Z, delta.to_angle())
            } else {
                // Clean up invalid connections.
                commands.entity(entity).despawn();
                if let Ok(mut attached_to) = attachments.get_mut(elastic.first()) {
                    attached_to.retain(|&mut x| x != elastic.second())
                }
                if let Ok(mut attached_to) = attachments.get_mut(elastic.first()) {
                    attached_to.retain(|&mut x| x != elastic.second())
                }
            }
        }
    }
}
