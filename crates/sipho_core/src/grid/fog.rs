use crate::prelude::*;

use bevy::render::{
    render_resource::{
        AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
    },
    texture::ImageSampler,
};
use image::DynamicImage;

/// Plugin for fog of war.
pub struct FogPlugin;
impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FogConfig>()
            .insert_resource(FogConfig::default())
            .init_resource::<FogAssets>()
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
                    .in_set(FixedUpdateStage::PreDespawn)
                    .in_set(GameStateSet::Running)
                    .after(GridEntity::cleanup),
            )
            .add_systems(
                FixedUpdate,
                (FogShaderMaterial::init.after(FogShaderMaterial::resize_on_change),)
                    .in_set(GameStateSet::Running),
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
            visibility_radius: 7,
            fog_radius: 6,
        }
    }
}

/// Represents an update to visibility.
#[derive(Default)]
pub struct VisibilityUpdate {
    pub team: Team,
    pub rowcol: RowCol,
    pub amount: f32,
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
            if let Some(rowcol) = grid_entity.rowcol {
                *visibility = grid.get_visibility(rowcol, configs.player_team)
            }
        }
    }

    pub fn update(
        mut grid: ResMut<Self>,
        config: Res<FogConfig>,
        mut grid_events: EventReader<EntityGridEvent>,
        mut visibility_events: EventWriter<VisibilityUpdateEvent>,
    ) {
        let mut updates = VisibilityUpdateEvent::default();

        for event in grid_events.read() {
            if let Some(prev_rowcol) = event.prev_rowcol {
                updates
                    .removals
                    .extend(grid.remove_visibility(prev_rowcol, event.team, &config));
            }
            if let Some(rowcol) = event.rowcol {
                updates
                    .additions
                    .extend(grid.add_visibility(rowcol, event.team, &config));
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
        for other_rowcol in self.get_in_radius_discrete(rowcol, config.visibility_radius) {
            // Don't add visibility on the boundary.
            if self.is_near_boundary(other_rowcol) {
                continue;
            }
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                if grid_visibility.get(team) > 0 {
                    *grid_visibility.get_mut(team) -= 1;
                }
                if team == config.player_team && grid_visibility.get(team) == 0 {
                    updates.push(VisibilityUpdate {
                        team,
                        rowcol: other_rowcol,
                        amount: 0.5,
                    });
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
        for other_rowcol in self.get_in_radius_discrete(cell, config.visibility_radius) {
            // Don't add visibility on the boundary.
            if self.is_boundary(other_rowcol) {
                continue;
            }
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                *grid_visibility.get_mut(team) += 1;
                if team == config.player_team {
                    let amount = if GridSpec::in_radius(cell, other_rowcol, config.fog_radius) {
                        1.0
                    } else {
                        0.5
                    };
                    updates.push(VisibilityUpdate {
                        team,
                        rowcol: other_rowcol,
                        amount,
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
    #[texture(2)]
    #[sampler(3)]
    visibility_texture: Handle<Image>,
}
impl FromWorld for FogShaderMaterial {
    fn from_world(world: &mut World) -> Self {
        Self {
            color: Color::BLACK,
            size: GridSize::default(),
            visibility_texture: world.get_resource::<FogAssets>().unwrap().texture.clone(),
        }
    }
}
impl ShaderPlaneMaterial for FogShaderMaterial {
    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
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
    pub fn init(spec: Res<GridSpec>, assets: ResMut<FogAssets>, mut images: ResMut<Assets<Image>>) {
        if !spec.is_changed() {
            return;
        }
        let size = Extent3d {
            width: spec.cols as u32,
            height: spec.rows as u32,
            ..default()
        };

        let image = images.get_mut(&assets.texture).unwrap();
        image.resize(size);
    }

    pub fn update(
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut updates: EventReader<VisibilityUpdateEvent>,
        fog_assets: Res<FogAssets>,
        mut images: ResMut<Assets<Image>>,
    ) {
        // Mark shader assets as changed.
        shader_assets.get_mut(&assets.shader_material);

        let image = images.get_mut(&fog_assets.texture).unwrap();
        if let Ok(DynamicImage::ImageRgba8(mut rgba)) = image.clone().try_into_dynamic() {
            for event in updates.read() {
                for &VisibilityUpdate { rowcol, amount, .. } in &event.removals {
                    let amount_u8 = ((amount * 0.99) * (u8::MAX as f32)) as u8;
                    let (y, x) = rowcol;
                    let pixel = rgba.get_pixel_mut(x as u32, y as u32);
                    pixel.0[3] = amount_u8;
                }
                for &VisibilityUpdate { rowcol, amount, .. } in &event.additions {
                    let amount_u8 = ((amount * 0.99) * (u8::MAX as f32)) as u8;
                    let (y, x) = rowcol;
                    let pixel = rgba.get_pixel_mut(x as u32, y as u32);
                    pixel.0[3] = amount_u8.max(pixel.0[3]);
                }
            }

            *image = Image::from_dynamic(DynamicImage::ImageRgba8(rgba), true, image.asset_usage);
        }
    }
}

/// Handles to shader plane assets.
#[derive(Resource)]
pub struct FogAssets {
    pub texture: Handle<Image>,
}
impl FromWorld for FogAssets {
    fn from_world(world: &mut World) -> Self {
        info!("FogAssets::from_world");
        let assets = Self {
            texture: {
                let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
                let size = Extent3d {
                    width: 256,
                    height: 256,
                    ..default()
                };
                let mut image = Image {
                    texture_descriptor: TextureDescriptor {
                        label: None,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Rgba8UnormSrgb,
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        usage: TextureUsages::TEXTURE_BINDING
                            | TextureUsages::COPY_DST
                            | TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    },
                    sampler: ImageSampler::linear(),
                    ..default()
                };
                image.resize(size);
                images.add(image)
            },
        };
        assets
    }
}
