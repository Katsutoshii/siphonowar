use bevy::{asset::AssetPath, prelude::*};

pub trait WorldAssets {
    fn add_asset<A: Asset>(&mut self, asset: A) -> Handle<A>;
    fn load_asset<'a, A: Asset>(&mut self, asset_path: impl Into<AssetPath<'a>>) -> Handle<A>;
    fn assets<A: Asset>(&mut self) -> Mut<Assets<A>>;
    fn asset_server<A: Asset>(&mut self) -> Mut<AssetServer>;
}

impl WorldAssets for World {
    fn add_asset<A: Asset>(&mut self, asset: A) -> Handle<A> {
        let mut assets = self.get_resource_mut::<Assets<A>>().unwrap();
        assets.add(asset)
    }
    fn load_asset<'a, A: Asset>(&mut self, path: impl Into<AssetPath<'a>>) -> Handle<A> {
        let server = self.get_resource_mut::<AssetServer>().unwrap();
        server.load(path)
    }
    fn assets<A: Asset>(&mut self) -> Mut<Assets<A>> {
        self.get_resource_mut::<Assets<A>>().unwrap()
    }
    fn asset_server<A: Asset>(&mut self) -> Mut<AssetServer> {
        self.get_resource_mut::<AssetServer>().unwrap()
    }
}
