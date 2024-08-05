use crate::prelude::*;
use bevy::pbr::NotShadowCaster;

use bevy_heightmap::*;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeightMapPlugin)
            .add_systems(Startup, Terrain::setup)
            .add_systems(OnExit(GameState::Loading), Terrain::setup_obstacles);
    }
}

pub const SCALE: f32 = 1024. * 16.;
pub const HEIGHT: f32 = 256.;
pub const SEALEVEL: f32 = 0.7;

#[derive(Component)]
pub struct Terrain;
impl Terrain {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut load_state: ResMut<AssetLoadState>,
    ) {
        let mesh: Handle<Mesh> = asset_server.load("textures/heightmaps/terrain.hmp.png");
        load_state.track(&mesh);
        commands.spawn((
            Name::new("Terrain"),
            Terrain,
            NotShadowCaster,
            PbrBundle {
                mesh,
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(69 / 4, 84 / 4, 180 / 4),
                    alpha_mode: AlphaMode::Opaque,
                    perceptual_roughness: 1.0,
                    ..default()
                }),
                transform: Transform {
                    translation: Vec2::ZERO.extend(-HEIGHT * SEALEVEL),
                    scale: Vec2::splat(SCALE).extend(HEIGHT),
                    ..default()
                },
                ..default()
            },
        ));
    }

    pub fn setup_obstacles(
        terrain: Query<&Handle<Mesh>, With<Terrain>>,
        meshes: Res<Assets<Mesh>>,
        mut obstacles: ResMut<Grid2<Obstacle>>,
    ) {
        let mesh_handle = terrain.single();
        let mesh: &Mesh = meshes.get(mesh_handle).unwrap();
        let vertex_positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        for position in vertex_positions {
            let position = Vec3::new(position[0] * SCALE, position[1] * SCALE, position[2]);
            if position.z > SEALEVEL {
                if let Some(rowcol) = obstacles.to_rowcol(position.xy()) {
                    obstacles[rowcol] = Obstacle::Full;
                }
            }
        }
    }
}
