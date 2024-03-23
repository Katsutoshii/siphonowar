use crate::prelude::*;

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, PauseMenu::create)
            .add_systems(OnEnter(PausedState::Paused), PauseMenu::show)
            .add_systems(OnExit(PausedState::Paused), PauseMenu::hide);
    }
}

#[derive(Component)]
pub struct PauseMenu;
impl PauseMenu {
    fn create(mut commands: Commands) {
        commands.spawn((
            PauseMenu,
            TextBundle {
                text: Text::from_section(
                    ["Paused"].join("\n"),
                    TextStyle {
                        font_size: 36.0,
                        ..default()
                    },
                ),
                visibility: Visibility::Hidden,
                ..default()
            }
            .with_style(Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            }),
        ));
    }

    fn show(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
    fn hide(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
