use crate::prelude::*;

use bevy_hanabi::prelude::*;
pub struct BubblesPlugin;
impl Plugin for BubblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
    query: Query<Entity, With<MainCamera>>,
) {
    let writer = ExprWriter::new();

    let texture_handle: Handle<Image> = asset_server.load("textures/vfx/bubble_transparent.png");
    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(4.).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let accel = writer.lit((Vec3::Y + Vec3::Z) * 20.).expr();
    let update_accel = AccelModifier::new(accel);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(1500.).expr(),
        dimension: ShapeDimension::Volume,
        axis: writer.lit(Vec3::Z).expr(),
    };

    let effect = effects.add(
        EffectAsset::new(
            2048,
            Spawner::rate(CpuValue::Uniform((5., 10.))),
            writer.finish(),
        )
        .with_name("emit:burst")
        .init(init_pos)
        .init(init_age)
        .init(init_lifetime)
        .update(update_accel)
        .render(ParticleTextureModifier {
            texture: texture_handle.clone(),
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec4::new(0.9, 1.0, 1.0, 0.0));
                gradient.add_key(0.5, Vec4::new(0.9, 1.0, 1.0, 0.2));
                gradient.add_key(1.0, Vec4::new(0.9, 1.0, 1.0, 0.0));
                gradient
            },
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec2::splat(10.0));
                gradient.add_key(1.0, Vec2::splat(5.0));
                gradient
            },
            screen_space_size: false,
        }),
    );

    let camera_id = query.single();
    let spawner_id = commands
        .spawn((
            Name::new("BubbleEmitter"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform::from_translation(Vec3::new(0., 0., -zindex::CAMERA)),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(camera_id).add_child(spawner_id);
}
