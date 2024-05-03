use crate::prelude::*;

use super::{assets::HudAssets, SpawnHudNode};

pub struct HudControlsPane;
impl SpawnHudNode for HudControlsPane {
    fn spawn(parent: &mut ChildBuilder, _assets: &HudAssets) {
        parent.spawn((ButtonBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(300.0),
                ..default()
            },
            background_color: Color::GRAY.with_a(0.02).into(),
            ..default()
        },));
    }
}
