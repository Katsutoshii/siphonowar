use crate::prelude::*;

use bevy::reflect::TypePath;
use bevy::render::render_resource::*;
use bevy::ui::RelativeCursorPosition;

use super::assets::HudAssets;
use super::SpawnHudNode;

pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MinimapUiMaterial>::default())
            .add_systems(
                Update,
                MinimapUiMaterial::update
                    .before(CameraController::update_screen_control)
                    .after(GridEntity::update),
            );
    }
}

#[derive(ShaderType, TypePath, Debug, Clone, Copy)]
pub struct MinimapGridEntry {
    visibility: f32,
    team_presence: [f32; 3],
}
impl Default for MinimapGridEntry {
    fn default() -> Self {
        Self {
            visibility: 0.,
            team_presence: [0.; 3],
        }
    }
}

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 256;
pub const SIZE: usize = WIDTH * HEIGHT;
pub const DEFAULT_VIEWPORT_SIZE: Vec2 = Vec2 {
    x: 16. * 1.5,
    y: 9. * 1.5,
};

#[derive(ShaderType, AsBindGroup, TypePath, Debug, Clone)]
struct MinimapUiMaterialInput {
    #[uniform(0)]
    colors: [Color; Team::COUNT],
    #[uniform(1)]
    size: GridSize,
    #[uniform(2)]
    camera_position: Vec2,
    #[uniform(3)]
    viewport_size: Vec2,
}
impl Default for MinimapUiMaterialInput {
    fn default() -> Self {
        Self {
            colors: Team::COLORS,
            size: GridSize::default(),
            camera_position: Vec2::ZERO,
            viewport_size: DEFAULT_VIEWPORT_SIZE,
        }
    }
}

#[derive(Component)]
pub struct MinimapUi;
impl SpawnHudNode for MinimapUi {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets) {
        parent.spawn((
            MinimapUi,
            RaycastTarget::Minimap,
            RelativeCursorPosition::default(),
            MaterialNodeBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(300.0),
                    ..default()
                },
                material: assets.minimap_material.clone(),
                ..default()
            },
        ));
    }
}

#[derive(AsBindGroup, Asset, TypePath, Default, Debug, Clone)]
pub struct MinimapUiMaterial {
    #[uniform(0)]
    input: MinimapUiMaterialInput,
    #[storage(1, read_only)]
    grid: Vec<MinimapGridEntry>,
}
impl UiMaterial for MinimapUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minimap.wgsl".into()
    }
}
impl MinimapUiMaterial {
    fn resize(&mut self, spec: &GridSpec) {
        self.input.size.rows = spec.rows as u32;
        self.input.size.cols = spec.cols as u32;
        self.input.size.width = spec.width;
        self.grid.resize(
            self.input.size.rows as usize * self.input.size.cols as usize,
            MinimapGridEntry::default(),
        );
    }

    pub fn update(
        spec: Res<GridSpec>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut grid_events: EventReader<EntityGridEvent>,
        mut visibility_updates: EventReader<VisibilityUpdateEvent>,
        mut camera_moves: EventReader<CameraMoveEvent>,
    ) {
        for (_, material) in shader_assets.iter_mut() {
            if spec.is_changed() {
                material.resize(&spec);
            }

            for event in grid_events.read() {
                let team = event.team as usize;
                if let Some(rowcol) = event.prev_rowcol {
                    if event.prev_empty && spec.in_bounds(rowcol) {
                        material.grid[spec.flat_index(rowcol)].team_presence[team] = 0.;
                    }
                }
                if let Some(rowcol) = event.rowcol {
                    if spec.in_bounds(rowcol) {
                        material.grid[spec.flat_index(rowcol)].team_presence[team] = 1.;
                    }
                }
            }

            for event in visibility_updates.read() {
                for &VisibilityUpdate { rowcol, amount, .. } in &event.removals {
                    if spec.in_bounds(rowcol) {
                        material.grid[spec.flat_index(rowcol)].visibility = amount;
                    }
                }
                for &VisibilityUpdate { rowcol, amount, .. } in &event.additions {
                    if spec.in_bounds(rowcol) {
                        material.grid[spec.flat_index(rowcol)].visibility = material.grid
                            [spec.flat_index(rowcol)]
                        .visibility
                        .max(amount);
                    }
                }
            }

            for event in camera_moves.read() {
                let position = event.position.xy() + event.position.z * MainCamera::THETA.tan();
                material.input.camera_position = spec.to_uv(position);
                material.input.viewport_size =
                    DEFAULT_VIEWPORT_SIZE * (event.position.z / zindex::CAMERA);
            }
        }
    }
}
