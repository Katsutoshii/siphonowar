use bevy::{ecs::system::SystemParam, prelude::*, ui::RelativeCursorPosition, utils::FloatOrd};

use crate::{Grid2, Team, TeamEntitySets};

#[derive(Component, Default, PartialEq, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub enum RaycastTarget {
    #[default]
    None,
    WorldGrid,
    Minimap,
    GridEntity,
}

#[derive(Component, Default)]
pub struct GridRaycastTarget;

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct RaycastCommands<'w, 's> {
    pub meshes: Query<
        'w,
        's,
        (
            Entity,
            &'static RaycastTarget,
            &'static Handle<Mesh>,
            &'static GlobalTransform,
        ),
    >,
    pub grid_meshes: Query<
        'w,
        's,
        (
            Entity,
            &'static GridRaycastTarget,
            &'static Handle<Mesh>,
            &'static GlobalTransform,
        ),
    >,
    pub assets: Res<'w, Assets<Mesh>>,
    pub relative_positions: Query<
        'w,
        's,
        (
            Entity,
            &'static RelativeCursorPosition,
            &'static RaycastTarget,
        ),
    >,
    pub grid: Res<'w, Grid2<TeamEntitySets>>,
}
impl RaycastCommands<'_, '_> {
    /// Raycast using grid position for retrieval.
    pub fn grid_raycast(&self, ray: Ray3d, event: &RaycastEvent) -> Option<RaycastEvent> {
        let mut hits = Vec::default();

        let radius = 32.;
        let n = 1;
        let entities =
            self.grid
                .get_n_entities_in_radius(event.world_position, radius, &Team::ALL, n);
        for (entity, _target, mesh_handle, transform) in entities
            .iter()
            .filter_map(|&entity| self.grid_meshes.get(entity).ok())
        {
            let mesh = self.assets.get(mesh_handle).unwrap();

            let mut local_transform = transform.compute_transform();
            if local_transform.scale.x <= 20. {
                local_transform.scale *= 6.0;
            } else {
                local_transform.scale *= 3.0;
            }
            local_transform.scale.z /= 2.0;

            let mesh_to_world = local_transform.compute_matrix();
            if let Some(intersection) = bevy_mod_raycast::prelude::ray_intersection_over_mesh(
                mesh,
                &mesh_to_world,
                ray,
                bevy_mod_raycast::prelude::Backfaces::Cull,
            ) {
                let distance = FloatOrd(intersection.distance());
                let event = RaycastEvent {
                    entity,
                    position: intersection.position().xy(),
                    world_position: intersection.position().xy(),
                    target: RaycastTarget::GridEntity,
                };
                hits.push((distance, event))
            }
        }
        hits.iter()
            .min_by_key(|&(distance, _)| distance)
            .map(|(_, event)| event.clone())
    }

    /// Raycast using UI
    pub fn ui_raycast(&self) -> Option<RaycastEvent> {
        for (entity, relative_position, raycast_target) in self.relative_positions.iter() {
            if relative_position.mouse_over() {
                if let Some(position) = relative_position.normalized {
                    return Some(RaycastEvent {
                        entity,
                        world_position: Vec2::ZERO,
                        position,
                        target: *raycast_target,
                    });
                }
            }
        }
        None
    }
    pub fn raycast(&self, ray: Ray3d) -> Option<RaycastEvent> {
        let worldcast = self.world_grid_raycast(ray);

        if let Some(mut event) = self.ui_raycast() {
            if let Some(worldcast) = worldcast {
                event.world_position = worldcast.world_position;
            }
            return Some(event);
        }
        worldcast
    }

    pub fn world_grid_raycast(&self, ray: Ray3d) -> Option<RaycastEvent> {
        if let Some(worldcast) = self.world_raycast(ray) {
            if let Some(gridcast) = self.grid_raycast(ray, &worldcast) {
                return Some(gridcast);
            }
            return Some(worldcast);
        }
        None
    }

    pub fn world_raycast(&self, ray: Ray3d) -> Option<RaycastEvent> {
        let mut hits = Vec::default();
        for (entity, &target, mesh_handle, transform) in self.meshes.iter() {
            let mesh = self.assets.get(mesh_handle).unwrap();
            let mesh_to_world = transform.compute_matrix();
            let world_to_mesh = mesh_to_world.inverse();
            if let Some(intersection) = bevy_mod_raycast::prelude::ray_intersection_over_mesh(
                mesh,
                &mesh_to_world,
                ray,
                bevy_mod_raycast::prelude::Backfaces::Cull,
            ) {
                let distance = FloatOrd(intersection.distance());
                let event = RaycastEvent {
                    entity,
                    position: world_to_mesh.transform_point3(intersection.position()).xy(),
                    world_position: intersection.position().xy(),
                    target,
                };
                hits.push((distance, event))
            }
        }

        hits.sort_by_key(|&(distance, _)| distance);
        if let Some((_distance, event)) = hits.first() {
            return Some(event.clone());
        }
        None
    }
}

#[derive(Event, Debug, Clone)]
pub struct RaycastEvent {
    pub entity: Entity,
    pub world_position: Vec2,
    pub position: Vec2,
    pub target: RaycastTarget,
}
