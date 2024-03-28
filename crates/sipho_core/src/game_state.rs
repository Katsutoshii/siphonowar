use bevy::{
    asset::{LoadState, UntypedAssetId},
    core::FrameCount,
};

use crate::prelude::*;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
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
        for &asset_id in self.assets.iter() {
            let load_state = server.get_load_state(asset_id);
            dbg!(load_state);
            if let Some(LoadState::Loaded) = load_state {
            } else {
                return false;
            }
        }
        true
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
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
        next_state.set(GameState::Loading)
    }
}

fn loading_state(
    mut next_state: ResMut<NextState<GameState>>,
    mut load_state: ResMut<AssetLoadState>,
    server: Res<AssetServer>,
) {
    // if server.is_changed() {
    //     info!("State changed!");
    if load_state.all_loaded(&server) {
        info!("Loaded!");
        next_state.set(GameState::Running);
        load_state.assets.clear();
    }
    // }
}
