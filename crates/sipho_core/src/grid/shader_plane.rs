use crate::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};
use std::marker::PhantomData;

/// Plugin for a 2D plane with a shader material.
#[derive(Default)]
pub struct ShaderPlanePlugin<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> Plugin for ShaderPlanePlugin<M>
where
    Material2dPlugin<M>: Plugin,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<M>::default())
            .init_resource::<ShaderPlaneAssets<M>>()
            .add_systems(FixedUpdate, M::resize_on_change);
    }
}

/// Trait must be implemented by all Plane shaders.
pub trait ShaderPlaneMaterial: Material2d + Default {
    /// If true, this grid shader will have the camera as a parent.
    fn parent_camera() -> bool {
        false
    }

    /// If true, receive raycasts.
    fn raycast_target() -> RaycastTarget {
        RaycastTarget::None
    }

    /// Scale factor
    fn scale(_window: &Window, spec: &GridSpec) -> Vec3 {
        spec.scale().extend(1.)
    }

    /// Translation
    fn translation(_window: &Window, spec: &GridSpec) -> Vec3;

    /// Resize the grid based on the grid spec.
    fn resize(&mut self, spec: &GridSpec);

    /// When the spec is changed, respawn the visualizer entity with the new size.
    fn resize_on_change(
        spec: Res<GridSpec>,
        query: Query<Entity, With<ShaderPlane<Self>>>,
        camera: Query<Entity, With<MainCamera>>,
        assets: Res<ShaderPlaneAssets<Self>>,
        window: Query<&Window, With<PrimaryWindow>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }

        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&spec);

        let plane_entity = {
            let window = window.single();
            let mut plane =
                commands.spawn(ShaderPlane::<Self>::default().bundle(&spec, window, &assets));
            if Self::raycast_target() != RaycastTarget::None {
                plane.insert(Self::raycast_target());
            }
            plane.id()
        };
        if Self::parent_camera() {
            let camera_entity = camera.single();
            commands
                .entity(camera_entity)
                .push_children(&[plane_entity]);
        }
    }
}

/// Component that marks an entity as a shader plane.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct ShaderPlane<M: ShaderPlaneMaterial>(PhantomData<M>);
impl<M: ShaderPlaneMaterial> ShaderPlane<M> {
    pub fn bundle(
        self,
        spec: &GridSpec,
        window: &Window,
        assets: &ShaderPlaneAssets<M>,
    ) -> impl Bundle {
        let material = assets.shader_material.clone();
        (
            MaterialMesh2dBundle::<M> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(M::scale(window, spec))
                    .with_translation(M::translation(window, spec)),
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
pub struct ShaderPlaneAssets<M: Material2d> {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<M>,
}
impl<M: Material2d + Default> FromWorld for ShaderPlaneAssets<M> {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(meshes::UNIT_SQUARE))
        };
        let shader_material = {
            let mut materials = world.get_resource_mut::<Assets<M>>().unwrap();
            materials.add(M::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}
