use std::f32::consts::PI;

use bevy::ecs::system::SystemParam;
use bevy_hanabi::ParticleEffect;

use crate::prelude::*;

pub struct LightningPlugin;
impl Plugin for LightningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightningAssets>()
            .init_resource::<LightningEffectPool>()
            .add_systems(
                FixedUpdate,
                Lightning::update
                    .in_set(GameStateSet::Running)
                    .in_set(SystemStage::PostCompute),
            )
            .add_systems(Startup, LightningCommands::setup);
    }
}

#[derive(SystemParam)]
pub struct LightningCommands<'w, 's> {
    commands: Commands<'w, 's>,
    pool: ResMut<'w, LightningEffectPool>,
    query: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Visibility,
            &'static mut Lightning,
        ),
        Without<ParticleEffect>,
    >,
    assets: Res<'w, LightningAssets>,
}

impl LightningCommands<'_, '_> {
    fn setup(mut commands: LightningCommands) {
        let parent = commands
            .commands
            .spawn((Name::new("LightningEffectPool"), SpatialBundle::default()))
            .id();
        for i in 0..POOL_SIZE {
            commands.pool[i] = commands
                .commands
                .spawn(LightningBundle {
                    lightning: Lightning {
                        timer: Timer::from_seconds(0.1, TimerMode::Once),
                    },
                    pbr: PbrBundle {
                        mesh: commands.assets.lightning_mesh.clone(),
                        material: commands.assets.lightning_material.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                })
                .with_children(|parent| {
                    let point_light = PointLight {
                        color: Color::WHITE,
                        intensity: 100_000_000.,
                        range: 1000.,
                        ..default()
                    };
                    let depth = 3.;
                    for offset in [-0.7, -0.2, 0.2, 0.7] {
                        parent.spawn(PointLightBundle {
                            point_light,
                            transform: Transform {
                                translation: offset * Vec3::X + Vec3::Z * -depth,
                                ..default()
                            },
                            ..default()
                        });
                    }
                })
                .set_parent_in_place(parent)
                .id();
        }
    }
    pub fn make_lightning(&mut self, start: Vec2, end: Vec2, depth: f32) {
        let delta = end - start;
        let entity = self.pool.take();
        let (mut transform, mut visibility, mut lightning) = self.query.get_mut(entity).unwrap();

        // Set transform.
        let width = 12.;
        let magnitude = delta.length();
        *transform = Transform {
            translation: ((start + end) / 2.).extend(depth),
            scale: Vec3::new(magnitude / 2., width, width),
            rotation: Quat::from_rotation_z(delta.to_angle()) * Quat::from_rotation_x(start.x % PI),
        };
        *visibility = Visibility::Visible;
        lightning.timer.reset();
    }
}

/// Schedule despawn for a particle.
#[derive(Component, Default)]
pub struct Lightning {
    pub timer: Timer,
}
impl Lightning {
    pub fn update(
        mut query: Query<(&mut Lightning, &mut Visibility, &mut Transform)>,
        time: Res<Time>,
    ) {
        for (mut lightning, mut visibility, mut transform) in &mut query {
            if *visibility == Visibility::Hidden {
                continue;
            }
            lightning.timer.tick(time.delta());
            if lightning.timer.finished() {
                *visibility = Visibility::Hidden;
            }
            transform.rotation *= Quat::from_rotation_x(PI / 4.);
        }
    }
}

#[derive(Bundle, Default)]
pub struct LightningBundle {
    pub lightning: Lightning,
    pub pbr: PbrBundle,
}

pub const POOL_SIZE: usize = 128;
#[derive(Resource, Default, Deref, DerefMut)]
pub struct LightningEffectPool(EntityPool<POOL_SIZE>);

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct LightningAssets {
    pub lightning_material: Handle<StandardMaterial>,
    pub lightning_mesh: Handle<Mesh>,
}
impl FromWorld for LightningAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            lightning_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    emissive: Color::WHITE,
                    diffuse_transmission: 1.0,
                    ..default()
                })
            },
            lightning_mesh: {
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                asset_server.load("models/lightning/lightning.glb#Mesh0/Primitive0")
            },
        }
    }
}
