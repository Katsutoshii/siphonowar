// Allows spawning enum bundles as trees.
use bevy::{ecs::system::EntityCommands, prelude::*};
pub use from_enum::From;

/// Spawn a tree of bundles.
/// We don't use the Bundle trait directly because that trait doesn't support enums.
#[derive(Clone)]
pub struct BundleTree<B: BundleEnum> {
    pub bundle: B,
    pub children: Vec<BundleTree<B>>,
}
impl<B: BundleEnum> BundleTree<B> {
    pub fn new(bundle: impl Into<B>) -> Self {
        Self {
            bundle: bundle.into(),
            children: Vec::default(),
        }
    }
    pub fn with_children(mut self, children: impl IntoIterator<Item = BundleTree<B>>) -> Self {
        self.children = children.into_iter().collect();
        self
    }
}

/// A bundle that can be spawned via commands.
pub trait BundleEnum: Sized {
    fn spawn<'c>(self, commands: &'c mut Commands) -> EntityCommands<'c>;
}

/// Trait for using commands to spawn BundleTree<B>.
pub trait BundleTreeSpawner {
    /// Spawns a BundleTree and returns the EntityCommands for the root.
    fn spawn_tree<B: BundleEnum>(&mut self, tree: BundleTree<B>) -> EntityCommands;
}
impl BundleTreeSpawner for Commands<'_, '_> {
    fn spawn_tree<B: BundleEnum>(&mut self, tree: BundleTree<B>) -> EntityCommands {
        let BundleTree { bundle, children } = tree;

        let mut child_ids: Vec<Entity> = Vec::with_capacity(children.len());
        for child in children.into_iter() {
            let entity = self.spawn_tree(child).id();
            child_ids.push(entity);
        }

        let mut e = bundle.spawn(self);
        e.push_children(&child_ids);
        e
    }
}

/// Trait for building a tree from a struct with custom context.
pub trait MakeBundleTree<B: BundleEnum, Context> {
    /// Returns the BundleTree associated with the given type.
    fn tree(self, context: Context) -> BundleTree<B>;
}
