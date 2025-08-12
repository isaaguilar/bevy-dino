use super::ChangeMenu;
use crate::app::AppState;
use crate::app::DisplayLanguage;
use crate::game::PostHighScore;
use crate::game::SceneChange;

use bevy::prelude::*;

pub fn start_game(mut commands: Commands) {
    commands.send_event(SceneChange(AppState::Game));
}

pub fn language_selection(
    mut display_language: ResMut<DisplayLanguage>,
    language: impl Into<String>,
) {
    display_language.0 = language.into();
}

pub fn menu_selection(mut change_menu: EventWriter<ChangeMenu>, menu: impl Into<String>) {
    change_menu.write(ChangeMenu::new(menu.into()));
}

pub fn show_leaderboard(mut commands: Commands) {
    commands.send_event(PostHighScore);
}

pub fn show_credits(mut commands: Commands) {
    commands.send_event(SceneChange(AppState::Credits));
}
