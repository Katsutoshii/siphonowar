use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{
    camera::{self, CameraAspectRatio},
    prelude::*,
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
                    .before(CameraController::update_screen_control)
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
    fn scale(_camera: &Camera, _spec: &GridSpec) -> Vec3 {
        (Vec2::ONE / 4.).extend(1.)
    }
    fn translation(cam: &Camera, _spec: &GridSpec) -> Vec3 {
        let aspect_ratio = cam.aspect_ratio();
        let bottom_left = aspect_ratio / 2.;
        let quad = Self::scale(cam, _spec).xy();
        let quad_offset = quad / 2.;
        let flip_y = Vec2 { x: 1., y: -1. };

        // Position the object in the bottom left.
        ((bottom_left - quad_offset) * flip_y).extend(-camera::get_depth())
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
impl Material for MinimapShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minimap.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
