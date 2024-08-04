use crate::{prelude::*, ui::selector::HighlightBundle};
use bevy::prelude::*;
use bevy_bundletree::*;
use zooid_head::{HeadBundle, NearestZooidHead};
use zooid_worker::WorkerBundle;

use super::{background::BackgroundBundle, plankton::PlanktonBundle};

#[derive(Bundle, Default)]
pub struct ShockerBundle {
    pub nearest_head: NearestZooidHead,
    pub object: ObjectBundle,
}

#[derive(Bundle, Default)]
pub struct ArmorBundle {
    pub nearest_head: NearestZooidHead,
    pub object: ObjectBundle,
}

#[derive(Bundle, Default)]
pub struct FoodBundle {
    pub follower: PathToHeadFollower,
    pub object: ObjectBundle,
}

#[derive(BundleEnum, IntoBundleTree)]
pub enum ObjectTree {
    Worker(WorkerBundle),
    Shocker(ShockerBundle),
    Armor(ArmorBundle),
    Head(HeadBundle),
    Plankton(PlanktonBundle),
    Food(FoodBundle),
    Background(BackgroundBundle),
    Highlight(HighlightBundle),
}
impl ObjectTree {
    pub fn new(
        spec: ObjectSpec,
        mesh: Handle<Mesh>,
        team_material: TeamMaterials,
        config: &ObjectConfig,
        time: &Time,
    ) -> BundleTree<ObjectTree> {
        let background = BackgroundBundle {
            mesh: mesh.clone(),
            material: team_material.background.clone(),
            ..default()
        };
        let object_type = spec.object;
        let object = ObjectBundle {
            mesh: mesh.clone(),
            material: team_material.primary,
            ..ObjectBundle::new(config, spec, time)
        };
        match object_type {
            Object::Worker => WorkerBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::Shocker => ShockerBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::Armor => ArmorBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::Head => HeadBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::Plankton => PlanktonBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::Food => FoodBundle {
                object,
                ..default()
            }
            .with_children([background.into_tree()]),
            Object::BuilderPreview => unreachable!(),
        }
    }
}
