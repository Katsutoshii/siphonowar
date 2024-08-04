use crate::prelude::*;

pub struct ObjectBackgroundPlugin;
impl Plugin for ObjectBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (ObjectBackground::update,)
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}
#[derive(Component, Default)]
pub struct ObjectBackground;
impl ObjectBackground {
    pub fn update(
        mut query: Query<(Entity, &mut Transform, Option<&Parent>), With<Self>>,
        parents: Query<(&GlobalTransform, &Velocity), With<Children>>,
    ) {
        for (entity, mut transform, parent) in &mut query {
            if let Some(parent) = parent {
                if let Ok((parent_transform, parent_velocity)) = parents.get(parent.get()) {
                    let offset = -0.1 * parent_velocity.extend(0.);
                    let inverse = parent_transform.compute_transform().rotation.inverse();
                    let result = inverse.mul_vec3(offset);
                    transform.translation.x = result.x;
                    transform.translation.y = result.y;
                }
            } else {
                info!("No parent, despawn background! {:?}", entity);
            }
        }
    }
}

#[derive(Bundle)]
pub struct BackgroundBundle {
    pub name: Name,
    pub background: ObjectBackground,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
}
impl Default for BackgroundBundle {
    fn default() -> Self {
        Self {
            name: Name::new("ObjectBackground"),
            background: ObjectBackground,
            mesh: Handle::<Mesh>::default(),
            material: Handle::<StandardMaterial>::default(),
            transform: Transform {
                scale: Vec3::splat(1.49),
                translation: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: -0.1,
                },
                ..default()
            },
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
