use crate::prelude::*;
use std::time::Duration;

pub struct ShockAttackerPlugin;
impl Plugin for ShockAttackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShockAttacker>().add_systems(
            FixedUpdate,
            ShockAttacker::update
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockAttackerState {
    Attacking,
    Searching,
}

/// Dash attacker does a periodic dash towards the target.
/// When attacking,
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct ShockAttacker {
    pub state: ShockAttackerState,
}
impl Default for ShockAttacker {
    fn default() -> Self {
        Self {
            state: ShockAttackerState::Searching,
        }
    }
}
impl ShockAttacker {
    pub const ATTACK_COOLDOWN: Duration = Duration::from_millis(2500);

    pub fn reset(&mut self, entity: Entity, commands: &mut Commands) {
        self.state = ShockAttackerState::Searching;
        commands
            .entity(entity)
            .insert(Cooldown::new(Self::ATTACK_COOLDOWN));
    }

    /// Check cooldown timers and accelerate when in state Attacking.
    /// Stop attacking after the first hit.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut query: Query<
            (
                Entity,
                &Object,
                &mut Velocity,
                &Navigator,
                &mut ShockAttacker,
                &Position,
                &EnemyNeighbors,
            ),
            (Without<Cooldown>, Without<Stunned>),
        >,
        configs: Res<ObjectConfigs>,
        mut damage_events: EventWriter<DamageEvent>,
        mut lightning: LightningCommands,
        mut firework_events: EventWriter<FireworkSpec>,
        mut audio: EventWriter<AudioEvent>,
        mut commands: Commands,
    ) {
        for (entity, object, mut velocity, navigator, mut attacker, position, enemies) in
            query.iter_mut()
        {
            let config = configs.get(object).unwrap();
            let delta = navigator.target - position.0;

            match attacker.state {
                ShockAttackerState::Searching => {
                    let distance_squared = delta.length_squared();
                    if distance_squared < config.attack_radius * config.attack_radius {
                        attacker.state = ShockAttackerState::Attacking;
                    }
                }
                ShockAttackerState::Attacking => {
                    if let Some(enemy) = enemies.first() {
                        *velocity = Velocity::ZERO;
                        attacker.reset(entity, &mut commands);
                        let interaction = config.interactions.get(&enemy.object).unwrap();
                        damage_events.send(DamageEvent {
                            damager: entity,
                            damaged: enemy.entity,
                            amount: interaction.damage_amount,
                            velocity: *velocity,
                            stun: true,
                        });
                        lightning.make_lightning(position.0, navigator.target, zindex::ZOOIDS_MAX);
                        firework_events.send(FireworkSpec {
                            position: navigator.target.extend(zindex::ZOOIDS_MAX),
                            color: FireworkColor::White,
                            size: VfxSize::Small,
                        });
                        audio.send(AudioEvent {
                            sample: AudioSample::RandomZap,
                            position: Some(position.0),
                            ..default()
                        });
                    }
                }
            }
        }
    }
}
