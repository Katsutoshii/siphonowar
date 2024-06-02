use std::time::Duration;

use bevy::input::ButtonState;

use crate::prelude::*;

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct SelectorPlugin;
impl Plugin for SelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectorAssets>()
            .register_type::<Selected>()
            .add_systems(Startup, (Selector::setup,))
            .add_systems(
                FixedUpdate,
                Selector::update
                    .in_set(GameStateSet::Running)
                    .in_set(FixedUpdateStage::Spawn),
            );
    }
}

#[derive(Component, Default, PartialEq, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct Selected;

#[derive(Component, Default, PartialEq, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Selectable;

#[derive(Bundle)]
pub struct HighlightBundle {
    pub name: Name,
    pub highlight: Highlight,
    pub pbr: PbrBundle,
}
impl HighlightBundle {
    pub fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>, scale: f32) -> Self {
        Self {
            name: Name::new("Highlight"),
            highlight: Highlight,
            pbr: PbrBundle {
                mesh,
                transform: Transform::default().with_scale(Vec3::splat(scale)),
                material,
                visibility: Visibility::Visible,
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct Highlight;
#[derive(Component)]
pub struct HoverHighlight;

#[derive(Component, Default)]
pub struct Selector {
    pub aabb: Aabb2,
}
impl Selector {
    pub fn setup(mut commands: Commands, assets: Res<SelectorAssets>) {
        commands.spawn(Self::default().bundle(&assets));
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn update(
        mut commands: Commands,
        mut query: Query<(&mut Self, &mut Transform, &mut Visibility)>,
        highlights: Query<Entity, (With<Highlight>, Without<HoverHighlight>)>,
        hover_highlights: Query<Entity, With<HoverHighlight>>,
        mut objects: Query<
            (&Object, &Position, &Team, &Handle<Mesh>),
            (Without<Selected>, With<Selectable>, Without<Self>),
        >,
        selected: Query<Entity, With<Selected>>,
        grid: Res<Grid2<TeamEntitySets>>,
        assets: Res<SelectorAssets>,
        config: Res<TeamConfig>,
        mut events: EventReader<ControlEvent>,
    ) {
        for control in events.read() {
            match control.action {
                ControlAction::Select => {
                    let (mut selector, mut transform, mut visibility) = query.single_mut();
                    match control.state {
                        ButtonState::Pressed => {
                            if *visibility == Visibility::Hidden {
                                for entity in selected.iter() {
                                    commands.entity(entity).remove::<Selected>();
                                }
                                for entity in highlights.iter() {
                                    commands.entity(entity).remove_parent().despawn();
                                }
                                selector.aabb.min = control.position;
                                *visibility = Visibility::Visible;
                                transform.scale = Vec3::ZERO;
                                transform.translation = control.position.extend(zindex::SELECTOR);
                            }

                            selector.aabb.max = control.position;
                            transform.translation = selector.aabb.center().extend(zindex::SELECTOR);
                            transform.scale = selector.aabb.size().extend(0.0);

                            // Correct the bounding box before we check entity collision, since it might be backwards.
                            let mut aabb = selector.aabb.clone();
                            aabb.enforce_minmax();
                            // Check the grid for entities in this bounding box.
                            for entity in grid.get_entities_in_aabb(&aabb) {
                                if let Ok((_object, position, team, mesh)) = objects.get_mut(entity)
                                {
                                    if aabb.contains(position.0) {
                                        if *team != config.player_team {
                                            continue;
                                        }
                                        let child_entity = commands
                                            .spawn(HighlightBundle::new(
                                                mesh.clone(),
                                                assets.white_material.clone(),
                                                1.2,
                                            ))
                                            .id();
                                        commands
                                            .entity(entity)
                                            .insert(Selected)
                                            .add_child(child_entity);
                                    }
                                }
                            }
                        }
                        ButtonState::Released => {
                            *visibility = Visibility::Hidden;
                            // On release, select the hovered entity.
                            if control.duration < Duration::from_millis(100) {
                                for entity in highlights.iter() {
                                    commands.entity(entity).remove_parent().despawn();
                                }
                                if let Ok((_object, _, _team, mesh)) =
                                    objects.get_mut(control.entity)
                                {
                                    // This entity reference is from PreUpdate, so it may have been deleted.
                                    if commands.get_entity(control.entity).is_none() {
                                        continue;
                                    }
                                    let child_entity = commands
                                        .spawn(HighlightBundle::new(
                                            mesh.clone(),
                                            assets.white_material.clone(),
                                            1.5,
                                        ))
                                        .id();
                                    commands
                                        .entity(control.entity)
                                        .insert(Selected)
                                        .add_child(child_entity);
                                }
                            }
                        }
                    }
                }
                ControlAction::SelectHover => {
                    if let Ok((_object, _, _team, mesh)) = objects.get_mut(control.entity) {
                        for entity in hover_highlights.iter() {
                            commands.entity(entity).remove_parent().despawn();
                        }

                        if control.state == ButtonState::Pressed {
                            // Spawn a lighter highlight on the hovered entity.
                            let child_entity = commands
                                .spawn((
                                    HoverHighlight,
                                    HighlightBundle::new(
                                        mesh.clone(),
                                        assets.hover_material.clone(),
                                        1.6,
                                    ),
                                ))
                                .id();
                            commands.entity(control.entity).add_child(child_entity);
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    fn bundle(self, assets: &SelectorAssets) -> impl Bundle {
        (
            self,
            Name::new("Selector"),
            // OutlineBundle {
            //     outline: OutlineVolume {
            //         visible: true,
            //         colour: Color::ANTIQUE_WHITE,
            //         width: 10.0,
            //     },
            //     ..default()
            // },
            PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default().with_scale(Vec2::splat(1.).extend(1.)),
                material: assets.selector_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
        )
    }
}

/// Handles to common selector assets.
#[derive(Resource)]
pub struct SelectorAssets {
    pub mesh: Handle<Mesh>,
    pub selector_material: Handle<StandardMaterial>,
    pub white_material: Handle<StandardMaterial>,
    pub hover_material: Handle<StandardMaterial>,
}

impl FromWorld for SelectorAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world.add_asset(Mesh::from(Cuboid::from_size(Vec2::splat(1.).extend(0.)))),
            selector_material: world.add_asset(StandardMaterial {
                base_color: Color::YELLOW.with_a(0.05),
                alpha_mode: AlphaMode::Blend,
                emissive: Color::YELLOW,
                unlit: true,
                ..default()
            }),
            white_material: world.add_asset(StandardMaterial {
                base_color: Color::ANTIQUE_WHITE.with_a(0.3),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            hover_material: world.add_asset(StandardMaterial {
                base_color: Color::WHITE.with_a(0.15),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}
