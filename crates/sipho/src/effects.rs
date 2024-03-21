use std::ops::Index;

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_hanabi::prelude::*;

/// Plugin for effects.
pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .init_resource::<EffectAssets>()
            .add_systems(FixedUpdate, ScheduleDespawn::despawn);
    }
}

fn color_gradient_from_team(team: Team) -> Gradient<Vec4> {
    let mut color_gradient = Gradient::new();
    match team {
        Team::Blue | Team::None => {
            let color = Color::TEAL.rgba_to_vec4();
            color_gradient.add_key(0.0, color * 1.3);
            color_gradient.add_key(0.1, color);
            color_gradient.add_key(0.9, color * 0.5);
            color_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 2.0, 0.0));
        }
        Team::Red => {
            let color = Color::TOMATO.rgba_to_vec4();
            color_gradient.add_key(0.0, color * 1.3);
            color_gradient.add_key(0.1, color);
            color_gradient.add_key(0.7, color * 0.5);
            color_gradient.add_key(1.0, Vec4::new(2.0, 0.0, 0.0, 0.0));
        }
    };
    color_gradient
}

pub fn firework_effect(team: Team, n: f32) -> EffectAsset {
    let color_gradient = color_gradient_from_team(team);

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(10.0));
    size_gradient1.add_key(0.3, Vec2::splat(7.0));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.5).uniform(writer.lit(0.7)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(5.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(60.)).expr(),
    };

    EffectAsset::new(n as u32, Spawner::once(n.into(), true), writer.finish())
        .with_name("firework")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}

#[derive(Resource)]
pub struct EffectAssets {
    fireworks: [Handle<EffectAsset>; Team::COUNT],
    small_fireworks: [Handle<EffectAsset>; Team::COUNT],
}
impl FromWorld for EffectAssets {
    fn from_world(world: &mut World) -> Self {
        let mut assets = world.get_resource_mut::<Assets<EffectAsset>>().unwrap();
        Self {
            fireworks: Team::ALL.map(|team| assets.add(firework_effect(team, 20.))),
            small_fireworks: Team::ALL.map(|team| assets.add(firework_effect(team, 5.))),
        }
    }
}
impl Index<Team> for [Handle<EffectAsset>; Team::COUNT] {
    type Output = Handle<EffectAsset>;
    fn index(&self, index: Team) -> &Self::Output {
        &self[index as usize]
    }
}

/// Represents size of an effect.
pub enum EffectSize {
    Small,
    Medium,
}
/// Describes a firework to create.
pub struct FireworkSpec {
    pub team: Team,
    pub transform: Transform,
    pub size: EffectSize,
}

/// Schedule despawn for a particle.
#[derive(Component, DerefMut, Deref)]
pub struct ScheduleDespawn(pub Timer);
impl ScheduleDespawn {
    pub fn despawn(
        mut query: Query<(Entity, &mut ScheduleDespawn)>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        for (entity, mut timer) in &mut query {
            timer.tick(time.delta());
            if timer.finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct EffectCommands<'w, 's> {
    assets: ResMut<'w, EffectAssets>,
    commands: Commands<'w, 's>,
}
impl EffectCommands<'_, '_> {
    pub fn make_fireworks(&mut self, spec: FireworkSpec) {
        self.commands.spawn((
            Name::new("firework"),
            ScheduleDespawn(Timer::from_seconds(0.5, TimerMode::Once)),
            ParticleEffectBundle {
                effect: ParticleEffect::new(match spec.size {
                    EffectSize::Small => self.assets.small_fireworks[spec.team].clone(),
                    EffectSize::Medium => self.assets.fireworks[spec.team].clone(),
                }),
                transform: spec.transform,
                ..Default::default()
            },
        ));
    }
}
