use bevy::pbr::NotShadowCaster;

use crate::prelude::*;
use std::marker::PhantomData;

/// Plugin for a 2D plane with a shader material.
pub struct ShaderPlanePlugin<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> Default for ShaderPlanePlugin<M> {
    fn default() -> Self {
        Self(PhantomData::<M>)
    }
}
impl<M: ShaderPlaneMaterial> Plugin for ShaderPlanePlugin<M>
where
    MaterialPlugin<M>: Plugin,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<M>::default())
            .init_resource::<ShaderPlaneAssets<M>>()
            .add_systems(Startup, M::setup)
            .add_systems(FixedUpdate, M::resize_on_change);
    }
}

/// Trait must be implemented by all Plane shaders.
pub trait ShaderPlaneMaterial: Material + FromWorld {
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

    fn setup(world: &mut World) {
        info!("ShaderPlaneMaterial::setup");
        let material = Self::from_world(world);
        let spec = world.get_resource::<GridSpec>().unwrap();
        let assets = world.get_resource::<ShaderPlaneAssets<Self>>().unwrap();
        let bundle = material.bundle(spec, assets);
        let mut plane = world.spawn(bundle);

        if Self::raycast_target() != RaycastTarget::None {
            plane.insert(Self::raycast_target());
        }
    }

    /// When the spec is changed, respawn the visualizer entity with the new size.
    fn resize_on_change(
        spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut transforms: Query<&mut Transform, With<ShaderPlane<Self>>>,
        mut shader_assets: ResMut<Assets<Self>>,
    ) {
        if !spec.is_changed() {
            return;
        }
        info!("ShaderPlaneMaterial::resize_on_change");

        let mut transform = transforms.single_mut();
        transform.scale = Self::scale(&spec);
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&spec);
    }

    fn bundle(self, spec: &GridSpec, assets: &ShaderPlaneAssets<Self>) -> impl Bundle {
        let material = assets.shader_material.clone();
        (
            MaterialMeshBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default()
                    .with_scale(Self::scale(spec))
                    .with_translation(Self::translation(spec)),
                material,
                ..default()
            },
            NotShadowCaster,
            Name::new(std::any::type_name::<ShaderPlane<Self>>()),
            ShaderPlane::<Self>::default(),
        )
    }
}

/// Component that marks an entity as a shader plane.
#[derive(Debug, Component, Clone)]
pub struct ShaderPlane<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> Default for ShaderPlane<M> {
    fn default() -> Self {
        Self(PhantomData::<M>)
    }
}

/// Handles to shader plane assets.
#[derive(Resource)]
pub struct ShaderPlaneAssets<M: ShaderPlaneMaterial> {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<M>,
}
impl<M: ShaderPlaneMaterial> FromWorld for ShaderPlaneAssets<M> {
    fn from_world(world: &mut World) -> Self {
        let assets = Self {
            mesh: {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(meshes::UNIT_SQUARE))
            },
            shader_material: {
                let material = M::from_world(world);
                let mut materials = world.get_resource_mut::<Assets<M>>().unwrap();
                materials.add(material)
            },
        };
        assets
    }
}
