use crate::{
    app::AppState,
    assets::lexi::{LexiCollection, Lexicon},
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(JsonAssetPlugin::<GameOverLex>::new(&[".json"]));
    app.add_systems(OnEnter(AppState::Preload), preload);
}

fn preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading Game Over Options");
    commands.insert_resource(LexiCollection::<GameOverLex>::new(
        &asset_server,
        vec![
            "lexi/game-over/win.json",
            "lexi/game-over/lose.json",
            "lexi/game-over/submit.json",
        ],
    ));
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct GameOverLex {
    pub id: String,
    pub lex: Lexicon,
}
