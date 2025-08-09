use crate::{assets::lexi::LexiCollection, client::AppState};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(JsonAssetPlugin::<GameScript>::new(&[".json"]));
    app.add_systems(OnEnter(AppState::Preload), preload);
    // app.add_systems(OnExit(AppState::Preload), get_data_test);
}

// fn get_data_test(data: Res<Assets<GameScript>>) {
//     info!("Getting test data");
//     for (_id, item) in data.into_inner().iter() {
//         info!(?item);
//     }
// }

fn preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading Game Dialog");
    commands.insert_resource(LexiCollection::<GameScript>::new(
        &asset_server,
        vec!["lexi/game/dialog.json"],
    ));
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct GameScript {
    pub dialogs: Vec<Dialog>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Dialog {
    pub id: String,
    pub name: String,
    pub events: Vec<String>,
    pub posessions: Vec<String>,
    pub choices: Option<Vec<Choice>>,
    pub language: Language,
    pub actions: Actions,
    pub display: Option<Display>,
    pub interaction_type: Option<String>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Display {
    pub background_color: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Actions {
    pub events_changed_on_enter: Vec<String>,
    pub events_changed_on_exit: Vec<String>,
    pub items_changed_on_enter: Vec<String>,
    pub items_changed_on_exit: Vec<String>,
    pub next_id: String,
    pub commands: Option<Vec<String>>,
    pub commands_on_enter: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Language {
    pub spanish: String,
    pub english: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Choice {
    pub choice: String,
    pub dialog: ChoiceDialog,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct ChoiceDialog {
    pub language: Language,
    pub actions: Actions,
}
