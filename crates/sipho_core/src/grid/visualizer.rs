use bevy::render::render_resource::{AsBindGroup, ShaderRef};

use crate::prelude::*;
use bevy::color::palettes::css::{ALICE_BLUE, MIDNIGHT_BLUE};

/// Plugin for visualizing the grid.
/// This plugin reads events from the entity grid and updates the shader's input buffer
/// to light up the cells that have entities.
pub struct GridVisualizerPlugin;
impl Plugin for GridVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<GridVisualizerShaderMaterial>::default())
            .add_systems(
                FixedUpdate,
                (GridVisualizerShaderMaterial::update
                    .after(GridEntity::update)
                    .run_if(should_visualize_grid),),
            );
    }
}

/// Returns true if the grid should be visualized.
fn should_visualize_grid(spec: Res<GridSpec>) -> bool {
    spec.visualize
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, Clone, AsBindGroup)]
pub struct GridVisualizerShaderMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(1)]
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<u32>,
    #[uniform(3)]
    wave_color: LinearRgba,
    #[texture(4)]
    #[sampler(5)]
    sand_texture: Handle<Image>,
}
impl FromWorld for GridVisualizerShaderMaterial {
    fn from_world(world: &mut World) -> Self {
        Self {
            color: MIDNIGHT_BLUE.into(),
            size: GridSize::default(),
            grid: Vec::default(),
            wave_color: ALICE_BLUE.into(),
            sand_texture: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("textures/background/sand.png"),
        }
    }
}
impl ShaderPlaneMaterial for GridVisualizerShaderMaterial {
    fn translation(_spec: &GridSpec) -> Vec3 {
        Vec2::ZERO.extend(zindex::SHADER_BACKGROUND)
    }

    fn raycast_target() -> RaycastTarget {
        RaycastTarget::WorldGrid
    }

    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid.resize(spec.rows as usize * spec.cols as usize, 0);
    }
}
impl GridVisualizerShaderMaterial {
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut grid_events: EventReader<EntityGridEvent>,
    ) {
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        for event in grid_events.read() {
            if let Some(prev_cell) = event.prev_rowcol {
                if event.prev_empty {
                    material.grid[grid_spec.flat_index(prev_cell)] = 0;
                }
            }
            if let Some(rowcol) = event.rowcol {
                material.grid[grid_spec.flat_index(rowcol)] = 1;
            }
        }
    }
}
impl Material for GridVisualizerShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid_background.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
