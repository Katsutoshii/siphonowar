use std::time::Duration;

use bevy::{input::ButtonState, prelude::*};

use crate::prelude::*;

#[derive(Component, Default, PartialEq, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub enum Selected {
    #[default]
    Unselected,
    Selected,
}
impl Selected {
    pub fn is_selected(&self) -> bool {
        self == &Self::Selected
    }
}

#[derive(Bundle)]
pub struct HighlightBundle {
    pub name: Name,
    pub highlight: Highlight,
    pub pbr: PbrBundle,
}
impl HighlightBundle {
    pub fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
        Self {
            name: Name::new("Highlight"),
            highlight: Highlight,
            pbr: PbrBundle {
                mesh,
                transform: Transform::default().with_scale(Vec2::splat(1.2).extend(1.2)),
                material,
                visibility: Visibility::Visible,
                ..default()
            },
        }
    }
}

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct SelectorPlugin;
impl Plugin for SelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectorAssets>()
            .register_type::<Selected>()
            .add_systems(Startup, (Selector::setup,))
            .add_systems(
                Update,
                Selector::update
                    .in_set(GameStateSet::Running)
                    .in_set(FixedUpdateStage::Spawn),
            );
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
        highlights: Query<Entity, With<Highlight>>,
        hover_highlights: Query<Entity, With<HoverHighlight>>,
        mut objects: Query<
            (&Object, &Position, &Team, &mut Selected, &Handle<Mesh>),
            Without<Self>,
        >,
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
                                // Reset other selections.
                                for (_object, _transform, _team, mut selected, _mesh) in
                                    &mut objects
                                {
                                    if let Selected::Selected = selected.as_ref() {}
                                    *selected = Selected::Unselected;
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
                                if let Ok((_object, position, team, mut selected, mesh)) =
                                    objects.get_mut(entity)
                                {
                                    if aabb.contains(position.0) {
                                        if selected.is_selected() || *team != config.player_team {
                                            continue;
                                        }
                                        let child_entity = commands
                                            .spawn(HighlightBundle::new(
                                                mesh.clone(),
                                                assets.white_material.clone(),
                                            ))
                                            .id();
                                        commands.entity(entity).add_child(child_entity);
                                        *selected = Selected::Selected;
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
                                if let Ok((_object, _, _team, mut selected, mesh)) =
                                    objects.get_mut(control.entity)
                                {
                                    if selected.is_selected() {
                                        continue;
                                    }
                                    // This entity reference is from PreUpdate, so it may have been deleted.
                                    if commands.get_entity(control.entity).is_none() {
                                        continue;
                                    }
                                    let child_entity = commands
                                        .spawn(HighlightBundle::new(
                                            mesh.clone(),
                                            assets.white_material.clone(),
                                        ))
                                        .id();
                                    commands.entity(control.entity).add_child(child_entity);

                                    *selected = Selected::Selected;
                                }
                            }
                        }
                    }
                }
                ControlAction::SelectHover => {
                    if let Ok((_object, _, _team, _, mesh)) = objects.get_mut(control.entity) {
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
            PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default().with_scale(Vec2::splat(1.).extend(1.)),
                material: assets.blue_material.clone(),
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
    pub blue_material: Handle<StandardMaterial>,
    pub white_material: Handle<StandardMaterial>,
    pub hover_material: Handle<StandardMaterial>,
}

impl FromWorld for SelectorAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(Cuboid::from_size(Vec2::splat(1.).extend(0.))))
            },
            blue_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial {
                    base_color: Color::rgba(0.3, 0.3, 1.0, 0.04),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })
            },
            white_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::WHITE.with_a(0.25)))
            },
            hover_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::WHITE.with_a(0.1)))
            },
        }
    }
}
