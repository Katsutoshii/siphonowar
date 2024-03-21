use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use crate::prelude::*;

use self::window::ScalableWindow;

use super::{
    fog::{VisibilityUpdate, VisibilityUpdateEvent},
    shader_plane::{ShaderPlaneAssets, ShaderPlanePlugin},
    ShaderPlaneMaterial,
};

/// Plugin for visualizing the grid.
/// This plugin reads events from the entity grid and updates the shader's input buffer
/// to light up the cells that have entities.
pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<MinimapShaderMaterial>::default())
            .add_systems(
                FixedUpdate,
                MinimapShaderMaterial::update
                    .before(CameraController::update)
                    .after(GridEntity::update),
            );
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MinimapShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    size: GridSize,
    #[uniform(2)]
    camera_position: Vec2,
    #[uniform(3)]
    viewport_size: Vec2,
    #[storage(4, read_only)]
    grid: Vec<u32>,
    #[storage(5, read_only)]
    visibility_grid: Vec<f32>,
}
impl Default for MinimapShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::TEAL,
            size: GridSize::default(),
            camera_position: Vec2::ZERO,
            viewport_size: Vec2 { x: 16., y: 9. },
            grid: Vec::default(),
            visibility_grid: Vec::default(),
        }
    }
}
impl ShaderPlaneMaterial for MinimapShaderMaterial {
    fn scale(window: &Window, _spec: &GridSpec) -> Vec3 {
        let quad_size = Self::quad_size(window);
        (quad_size * Vec2 { x: 1., y: -1. }).extend(1.)
    }
    fn translation(window: &Window, _spec: &GridSpec) -> Vec3 {
        let viewport_size = window.scaled_size();
        let quad_size = Self::quad_size(window);

        let mut translation = Vec2::ZERO;
        translation += Vec2 {
            x: viewport_size.x,
            y: -viewport_size.y,
        } / 2.;
        translation -= Vec2 {
            x: quad_size.x,
            y: -quad_size.y,
        } / 2.;
        translation.extend(zindex::MINIMAP)
    }

    fn resize(&mut self, spec: &GridSpec) {
        self.size.rows = spec.rows as u32;
        self.size.cols = spec.cols as u32;
        self.size.width = spec.width;
        self.grid
            .resize(self.size.rows as usize * self.size.cols as usize, 0);

        self.visibility_grid
            .resize(self.size.rows as usize * self.size.cols as usize, 1.);
    }
    fn raycast_target() -> RaycastTarget {
        RaycastTarget::Minimap
    }
    fn parent_camera() -> bool {
        true
    }
}
impl MinimapShaderMaterial {
    const SCREEN_RATIO: f32 = 1. / 8.;

    fn quad_size(window: &Window) -> Vec2 {
        window.scaled_size().xx() * Self::SCREEN_RATIO
    }

    /// Update the grid shader material.
    pub fn update(
        spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut grid_events: EventReader<EntityGridEvent>,
        mut visibility_updates: EventReader<VisibilityUpdateEvent>,
        mut camera_moves: EventReader<CameraMoveEvent>,
    ) {
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();

        for &EntityGridEvent {
            entity: _,
            prev_cell,
            prev_cell_empty,
            cell: rowcol,
        } in grid_events.read()
        {
            if let Some(rowcol) = prev_cell {
                if prev_cell_empty && spec.in_bounds(rowcol) {
                    material.grid[spec.flat_index(rowcol)] = 0;
                }
            }
            if let Some(rowcol) = rowcol {
                if spec.in_bounds(rowcol) {
                    material.grid[spec.flat_index(rowcol)] = 1;
                }
            }
        }

        for event in visibility_updates.read() {
            for &VisibilityUpdate { team: _, rowcol } in &event.removals {
                if spec.in_bounds(rowcol) {
                    material.visibility_grid[spec.flat_index(rowcol)] = 0.5;
                }
            }
            for &VisibilityUpdate { team: _, rowcol } in &event.additions {
                if spec.in_bounds(rowcol) {
                    material.visibility_grid[spec.flat_index(rowcol)] = 0.;
                }
            }
        }

        for event in camera_moves.read() {
            let rowcol = spec.to_rowcol(event.position);
            material.camera_position = Vec2 {
                x: rowcol.1 as f32,
                y: rowcol.0 as f32,
            };
        }
    }
}
impl Material2d for MinimapShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minimap.wgsl".into()
    }
}
