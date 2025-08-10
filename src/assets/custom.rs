use bevy::ecs::resource::Resource;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]

pub struct CustomAssets {
    #[asset(path = "bevylogo.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub bevy_logo: Handle<Image>,

    #[asset(path = "forest-tilemap.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub forest_tilemap: Handle<Image>,

    #[asset(path = "tree.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub tree: Handle<Image>,

    #[asset(path = "leaves.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub leaves: Handle<Image>,

    #[asset(path = "dino-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub dino: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 128, tile_size_y = 128, columns = 5, rows = 6))]
    pub dino_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "circle-transition.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub circle_transition: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 512, tile_size_y = 320, columns = 7, rows = 2))]
    pub circle_transition_layout: Handle<TextureAtlasLayout>,
}
