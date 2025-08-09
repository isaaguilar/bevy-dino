use crate::{
    app::AppState,
    assets::lexi::{LexiCollection, Lexicon},
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(JsonAssetPlugin::<SplashLex>::new(&[".json"]));
    // app.add_plugins(JsonAssetPlugin::<Menu>::new(&[".json"]));
    app.add_systems(OnEnter(AppState::Preload), preload);
}

fn preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading Splash Lex");
    commands.insert_resource(LexiCollection::<SplashLex>::new(
        &asset_server,
        vec!["lexi/splash/splash.json"],
    ));
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct SplashLex {
    pub id: String,
    pub lex: Lexicon,
}
