use bevy::ecs::resource::Resource;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]

pub struct CustomAssets {
    #[asset(path = "bevylogo.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub _bevy_logo: Handle<Image>,

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

    #[asset(path = "apple.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub apple: Handle<Image>,

    #[asset(path = "appleicon.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub appleicon: Handle<Image>,

    #[asset(path = "flag.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub flag: Handle<Image>,

    #[asset(path = "clock.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub clock: Handle<Image>,

    #[asset(path = "dinoicon.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub dinoicon: Handle<Image>,

    #[asset(path = "circle-transition.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub circle_transition: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 512, tile_size_y = 320, columns = 7, rows = 2))]
    pub circle_transition_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "sfx/tropical.ogg")]
    pub music: Handle<AudioSource>,

    #[asset(path = "sfx/win.ogg")]
    pub win: Handle<AudioSource>,

    #[asset(path = "sfx/lose.ogg")]
    pub lose: Handle<AudioSource>,

    #[asset(path = "sfx/menu.ogg")]
    pub menu_music: Handle<AudioSource>,

    #[asset(path = "sfx/collect.ogg")]
    pub collect_sfx: Handle<AudioSource>,

    #[asset(path = "sfx/walk1.ogg")]
    pub walk1: Handle<AudioSource>,
    #[asset(path = "sfx/walk2.ogg")]
    pub walk2: Handle<AudioSource>,
    #[asset(path = "sfx/walk3.ogg")]
    pub walk3: Handle<AudioSource>,
    #[asset(path = "sfx/walk4.ogg")]
    pub walk4: Handle<AudioSource>,
    #[asset(path = "sfx/walk5.ogg")]
    pub walk5: Handle<AudioSource>,
    #[asset(path = "sfx/walk6.ogg")]
    pub walk6: Handle<AudioSource>,
    #[asset(path = "sfx/walk7.ogg")]
    pub walk7: Handle<AudioSource>,
    #[asset(path = "sfx/walk8.ogg")]
    pub walk8: Handle<AudioSource>,
    #[asset(path = "sfx/walk9.ogg")]
    pub walk9: Handle<AudioSource>,
    #[asset(path = "sfx/walk10.ogg")]
    pub walk10: Handle<AudioSource>,

    #[asset(path = "sfx/boingjump1.ogg")]
    pub boingjump1: Handle<AudioSource>,
    #[asset(path = "sfx/boingjump2.ogg")]
    pub boingjump2: Handle<AudioSource>,

    #[asset(path = "sfx/impact1.ogg")]
    pub impact1: Handle<AudioSource>,
    #[asset(path = "sfx/impact2.ogg")]
    pub impact2: Handle<AudioSource>,
    #[asset(path = "sfx/impact3.ogg")]
    pub impact3: Handle<AudioSource>,

    #[asset(path = "sfx/jump1.ogg")]
    pub swoosh1: Handle<AudioSource>,
    #[asset(path = "sfx/jump2.ogg")]
    pub swoosh2: Handle<AudioSource>,
    #[asset(path = "sfx/jump2.ogg")]
    pub swoosh3: Handle<AudioSource>,
    #[asset(path = "sfx/jump4.ogg")]
    pub swoosh4: Handle<AudioSource>,

    #[asset(path = "sfx/thud1.ogg")]
    pub thud1: Handle<AudioSource>,
    #[asset(path = "sfx/thud2.ogg")]
    pub thud2: Handle<AudioSource>,
    #[asset(path = "sfx/thud3.ogg")]
    pub thud3: Handle<AudioSource>,
}
