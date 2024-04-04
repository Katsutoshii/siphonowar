use crate::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

pub struct NavigationVisualizerPlugin;
impl Plugin for NavigationVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<NavigationShaderMaterial>::default())
            .add_systems(
                FixedUpdate,
                (NavigationShaderMaterial::update,).run_if(should_visualize_grid),
            );
    }
}

/// Returns true if the grid should be visualized.
fn should_visualize_grid(spec: Res<GridSpec>) -> bool {
    spec.visualize_navigation
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct NavigationShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<f32>,
}
impl Default for NavigationShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::ORANGE_RED,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl ShaderPlaneMaterial for NavigationShaderMaterial {
    fn translation(_spec: &GridSpec) -> Vec3 {
        Vec2::ZERO.extend(zindex::NAVIGATION_LAYER)
    }
    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 0.);
    }
}
impl NavigationShaderMaterial {
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        mut events: EventReader<NavigationCostEvent>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut input_actions: EventReader<ControlEvent>,
    ) {
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        for control in input_actions.read() {
            if control.is_released(ControlAction::Move) {
                material.grid = vec![0.; material.grid.len()];
            }
        }
        for &NavigationCostEvent { rowcol, cost } in events.read() {
            material.grid[grid_spec.flat_index(rowcol)] = cost * 0.002;
        }
    }
}
impl Material for NavigationShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/navigation_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
