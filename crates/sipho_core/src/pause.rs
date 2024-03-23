use crate::prelude::*;

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PausedState>()
            .add_systems(Update, toggle_pause_game);
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PausedState {
    #[default]
    Running,
    Paused,
}

fn toggle_pause_game(
    state: Res<State<PausedState>>,
    mut next_state: ResMut<NextState<PausedState>>,
    mut controls: EventReader<ControlEvent>,
) {
    for control in controls.read() {
        if !control.is_pressed(ControlAction::PauseMenu) {
            continue;
        }
        match state.get() {
            PausedState::Paused => {
                info!("Unpause");
                next_state.set(PausedState::Running)
            }
            PausedState::Running => {
                info!("Pause");
                next_state.set(PausedState::Paused)
            }
        }
    }
}
