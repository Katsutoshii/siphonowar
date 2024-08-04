use crate::prelude::*;
use bevy::color::palettes::css::ALICE_BLUE;
use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

/// Plugin to manage a virtual cursor.
pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorAssets>()
            .add_systems(Startup, Cursor::startup)
            .add_systems(
                PreUpdate,
                Cursor::update.in_set(FixedUpdateStage::AccumulateForces),
            );
    }
}

#[derive(SystemParam)]
pub struct CursorParam<'w, 's> {
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<MainCamera>>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}
impl CursorParam<'_, '_> {
    /// Returns the world position of the cursor.
    pub fn ray3d(&self) -> Option<Ray3d> {
        let (camera, camera_transform) = self.camera.single();
        let window = self.window.single();
        let cursor_position = window.cursor_position()?;
        camera.viewport_to_world(camera_transform, cursor_position)
    }
}

#[derive(Component, Debug, Default)]
pub struct Cursor;
impl Cursor {
    ///Spawn the cursor.
    pub fn startup(mut commands: Commands, assets: Res<CursorAssets>) {
        let style = Style {
            width: Val::Px(32.0),
            position_type: PositionType::Absolute,
            ..default()
        };
        commands.spawn((
            ImageBundle {
                style: style.clone(),
                image: UiImage::new(assets.cursor.clone()),
                z_index: ZIndex::Global(i32::MAX),
                transform: Transform {
                    scale: Vec3::splat(0.7),
                    ..default()
                },
                ..default()
            },
            Cursor,
        ));
    }
    pub fn update(
        mut cursor: Query<(&mut Style, &mut UiImage), With<Self>>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
        control_state: Res<ControlState>,
        assets: Res<CursorAssets>,
    ) {
        let window = window.single_mut();
        if let Some(cursor_pixel_position) = window.cursor_position() {
            let (mut style, mut image) = cursor.single_mut();
            style.left = Val::Px(cursor_pixel_position.x - 2.0);
            style.top = Val::Px(cursor_pixel_position.y - 4.0);

            if control_state.is_changed() {
                image.texture = match control_state.mode {
                    ControlMode::Normal => assets.cursor.clone(),
                    ControlMode::Attack => assets.attack.clone(),
                }
            }
        }
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct CursorAssets {
    pub mesh: Handle<Mesh>,
    pub cursor: Handle<Image>,
    pub attack: Handle<Image>,
    pub blue_material: Handle<StandardMaterial>,
}
impl FromWorld for CursorAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = Self {
            mesh: world.append_asset(Mesh::from(RegularPolygon {
                circumcircle: Circle {
                    radius: 2f32.sqrt() / 2.,
                },
                sides: 3,
            })),
            cursor: world.load_asset("textures/cursor/paper-arrow-s.png"),
            attack: world.load_asset("textures/cursor/plain-dagger-s.png"),
            blue_material: world.append_asset(StandardMaterial::from(Color::from(
                ALICE_BLUE.with_alpha(0.5),
            ))),
        };
        let mut load_state = world.get_resource_mut::<AssetLoadState>().unwrap();
        load_state.track(&assets.cursor);
        load_state.track(&assets.attack);
        assets
    }
}
