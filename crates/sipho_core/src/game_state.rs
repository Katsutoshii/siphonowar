use bevy::{
    asset::{LoadState, UntypedAssetId},
    core::FrameCount,
};

use crate::prelude::*;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<DebugState>()
            .init_resource::<AssetLoadState>()
            .add_systems(
                Update,
                (
                    prepare_window.run_if(in_state(GameState::PrepareWindow)),
                    loading_state.run_if(in_state(GameState::Loading)),
                    toggle_pause_game,
                ),
            );
    }
}

#[derive(Resource, Default, Debug)]
pub struct AssetLoadState {
    pub assets: Vec<UntypedAssetId>,
}
impl AssetLoadState {
    pub fn track<A: Asset>(&mut self, handle: &Handle<A>) {
        self.assets.push(handle.into());
    }
    pub fn all_loaded(&self, server: &AssetServer) -> bool {
        let mut num_loaded = 0;
        let total_assets = self.assets.len();
        for &asset_id in self.assets.iter() {
            let load_state = server.get_load_state(asset_id);
            match load_state {
                Some(LoadState::NotLoaded | LoadState::Loading) => {}
                Some(LoadState::Failed | LoadState::Loaded) | None => {
                    num_loaded += 1;
                }
            }
        }
        info!("Loaded {} / {}", num_loaded, total_assets);
        num_loaded == total_assets
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    PrepareWindow,
    Loading,
    Running,
    Paused,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DebugState {
    #[default]
    NoDebug,
    DebugConsole,
}

fn toggle_pause_game(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut controls: EventReader<ControlEvent>,
) {
    for control in controls.read() {
        if !control.is_pressed(ControlAction::PauseMenu) {
            continue;
        }
        match state.get() {
            GameState::Paused => next_state.set(GameState::Running),
            GameState::Running => next_state.set(GameState::Paused),
            _ => {}
        }
    }
}

fn prepare_window(
    mut next_state: ResMut<NextState<GameState>>,
    mut window: Query<&mut Window>,
    frames: Res<FrameCount>,
) {
    if frames.0 == 3 {
        window.single_mut().visible = true;
        next_state.set(GameState::Loading)
    }
}

fn loading_state(
    mut next_state: ResMut<NextState<GameState>>,
    mut load_state: ResMut<AssetLoadState>,
    server: Res<AssetServer>,
) {
    if load_state.all_loaded(&server) {
        info!("Loaded!");
        next_state.set(GameState::Running);
        load_state.assets.clear();
    }
}
