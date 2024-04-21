use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_hanabi::prelude::*;

/// Plugin for effects.
pub struct FireworkPlugin;
impl Plugin for FireworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_event::<FireworkSpec>()
            .init_resource::<EffectAssets>()
            .init_resource::<ParticleEffectPool<FIREWORK_COLOR_BLUE>>()
            .init_resource::<ParticleEffectPool<FIREWORK_COLOR_RED>>()
            .init_resource::<ParticleEffectPool<FIREWORK_COLOR_WHITE>>()
            .add_systems(Startup, FireworkCommands::startup)
            .add_systems(
                FixedUpdate,
                FireworkSpec::update
                    .in_set(GameStateSet::Running)
                    .in_set(FixedUpdateStage::PostPhysics),
            );
    }
}

fn get_standard_color_gradient(color: Color) -> Gradient<Vec4> {
    let mut color_gradient = Gradient::new();
    let color = color.rgba_to_vec4();
    color_gradient.add_key(0.0, color + Vec4::new(0.0, 0.0, 0.0, 0.5));
    color_gradient.add_key(0.1, color);
    color_gradient.add_key(0.7, color * 0.5);
    color_gradient.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 1.0));
    color_gradient
}

pub fn firework_effect(color_gradient: Gradient<Vec4>, n: f32) -> EffectAsset {
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
    fireworks: [Handle<EffectAsset>; FireworkColor::COUNT],
}
impl FromWorld for EffectAssets {
    fn from_world(world: &mut World) -> Self {
        let mut assets = world.get_resource_mut::<Assets<EffectAsset>>().unwrap();
        Self {
            fireworks: [
                FireworkColor::Blue,
                FireworkColor::Red,
                FireworkColor::White,
            ]
            .map(|color| {
                assets.add(firework_effect(
                    get_standard_color_gradient(color.into()),
                    4.,
                ))
            }),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FireworkColor {
    Blue,
    Red,
    White,
}
pub const FIREWORK_COLOR_BLUE: u8 = FireworkColor::Blue as u8;
pub const FIREWORK_COLOR_RED: u8 = FireworkColor::Red as u8;
pub const FIREWORK_COLOR_WHITE: u8 = FireworkColor::White as u8;
impl FireworkColor {
    pub const COUNT: usize = 3;
}
impl From<FireworkColor> for Color {
    fn from(value: FireworkColor) -> Self {
        match value {
            FireworkColor::Blue => Team::COLORS[Team::Blue as usize],
            FireworkColor::Red => Team::COLORS[Team::Red as usize],
            FireworkColor::White => Color::WHITE,
        }
    }
}
impl From<Team> for FireworkColor {
    fn from(value: Team) -> Self {
        match value {
            Team::None => Self::White,
            Team::Blue => Self::Blue,
            Team::Red => Self::Red,
        }
    }
}

/// Describes a firework to create.
#[derive(Debug, Event, Clone)]
pub struct FireworkSpec {
    pub color: FireworkColor,
    pub position: Vec3,
    pub size: VfxSize,
}
impl FireworkSpec {
    pub fn update(mut events: EventReader<Self>, mut commands: FireworkCommands) {
        for event in events.read() {
            commands.make_fireworks(event);
        }
    }
}

pub const POOL_SIZE: usize = 128;
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ParticleEffectPool<const T: u8>(EntityPool<POOL_SIZE>);

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct FireworkCommands<'w, 's> {
    commands: Commands<'w, 's>,
    assets: ResMut<'w, EffectAssets>,
    blue_pool: ResMut<'w, ParticleEffectPool<FIREWORK_COLOR_BLUE>>,
    red_pool: ResMut<'w, ParticleEffectPool<FIREWORK_COLOR_RED>>,
    white_pool: ResMut<'w, ParticleEffectPool<FIREWORK_COLOR_WHITE>>,
    effects: Query<'w, 's, (&'static mut Transform, &'static mut EffectSpawner)>,
}

impl FireworkCommands<'_, '_> {
    pub fn startup(mut commands: FireworkCommands) {
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
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(
                        commands.assets.fireworks[FireworkColor::Blue as usize].clone(),
                    ),
                    ..default()
                })
                .set_parent_in_place(blue_parent)
                .id();
            commands.red_pool[i] = commands
                .commands
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(
                        commands.assets.fireworks[FireworkColor::Red as usize].clone(),
                    ),
                    ..default()
                })
                .set_parent_in_place(red_parent)
                .id();
            commands.white_pool[i] = commands
                .commands
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(
                        commands.assets.fireworks[FireworkColor::White as usize].clone(),
                    ),
                    ..default()
                })
                .set_parent_in_place(red_parent)
                .id();
        }
    }
    pub fn make_fireworks(&mut self, spec: &FireworkSpec) {
        let count = match spec.size {
            VfxSize::Small => 1,
            VfxSize::Medium => 2,
        };
        for _ in 0..count {
            let entity = match spec.color {
                FireworkColor::Blue => self.blue_pool.take(),
                FireworkColor::Red => self.red_pool.take(),
                FireworkColor::White => self.white_pool.take(),
            };
            let (mut transform, mut spawner) = self.effects.get_mut(entity).unwrap();
            transform.translation = spec.position;
            spawner.set_active(true);
            spawner.reset();
        }
    }
}
