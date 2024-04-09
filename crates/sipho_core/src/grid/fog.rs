use crate::prelude::*;

use bevy::render::render_resource::{AsBindGroup, ShaderRef};

/// Plugin for fog of war.
pub struct FogPlugin;
impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FogConfig>()
            .insert_resource(FogConfig::default())
            .add_plugins(ShaderPlanePlugin::<FogShaderMaterial>::default())
            .add_plugins(Grid2Plugin::<TeamVisibility>::default())
            .add_event::<VisibilityUpdateEvent>()
            .add_systems(
                FixedUpdate,
                (
                    Grid2::<TeamVisibility>::update,
                    Grid2::<TeamVisibility>::update_visibility,
                    FogShaderMaterial::update,
                )
                    .chain()
                    .in_set(SystemStage::Cleanup),
            );
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct FogConfig {
    pub player_team: Team,
    pub visibility_radius: u16,
    pub fog_radius: u16,
}
impl Default for FogConfig {
    fn default() -> Self {
        Self {
            player_team: Team::Blue,
            visibility_radius: 5,
            fog_radius: 6,
        }
    }
}

/// Represents an update to visibility.
#[derive(Default)]
pub struct VisibilityUpdate {
    pub team: Team,
    pub rowcol: RowCol,
}

/// Communicates to other systems that visibility has been updated.
#[derive(Event, Default)]
pub struct VisibilityUpdateEvent {
    pub additions: Vec<VisibilityUpdate>,
    pub removals: Vec<VisibilityUpdate>,
}

/// Stores visibility per team.
#[derive(Clone, Default)]
pub struct TeamVisibility {
    teams: [u32; Team::COUNT],
}
impl TeamVisibility {
    pub fn get(&self, team: Team) -> u32 {
        self.teams[team as usize]
    }

    pub fn get_mut(&mut self, team: Team) -> &mut u32 {
        &mut self.teams[team as usize]
    }
}

impl Grid2<TeamVisibility> {
    pub fn update_visibility(
        mut query: Query<(&GridEntity, &mut Visibility)>,
        grid: ResMut<Self>,
        configs: Res<FogConfig>,
    ) {
        for (grid_entity, mut visibility) in &mut query {
            if let Some(cell) = grid_entity.cell {
                *visibility = grid.get_visibility(cell, configs.player_team)
            }
        }
    }

    pub fn update(
        mut grid: ResMut<Self>,
        config: Res<FogConfig>,
        teams: Query<&Team>,
        mut grid_events: EventReader<EntityGridEvent>,
        mut visibility_events: EventWriter<VisibilityUpdateEvent>,
    ) {
        let mut updates = VisibilityUpdateEvent::default();

        for event in grid_events.read() {
            let team = *teams.get(event.entity).unwrap();
            if let Some(prev_cell) = event.prev_cell {
                updates
                    .removals
                    .extend(grid.remove_visibility(prev_cell, team, &config))
            }
            if let Some(cell) = event.cell {
                updates
                    .additions
                    .extend(grid.add_visibility(cell, team, &config));
            }
        }

        visibility_events.send(updates);
    }

    fn remove_visibility(
        &mut self,
        rowcol: RowCol,
        team: Team,
        config: &FogConfig,
    ) -> Vec<VisibilityUpdate> {
        let mut updates = Vec::default();
        let radius = config.visibility_radius;
        for other_rowcol in self.get_in_radius_discrete(rowcol, radius) {
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                if grid_visibility.get(team) > 0 {
                    *grid_visibility.get_mut(team) -= 1;
                    if team == config.player_team && grid_visibility.get(team) == 0 {
                        updates.push(VisibilityUpdate {
                            team,
                            rowcol: other_rowcol,
                        });
                    }
                }
            }
        }
        updates
    }

    /// Return the visibility status at the cell corresponding to position for the given team.
    pub fn get_visibility(&self, rowcol: RowCol, team: Team) -> Visibility {
        if let Some(visibility) = self.get(rowcol) {
            if visibility.get(team) > 0 {
                return Visibility::Visible;
            }
        }
        Visibility::Hidden
    }

    fn add_visibility(
        &mut self,
        cell: RowCol,
        team: Team,
        config: &FogConfig,
    ) -> Vec<VisibilityUpdate> {
        let mut updates = Vec::default();
        let radius = config.visibility_radius;
        for other_rowcol in self.get_in_radius_discrete(cell, radius) {
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                *grid_visibility.get_mut(team) += 1;
                if team == config.player_team
                    && GridSpec::in_radius(cell, other_rowcol, config.fog_radius)
                {
                    updates.push(VisibilityUpdate {
                        team,
                        rowcol: other_rowcol,
                    });
                }
            }
        }
        updates
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FogShaderMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(1)]
    pub size: GridSize,
    #[storage(2, read_only)]
    pub grid: Vec<f32>,
}
impl Default for FogShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl ShaderPlaneMaterial for FogShaderMaterial {
    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 1.);
    }
    fn translation(_spec: &GridSpec) -> Vec3 {
        Vec2::ZERO.extend(zindex::FOG_OF_WAR)
    }
}
impl Material for FogShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fog_of_war.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
impl FogShaderMaterial {
    pub fn update(
        spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut updates: EventReader<VisibilityUpdateEvent>,
    ) {
        let material: &mut FogShaderMaterial =
            shader_assets.get_mut(&assets.shader_material).unwrap();
        for event in updates.read() {
            for &VisibilityUpdate { team: _, rowcol } in &event.removals {
                material.grid[spec.flat_index(rowcol)] = 0.5;
            }
            for &VisibilityUpdate { team: _, rowcol } in &event.additions {
                material.grid[spec.flat_index(rowcol)] = 0.;
            }
        }
    }
}
