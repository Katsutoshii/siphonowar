// Allows spawning enum bundles as trees.
use bevy::{ecs::system::EntityCommands, prelude::*};
pub use from_enum::From;

/// Spawn a tree of bundles.
/// We don't use the Bundle trait directly because that trait doesn't support enums.
#[derive(Clone)]
pub struct BundleTree<B: SpawnableBundle> {
    pub bundle: B,
    pub children: Vec<BundleTree<B>>,
}
impl<B: SpawnableBundle> Default for BundleTree<B> {
    fn default() -> Self {
        Self {
            bundle: B::default(),
            children: Vec::default(),
        }
    }
}
impl<B: SpawnableBundle> BundleTree<B> {
    pub fn new(bundle: impl Into<B>) -> Self {
        Self {
            bundle: bundle.into(),
            children: Vec::default(),
        }
    }
    pub fn with_children(mut self, children: Vec<Self>) -> Self {
        self.children = children;
        self
    }
}

/// A bundle that can be spawned via commands.
pub trait SpawnableBundle: Default + Sized {
    fn spawn<'c>(self, commands: &'c mut Commands) -> EntityCommands<'c>;
    fn spawn_child<'c>(self, commands: &'c mut ChildBuilder<'_>) -> EntityCommands<'c>;
}

/// Trait for building a tree from a struct with custom context.
pub trait MakeBundleTree<B: SpawnableBundle, Context> {
    fn tree(self, context: Context) -> BundleTree<B>;
}

/// Trait for using commands to spawn BundleTree<B>.
pub trait BundleTreeSpawner {
    fn spawn_tree<B: SpawnableBundle>(&mut self, tree: BundleTree<B>) -> EntityCommands;
}
impl BundleTreeSpawner for Commands<'_, '_> {
    fn spawn_tree<B: SpawnableBundle>(&mut self, tree: BundleTree<B>) -> EntityCommands {
        let mut commands = tree.bundle.spawn(self);
        commands.with_children(|parent| {
            for child in tree.children.into_iter() {
                parent.spawn_tree(child);
            }
        });
        commands
    }
}

/// Trait for using child builder to spawn BundleTree<B>
pub trait BundleTreeChildSpawner {
    fn spawn_tree<B: SpawnableBundle>(&mut self, tree: BundleTree<B>);
}
impl BundleTreeChildSpawner for ChildBuilder<'_> {
    /// Requires that self outlives the builder.
    fn spawn_tree<B: SpawnableBundle>(&mut self, tree: BundleTree<B>) {
        tree.bundle.spawn_child(self).with_children(|parent| {
            for child in tree.children.into_iter() {
                parent.spawn_tree(child);
            }
        });
    }
}
