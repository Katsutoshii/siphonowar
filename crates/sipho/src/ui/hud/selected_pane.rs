use crate::prelude::*;

use super::{assets::HudAssets, SpawnHudNode};

pub struct HudSelectedPane;
impl SpawnHudNode for HudSelectedPane {
    fn spawn(parent: &mut ChildBuilder, _assets: &HudAssets) {
        // Column 2: Selected list
        parent.spawn((ButtonBundle {
            style: Style {
                // top: Val::Percent(50.),
                width: Val::Px(600.0),
                height: Val::Px(150.0),
                ..default()
            },
            background_color: Color::GRAY.with_a(0.02).into(),
            ..default()
        },));
    }
}
