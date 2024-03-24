use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

/// Plugin to manage a virtual cursor.
pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorAssets>()
            .add_systems(Startup, Cursor::startup)
            .add_systems(PreUpdate, Cursor::update.in_set(SystemStage::Compute));
    }
}

#[derive(SystemParam)]
pub struct CursorParam<'w, 's> {
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<MainCamera>>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}
impl CursorParam<'_, '_> {
    /// Returns the world position of the cursor.
    pub fn world_position(&self) -> Option<Vec2> {
        let (camera, camera_transform) = self.camera.single();
        let window = self.window.single();
        let cursor_position = window.cursor_position()?;
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    }
    pub fn ray3d(&self) -> Option<Ray3d> {
        let world_position = self.world_position()?;
        Some(Ray3d::new(world_position.extend(zindex::CURSOR), -Vec3::Z))
    }
}

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct Cursor;
impl Cursor {
    ///Spawn the cursor.
    pub fn startup(mut commands: Commands, assets: Res<CursorAssets>) {
        info!("Startup");
        let style = Style {
            width: Val::Px(32.0),
            position_type: PositionType::Absolute,
            ..default()
        };
        commands.spawn((
            ImageBundle {
                style: style.clone(),
                image: UiImage::new(assets.cursor.clone()),
                ..default()
            },
            Cursor,
        ));
    }
    pub fn update(
        mut cursor: Query<&mut Style, With<Self>>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
    ) {
        let window = window.single_mut();
        if let Some(cursor_pixel_position) = window.cursor_position() {
            let mut style = cursor.single_mut();
            style.left = Val::Px(cursor_pixel_position.x - 2.0);
            style.top = Val::Px(cursor_pixel_position.y - 0.0);
        }
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct CursorAssets {
    pub mesh: Handle<Mesh>,
    pub cursor: Handle<Image>,
    pub attack: Handle<Image>,
    pub blue_material: Handle<ColorMaterial>,
}
impl FromWorld for CursorAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(RegularPolygon {
                circumcircle: Circle {
                    radius: 2f32.sqrt() / 2.,
                },
                sides: 3,
            }))
        };
        let (cursor, crosshair) = {
            // let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
            let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
            (
                asset_server.load("textures/cursor/paper-arrow-s.png"),
                asset_server.load("textures/cursor/plain-dagger-s.png"),
            )
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            cursor,
            attack: crosshair,
            blue_material: materials.add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.5))),
        }
    }
}
