use bevy::prelude::*;
use std::collections::HashMap;

use crate::app::AppState;

// pub mod game;
pub mod menu;
pub mod splash;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        menu::plugin,
        // game::plugin,
        splash::plugin,
    ));
    app.add_systems(Update, preload.run_if(in_state(AppState::Preload)));
}

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub struct LexiCollection<A: Asset + TypePath>(pub Vec<Handle<A>>);

impl<A: Asset + TypePath> LexiCollection<A> {
    fn new(asset_server: &Res<AssetServer>, files: Vec<impl Into<String>>) -> Self
    where
        A: Asset + TypePath,
    {
        let handles = files
            .into_iter()
            .map(|file| asset_server.load(&file.into()))
            .collect::<Vec<_>>();

        Self(handles)
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Lexicon {
    pub translations: HashMap<String, String>,
}

impl Lexicon {
    pub fn from_language(&self, language: impl Into<String>) -> String {
        self.translations
            .get(&language.into())
            .cloned()
            .unwrap_or(String::new())
    }
}

fn preload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    info!("Loading levels");
    // let level = crate::level::LevelHandle(asset_server.load("levels.json"));
    // commands.insert_resource(level);

    app_state.set(AppState::Loading);
}
