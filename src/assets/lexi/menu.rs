use crate::{
    app::AppState,
    assets::lexi::{LexiCollection, Lexicon},
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(JsonAssetPlugin::<MenuData>::new(&[".json"]));
    app.add_systems(OnEnter(AppState::Preload), preload);
}

fn preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading menu");
    commands.insert_resource(LexiCollection::<MenuData>::new(
        &asset_server,
        vec![
            "lexi/menu/main.json",
            "lexi/menu/language.json",
            "lexi/menu/howto.json",
            // ...more menus here,
        ],
    ));
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct MenuData {
    pub id: String,
    pub lex: Lexicon,
    pub choices: Option<Vec<Choice>>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Display {}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Choice {
    pub id: String,
    pub choice: ChoiceLex,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct ChoiceLex {
    pub lex: Lexicon,
    pub action: Option<String>,
    pub next_id: Option<String>,
}
