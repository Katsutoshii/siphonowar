use crate::prelude::*;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Elastic {
    entities: [Entity; 2],
}

impl Elastic {
    pub fn tie_together(
        mut commands: Commands,
        mut control_events: EventReader<ControlEvent>,
        query: Query<(Entity, &Selected)>,
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
                    if let [a, b] = slice {
                        commands.spawn(Elastic { entities: [*a, *b] });
                    }
                }
            }
        }
    }
    pub fn update(
        mut commands: Commands,
        elastic_query: Query<(Entity, &Elastic)>,
        worker_query: Query<(Entity, &GlobalTransform)>,
        mut accel_query: Query<&mut Acceleration>,
        mut gizmos: Gizmos,
    ) {
        for (cord_entity, cord) in elastic_query.iter() {
            if let (Ok((entity1, transform1)), Ok((entity2, transform2))) = (
                worker_query.get(cord.entities[0]),
                worker_query.get(cord.entities[1]),
            ) {
                let delta = transform2.translation().xy() - transform1.translation().xy();
                let direction = delta.normalize_or_zero();
                let force = delta.length_squared() * 0.0005;
                *accel_query.get_mut(entity1).unwrap() += Acceleration(direction * force);
                *accel_query.get_mut(entity2).unwrap() -= Acceleration(direction * force);
                gizmos.line_2d(
                    transform1.translation().xy(),
                    transform2.translation().xy(),
                    Color::GRAY,
                );
            } else {
                commands.entity(cord_entity).despawn();
            }
        }
    }
}

pub struct ElasticPlugin;
impl Plugin for ElasticPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Elastic>().add_systems(
            FixedUpdate,
            (Elastic::tie_together, Elastic::update)
                .in_set(SystemStage::PostCompute)
                .in_set(GameStateSet::Running),
        );
    }
}
