use crate::prelude::*;
use std::marker::PhantomData;

/// Plugin for a 2D plane with a shader material.
#[derive(Default)]
pub struct ShaderPlanePlugin<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> Plugin for ShaderPlanePlugin<M>
where
    MaterialPlugin<M>: Plugin,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<M>::default())
            .init_resource::<ShaderPlaneAssets<M>>()
            .add_systems(FixedUpdate, M::resize_on_change);
    }
}

/// Trait must be implemented by all Plane shaders.
pub trait ShaderPlaneMaterial: Material + Default {
    /// If true, receive raycasts.
    fn raycast_target() -> RaycastTarget {
        RaycastTarget::None
    }

    /// Scale factor
    fn scale(spec: &GridSpec) -> Vec3 {
        spec.scale().extend(1.)
    }

    /// Translation
    fn translation(spec: &GridSpec) -> Vec3;

    /// Resize the grid based on the grid spec.
    fn resize(&mut self, spec: &GridSpec);

    /// When the spec is changed, respawn the visualizer entity with the new size.
    fn resize_on_change(
        spec: Res<GridSpec>,
        mut query: Query<(Entity, &mut Visibility), With<ShaderPlane<Self>>>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for (entity, mut visibility) in query.iter_mut() {
            *visibility = Visibility::Hidden;
            commands.entity(entity).despawn();
        }

        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&spec);

        let mut plane = commands.spawn(ShaderPlane::<Self>::default().bundle(&spec, &assets));
        if Self::raycast_target() != RaycastTarget::None {
            plane.insert(Self::raycast_target());
        }
    }
}

/// Component that marks an entity as a shader plane.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct ShaderPlane<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> ShaderPlane<M> {
    pub fn bundle(self, spec: &GridSpec, assets: &ShaderPlaneAssets<M>) -> impl Bundle {
        let material = assets.shader_material.clone();
        (
            MaterialMeshBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default()
                    .with_scale(M::scale(spec))
                    .with_translation(M::translation(spec)),
                material,
                ..default()
            },
            Name::new(std::any::type_name::<Self>()),
            self,
        )
    }
}

/// Handles to shader plane assets.
#[derive(Resource)]
pub struct ShaderPlaneAssets<M: Material> {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<M>,
}
impl<M: Material + Default> FromWorld for ShaderPlaneAssets<M> {
    fn from_world(world: &mut World) -> Self {
        let assets = Self {
            mesh: {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(meshes::UNIT_SQUARE))
            },
            shader_material: {
                let mut materials = world.get_resource_mut::<Assets<M>>().unwrap();
                materials.add(M::default())
            },
        };
        assets
    }
}
