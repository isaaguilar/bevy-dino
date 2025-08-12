use crate::app::{AFTER_LOADING_STATE, AppState};
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    LoadingState, LoadingStateAppExt, config::ConfigureLoadingState,
};

pub mod custom;
pub mod lexi;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(lexi::plugin);
    app.add_loading_state(
        LoadingState::new(AppState::Loading)
            .continue_to_state(AFTER_LOADING_STATE)
            .load_collection::<custom::ImageAssets>()
            .load_collection::<custom::SoundAssets>(),
    );
}
