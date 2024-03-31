use crate::prelude::*;
use std::ops::Index;

pub struct TeamPlugin;
impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Team>()
            .register_type::<TeamConfig>()
            .insert_resource(TeamConfig::default());
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TeamConfig {
    pub player_team: Team,
}
impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            player_team: Team::Blue,
        }
    }
}

/// Enum to specify the team of the given object.
#[derive(Component, Default, Debug, PartialEq, Eq, Reflect, Clone, Copy, Hash, clap::ValueEnum)]
#[reflect(Component)]
#[repr(u8)]
pub enum Team {
    #[default]
    None = 0,
    Blue = 1,
    Red = 2,
}
impl Team {
    /// Number of teams.
    pub const COUNT: usize = 3;

    pub const BRIGHT_SEA_GREEN: Color = Color::rgb(0.18 + 0.2, 0.55 + 0.2, 0.34 + 0.2);
    pub const BRIGHT_TEAL: Color = Color::rgb(0.1, 0.5 + 0.2, 0.5 + 0.2);

    pub const ALL: [Self; Self::COUNT] = [Self::None, Self::Blue, Self::Red];
    pub const COLORS: [Color; Self::COUNT] =
        [Self::BRIGHT_SEA_GREEN, Self::BRIGHT_TEAL, Color::TOMATO];
}

#[derive(Default, Clone)]
pub struct TeamMaterials {
    pub primary: Handle<StandardMaterial>,
    pub secondary: Handle<StandardMaterial>,
    pub background: Handle<StandardMaterial>,
}
impl TeamMaterials {
    pub fn new(color: Color, assets: &mut Assets<StandardMaterial>) -> Self {
        Self {
            primary: assets.add(StandardMaterial::from(color)),
            secondary: assets.add(StandardMaterial::from(color)),
            background: assets.add(StandardMaterial::from(color.with_a(0.3))),
        }
    }
}

impl<T: Sized> Index<Team> for [T] {
    type Output = T;
    fn index(&self, index: Team) -> &Self::Output {
        &self[index as usize]
    }
}
