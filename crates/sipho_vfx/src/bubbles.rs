use crate::prelude::*;

use bevy_hanabi::prelude::*;
pub struct BubblesPlugin;
impl Plugin for BubblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), BubbleSpawner::setup)
            .add_systems(FixedUpdate, BubbleSpawner::update);
    }
}

#[derive(Component)]
pub struct BubbleSpawner;
impl BubbleSpawner {
    fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut effects: ResMut<Assets<EffectAsset>>,
        camera: Query<&GlobalTransform, With<MainCamera>>,
    ) {
        let writer = ExprWriter::new();

        let texture_handle: Handle<Image> =
            asset_server.load("textures/vfx/bubble_transparent.png");
        let age = writer.lit(0.).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        let lifetime = writer.lit(4.).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

        let accel = (writer.lit((Vec3::Y + Vec3::Z) * 10.)
            + (writer.time() * writer.lit(2.)).sin() * writer.lit(Vec3::X * 40.))
        .expr();
        let update_accel = AccelModifier::new(accel);

        let init_pos = SetPositionCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            radius: writer.lit(1500.).expr(),
            dimension: ShapeDimension::Volume,
            axis: writer.lit(Vec3::Z).expr(),
        };

        let rotation = (writer.rand(ScalarType::Float) * writer.lit(std::f32::consts::TAU)).expr();
        let init_rotation = SetAttributeModifier::new(Attribute::F32_0, rotation);

        // The rotation of the OrientModifier is read from the F32_0 attribute (our
        // per-particle rotation)
        let rotation_attr = writer.attr(Attribute::F32_0).expr();

        let effect: Handle<EffectAsset> = effects.add(
            EffectAsset::new(
                vec![2048],
                Spawner::rate(CpuValue::Uniform((10., 15.))),
                writer.finish(),
            )
            .with_name("emit:burst")
            .init(init_pos)
            .init(init_age)
            .init(init_lifetime)
            .init(init_rotation)
            .update_groups(update_accel, ParticleGroupSet(1))
            .render_groups(
                ParticleTextureModifier {
                    texture: texture_handle.clone(),
                    sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
                },
                ParticleGroupSet(1),
            )
            .render_groups(
                OrientModifier {
                    mode: OrientMode::FaceCameraPosition,
                    rotation: Some(rotation_attr),
                },
                ParticleGroupSet(1),
            )
            .render_groups(
                ColorOverLifetimeModifier {
                    gradient: {
                        let mut gradient = Gradient::new();
                        gradient.add_key(0.0, Vec4::new(0.9, 1.0, 1.0, 0.0));
                        gradient.add_key(0.5, Vec4::new(0.9, 1.0, 1.0, 0.07));
                        gradient.add_key(1.0, Vec4::new(0.9, 1.0, 1.0, 0.0));
                        gradient
                    },
                },
                ParticleGroupSet(1),
            )
            .render_groups(
                SizeOverLifetimeModifier {
                    gradient: {
                        let mut gradient = Gradient::new();
                        gradient.add_key(0.0, Vec2::splat(5.0));
                        gradient.add_key(1.0, Vec2::splat(9.0));
                        gradient
                    },
                    screen_space_size: false,
                },
                ParticleGroupSet(1),
            ),
        );

        let camera_transform = camera.single();
        commands.spawn((
            Name::new("BubbleSpawner"),
            BubbleSpawner,
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform {
                    translation: camera_transform.translation().xy().extend(0.)
                        + Vec3::Y * MainCamera::y_offset(zindex::CAMERA),
                    ..default()
                },
                ..default()
            },
        ));
    }

    pub fn update(
        camera: Query<&GlobalTransform, With<MainCamera>>,
        mut spawners: Query<&mut Transform, With<BubbleSpawner>>,
    ) {
        let camera_transform = camera.single();
        for mut transform in spawners.iter_mut() {
            transform.translation = camera_transform.translation().xy().extend(0.)
                + Vec3::Y * (zindex::CAMERA * MainCamera::THETA.tan());
        }
    }
}
