use crate::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// Plugin for obstacles.
/// Obstacles are implemented as a hacky force field in the center of each cell they are present in.
pub struct ObstaclesPlugin;
impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        let debug_obstacles = false;
        if debug_obstacles {
            app.add_plugins(ShaderPlanePlugin::<ObstaclesShaderMaterial>::default())
                .add_systems(
                    Update,
                    (ObstaclesShaderMaterial::update,).in_set(GameStateSet::Running),
                );
        }
        app.add_plugins(Grid2Plugin::<Obstacle>::default())
            .register_type::<ObstaclesSpec>()
            .register_type::<Obstacle>()
            .register_type::<Vec<(RowCol, Obstacle)>>()
            .register_type::<(RowCol, Obstacle)>()
            .register_type::<RowCol>()
            .add_systems(
                FixedUpdate,
                (Grid2::<Obstacle>::bounce_off_obstacles.in_set(FixedUpdateStage::PostPhysics),)
                    .in_set(GameStateSet::Running),
            );
    }
}

// Represents obstacle presence and orientation
#[derive(Default, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Obstacle {
    #[default]
    Empty = 0,
    UpRight = 1,
    UpLeft = 2,
    DownRight = 3,
    DownLeft = 4,
    Full = 5,
}

/// Grid of obstacle data.
#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct ObstaclesSpec(pub Vec<(RowCol, Obstacle)>);

impl Grid2<Obstacle> {
    pub fn is_clear(&self, rowcol: RowCol) -> bool {
        self[rowcol] == Obstacle::Empty && !self.is_boundary(rowcol)
    }

    fn obstacle_force(
        &self,
        position: Vec2,
        velocity: Velocity,
        rowcol: RowCol,
        direction: (i16, i16),
    ) -> Force {
        let mut force = Force::ZERO;
        let obstacle_position = self.to_world_position(rowcol);

        if !self.is_clear(rowcol) {
            //   W
            // ┏---┓
            // ┏━━━┳━━━┓
            // ┃ X ┃   Y
            // ┗━━━┻━━━┛
            //   ┗-----┛
            //    1.5 W
            let max_d = 1.5 * self.spec.width;
            let d = obstacle_position - position; // Distance to the obstacle per each axis.
            let magnitude = ((max_d - d.abs()) / max_d).clamp(Vec2::ZERO, Vec2::ONE); // [0, 1], [far from obstacle, near to obstacle]

            // d points towards the obstacle.
            // when d and v point in the same direction
            // Apply greater force when moving towards the obstacle.
            let directional_adjustment = 1.0 + d.dot(velocity.0).clamp(0., 0.5);
            // let directional_adjustment = 1.;
            let direction = Vec2::new(direction.1 as f32, direction.0 as f32);
            force += Force(-magnitude * directional_adjustment * direction);
        }
        force
    }

    /// Compute force due to neighboring obstacles.
    /// For each neighboring obstacle, if the object is moving towards the obstacle
    /// we apply a force away from the obstacle.
    pub fn force(&self, position: Vec2, velocity: Velocity) -> Force {
        let mut force = Force::ZERO;
        if let Some((row, col)) = self.to_rowcol(position) {
            if self.is_boundary((row, col)) {
                return Force::ZERO;
            }
            for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let obstacle_rowcol = ((row as i16 + dr) as u16, (col as i16 + dc) as u16);
                force += self.obstacle_force(position, velocity, obstacle_rowcol, (dr, dc));
            }
        }
        force
    }

    /// Bounce simulated objects off obstacles.
    pub fn bounce_off_obstacles(
        obstacles: Res<Self>,
        mut query: Query<(&mut Position, &mut Velocity, &mut Force)>,
    ) {
        for (mut position, mut velocity, mut force) in query.iter_mut() {
            let obstacle_force = obstacles.force(position.0, *velocity) * 10.;
            *force += obstacle_force;
            if let Some(rowcol) = obstacles.to_rowcol(position.0) {
                if !obstacles.is_clear(rowcol) {
                    velocity.0 *= -1.0;
                    position.0 += velocity.0;
                }
            }
        }
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ObstaclesShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    size: GridSize,
    #[storage(2, read_only)]
    pub grid: Vec<u32>,
}
impl Default for ObstaclesShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::MIDNIGHT_BLUE,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl ShaderPlaneMaterial for ObstaclesShaderMaterial {
    fn translation(_spec: &GridSpec) -> Vec3 {
        Vec2::ZERO.extend(zindex::OBSTACLES)
    }

    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid.resize(
            spec.rows as usize * spec.cols as usize,
            Obstacle::Empty as u32,
        );
    }
}
impl ObstaclesShaderMaterial {
    /// Update the grid shader material.
    pub fn update(
        grid: Res<Grid2<Obstacle>>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
    ) {
        if !grid.is_changed() {
            return;
        }
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();

        for row in 0..grid.rows {
            for col in 0..grid.cols {
                material.grid[grid.flat_index((row, col))] = grid[(row, col)] as u32;
            }
        }
    }
}
impl Material for ObstaclesShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/obstacles.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
