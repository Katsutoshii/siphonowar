use std::f32::consts::PI;

use crate::prelude::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

/// Plugin to manage a virtual cursor.
pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorAssets>()
            .add_systems(PreUpdate, Cursor::update.in_set(SystemStage::Compute));
    }
}

#[derive(Component, Debug, Default)]
pub struct Cursor;
impl Cursor {
    pub fn update(
        mut cursor: Query<&mut Transform, With<Self>>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
    ) {
        let window = window.single_mut();
        let window_size = Vec2 {
            x: window.physical_width() as f32,
            y: window.physical_height() as f32,
        } / window.scale_factor();

        let mut cursor_transform = cursor.single_mut();

        if let Some(cursor_pixel_position) = window.cursor_position() {
            let cursor_position =
                (cursor_pixel_position - window_size / 2.) * Vec2 { x: 1., y: -1. };
            cursor_transform.translation = cursor_position.extend(cursor_transform.translation.z);
        }
    }

    pub fn bundle(self, assets: &CursorAssets, translation: Vec3) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2 { x: 10., y: 20. }.extend(1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI / 4.))
                    .with_translation(translation),
                material: assets.blue_material.clone(),
                ..default()
            },
            self,
        )
    }

    /// Get the transform for the cursor.
    pub fn ray3d(transform: &GlobalTransform) -> Ray3d {
        Ray3d::new(transform.translation(), -Vec3::Z)
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct CursorAssets {
    pub mesh: Handle<Mesh>,
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
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            blue_material: materials.add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.5))),
        }
    }
}
