use crate::prelude::*;

pub struct DamagePlugin;
impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>().add_systems(
            FixedUpdate,
            (Health::update, DamageEvent::update)
                .in_set(SystemStage::Compute)
                .in_set(GameStateSet::Running)
                .after(Object::update_objective),
        );
    }
}

#[derive(Component)]
pub struct Health {
    pub health: i32,
    pub hit_timer: Timer,
}
impl Default for Health {
    fn default() -> Self {
        Self {
            health: 1,
            hit_timer: Timer::from_seconds(0.05, TimerMode::Once),
        }
    }
}
impl Health {
    pub fn new(amount: i32) -> Self {
        Self {
            health: amount,
            ..default()
        }
    }
    pub fn damageable(&self) -> bool {
        self.hit_timer.finished()
    }
    pub fn damage(&mut self, amount: i32) {
        self.health -= amount;
        self.hit_timer = Timer::from_seconds(0.5, TimerMode::Once);
    }
    pub fn update(mut query: Query<&mut Health>, time: Res<Time>) {
        for mut health in query.iter_mut() {
            health.hit_timer.tick(time.delta());
        }
    }
}

#[derive(Event)]
pub struct DamageEvent {
    pub damager: Entity,
    pub damaged: Entity,
    pub amount: i32,
    pub velocity: Velocity,
}
impl DamageEvent {
    pub fn update(
        mut query: Query<(&mut Acceleration, &mut Health, &Team, &GlobalTransform)>,
        mut events: EventReader<DamageEvent>,
        mut effects: EffectCommands,
    ) {
        for event in events.read() {
            // Knock back the damager
            if let Ok((mut acceleration, _health, _team, _transform)) = query.get_mut(event.damager)
            {
                *acceleration +=
                    Acceleration(*event.velocity * if event.amount > 0 { -10. } else { -2. });
            }
            // Reduce health and set off firework for the damaged.
            if let Ok((mut acceleration, mut health, &team, &transform)) =
                query.get_mut(event.damaged)
            {
                health.damage(event.amount);
                if event.amount > 0 {
                    effects.make_fireworks(FireworkSpec {
                        size: if event.amount > 0 {
                            VfxSize::Small
                        } else {
                            VfxSize::Tiny
                        },
                        team,
                        transform: transform.into(),
                    });
                }
                *acceleration +=
                    Acceleration(event.velocity.0) * if event.amount > 0 { 20. } else { 10. };
            }
        }
    }
}
