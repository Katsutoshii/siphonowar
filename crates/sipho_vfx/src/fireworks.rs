use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_hanabi::prelude::*;

/// Plugin for effects.
pub struct FireworkPlugin;
impl Plugin for FireworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .init_resource::<EffectAssets>()
            .init_resource::<ParticleEffectPool<TEAM_BLUE>>()
            .init_resource::<ParticleEffectPool<TEAM_RED>>()
            .add_systems(Startup, EffectCommands::startup);
    }
}

// pub const FIREWORK_COLOR: Color = Color::rgb(1.0, 1.0, 0.25);

fn color_gradient_from_team(team: Team) -> Gradient<Vec4> {
    let mut color_gradient = Gradient::new();
    match team {
        Team::Blue | Team::None => {
            let color = Color::TEAL.rgba_to_vec4();
            color_gradient.add_key(0.0, color + Vec4::new(0.0, 0.0, 0.0, 0.5));
            color_gradient.add_key(0.1, color);
            color_gradient.add_key(0.7, color * 0.5);
            color_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
        }
        Team::Red => {
            let color = Color::TOMATO.rgba_to_vec4();
            color_gradient.add_key(0.0, color + Vec4::new(0.0, 0.0, 0.0, 0.5));
            color_gradient.add_key(0.1, color);
            color_gradient.add_key(0.7, color * 0.5);
            color_gradient.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 1.0));
        }
    };
    color_gradient
}

pub fn firework_effect(team: Team, n: f32) -> EffectAsset {
    let color_gradient = color_gradient_from_team(team);

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(5.0 * n.powf(0.25)));
    size_gradient1.add_key(0.3, Vec2::splat(2.0));
    size_gradient1.add_key(0.9, Vec2::splat(1.5));
    size_gradient1.add_key(1.0, Vec2::splat(0.5));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.5).uniform(writer.lit(0.7)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add drag to make particles slow down a bit after the initial explosion
    // The more particles we have, the less draw we want.
    let drag = writer.lit(1. / n).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(90.)).expr(),
    };

    EffectAsset::new(
        n as u32,
        Spawner::once(n.into(), true).with_starts_active(false),
        writer.finish(),
    )
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
}
impl FromWorld for EffectAssets {
    fn from_world(world: &mut World) -> Self {
        let mut assets = world.get_resource_mut::<Assets<EffectAsset>>().unwrap();
        Self {
            fireworks: Team::ALL.map(|team| assets.add(firework_effect(team, 4.))),
        }
    }
}

/// Describes a firework to create.
#[derive(Debug)]
pub struct FireworkSpec {
    pub team: Team,
    pub transform: Transform,
    pub size: VfxSize,
}

pub const POOL_SIZE: usize = 128;
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ParticleEffectPool<const T: u8>(EntityPool<POOL_SIZE>);

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct EffectCommands<'w, 's> {
    commands: Commands<'w, 's>,
    assets: ResMut<'w, EffectAssets>,
    blue_pool: ResMut<'w, ParticleEffectPool<{ Team::Blue as u8 }>>,
    red_pool: ResMut<'w, ParticleEffectPool<{ Team::Red as u8 }>>,
    effects: Query<'w, 's, (&'static mut Transform, &'static mut EffectSpawner)>,
}

impl EffectCommands<'_, '_> {
    pub fn startup(mut commands: EffectCommands) {
        let blue_parent = commands
            .commands
            .spawn((Name::new("ParticlePool<Blue>"), SpatialBundle::default()))
            .id();
        let red_parent = commands
            .commands
            .spawn((Name::new("ParticlePool<Red>"), SpatialBundle::default()))
            .id();
        for i in 0..POOL_SIZE {
            commands.blue_pool[i] = commands
                .commands
                .spawn((
                    Name::new("Particles"),
                    ParticleEffectBundle {
                        effect: ParticleEffect::new(commands.assets.fireworks[Team::Blue].clone()),
                        ..default()
                    },
                ))
                .set_parent_in_place(blue_parent)
                .id();

            commands.red_pool[i] = commands
                .commands
                .spawn((
                    Name::new("Particles"),
                    ParticleEffectBundle {
                        effect: ParticleEffect::new(commands.assets.fireworks[Team::Red].clone()),
                        ..default()
                    },
                ))
                .set_parent_in_place(red_parent)
                .id();
        }
    }
    pub fn make_fireworks(&mut self, spec: FireworkSpec) {
        let count = match spec.size {
            VfxSize::Tiny | VfxSize::Small => 1,
            VfxSize::Medium => 2,
        };
        for _ in 0..count {
            let entity = match spec.team {
                Team::None | Team::Blue => self.blue_pool.take(),
                Team::Red => self.red_pool.take(),
            };
            let (mut transform, mut spawner) = self.effects.get_mut(entity).unwrap();
            *transform = spec.transform;
            spawner.set_active(true);
            spawner.reset();
        }
    }
}
