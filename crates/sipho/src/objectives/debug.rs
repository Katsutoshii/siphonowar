use bevy::text::Text2dBounds;

use crate::prelude::*;

#[derive(Component)]
pub struct ObjectiveDebugger;
impl ObjectiveDebugger {
    #[allow(dead_code)]
    pub fn bundle(self) -> impl Bundle {
        (
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "Objective",
                        TextStyle {
                            font_size: 18.0,
                            ..default()
                        },
                    )],
                    justify: JustifyText::Center,
                    ..default()
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: Vec2::new(1., 1.),
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_translation(Vec3::Z).with_scale(Vec3::new(0.1, 0.1, 1.)),
                ..default()
            },
            self,
        )
    }

    #[allow(dead_code)]
    pub fn update(
        mut query: Query<(&mut Text, &Parent), With<Self>>,
        objectives: Query<&Objectives, Without<Self>>,
    ) {
        for (mut text, parent) in query.iter_mut() {
            let objective = objectives.get(parent.get()).unwrap();
            *text = Text::from_sections(vec![TextSection::new(
                format!("{:?}", objective),
                TextStyle {
                    font_size: 18.0,
                    ..default()
                },
            )]);
        }
    }
}
