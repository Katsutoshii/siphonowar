use bevy::{input::ButtonState, prelude::*};

use crate::prelude::*;

#[derive(Component, Default, PartialEq, Clone)]
pub enum Selected {
    #[default]
    Unselected,
    Selected,
}
impl Selected {
    pub fn is_selected(&self) -> bool {
        self != &Self::Unselected
    }
}

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct SelectorPlugin;
impl Plugin for SelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectorAssets>()
            .add_systems(Startup, Selector::startup)
            .add_systems(FixedUpdate, Selector::update.in_set(GameStateSet::Running));
    }
}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Highlight;

#[derive(Component, Default)]
pub struct Selector {
    pub active: bool,
    pub aabb: Aabb2,
}
impl Selector {
    pub fn startup(mut commands: Commands, assets: Res<SelectorAssets>) {
        commands.spawn(Self::default().bundle(&assets));
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn update(
        mut commands: Commands,
        mut query: Query<(&mut Self, &mut Transform, &mut Visibility)>,
        highlights: Query<Entity, With<Highlight>>,
        mut objects: Query<
            (
                &Object,
                &GlobalTransform,
                &Team,
                &mut Selected,
                &Handle<Mesh>,
            ),
            Without<Self>,
        >,
        grid: Res<Grid2<EntitySet>>,
        assets: Res<SelectorAssets>,
        config: Res<TeamConfig>,
        mut events: EventReader<ControlEvent>,
    ) {
        for control in events.read() {
            if control.action != ControlAction::Select {
                continue;
            }
            let (mut selector, mut transform, mut visibility) = query.single_mut();

            match control.state {
                ButtonState::Pressed => {
                    if *visibility == Visibility::Hidden {
                        // Reset other selections.
                        for (_object, _transform, _team, mut selected, _mesh) in &mut objects {
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

                    // While held
                    selector.aabb.max = control.position;
                    // Resize the square to match the bounding box.
                    transform.translation = selector.aabb.center().extend(zindex::SELECTOR);
                    transform.scale = selector.aabb.size().extend(0.0);

                    // Correct the bounding box before we check entity collision, since it might be backwards.
                    let mut aabb = selector.aabb.clone();
                    aabb.enforce_minmax();
                    // Check the grid for entities in this bounding box.
                    for entity in grid.get_entities_in_aabb(&aabb) {
                        if let Ok(mut_obj) = objects.get_mut(entity) {
                            let (_object, transform, team, mut selected, mesh) = mut_obj;
                            if aabb.contains(transform.translation().xy()) {
                                if selected.is_selected() || *team != config.player_team {
                                    continue;
                                }
                                let child_entity = commands
                                    .spawn(Self::highlight_bundle(&assets, mesh.clone()))
                                    .id();
                                commands.entity(entity).add_child(child_entity);
                                *selected = Selected::Selected;
                            }
                        }
                    }
                }
                ButtonState::Released => {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }

    fn highlight_bundle(assets: &SelectorAssets, mesh: Handle<Mesh>) -> impl Bundle {
        (
            Name::new("Highlight"),
            Highlight,
            PbrBundle {
                mesh,
                transform: Transform::default().with_scale(Vec2::splat(1.2).extend(1.2)),
                material: assets.white_material.clone(),
                visibility: Visibility::Visible,
                ..default()
            },
        )
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

/// Handles to common grid assets.
#[derive(Resource)]
pub struct SelectorAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<StandardMaterial>,
    pub white_material: Handle<StandardMaterial>,
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
                materials.add(StandardMaterial::from(Color::BLUE.with_a(0.04)))
            },
            white_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::ALICE_BLUE.with_a(0.2)))
            },
        }
    }
}
