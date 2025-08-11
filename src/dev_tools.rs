use crate::app::AppState;
use crate::game::Apple;
use crate::game::Dino;
use crate::game::GameState;
use crate::game::GameStatus;
use crate::game::Obstacle;
use crate::game::SceneChange;
use bevy::dev_tools::states::log_transitions;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<AppState>)
        .add_systems(Update, lose.run_if(input_just_pressed(KeyCode::KeyX)))
        .add_systems(Update, win.run_if(input_just_pressed(KeyCode::KeyZ)))
        .add_systems(
            PostUpdate,
            draw_aabb_gizmos.run_if(in_state(AppState::Game)),
        );
}

fn lose(
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    *game_status = GameStatus::Lose;
    game_state.set(GameState::NotRunning);
    commands.send_event(SceneChange(AppState::GameOver));
}

fn win(
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    *game_status = GameStatus::Win;
    game_state.set(GameState::NotRunning);
    commands.send_event(SceneChange(AppState::GameOver));
}

pub fn draw_aabb_gizmos(
    mut gizmos: Gizmos,
    query: Query<&Dino>,
    obstacles: Query<&Obstacle>,
    apples: Query<&Apple>,
) {
    for apple in apples.iter() {
        let min = apple.aabb.min.extend(0.0);
        let max = apple.aabb.max.extend(0.0);
        let points = [
            min,
            Vec3::new(max.x, min.y, 0.0),
            max,
            Vec3::new(min.x, max.y, 0.0),
            min,
        ];
        gizmos.linestrip(points, bevy::color::palettes::css::BLUE);
    }

    for obstacle in obstacles.iter() {
        let min = obstacle.aabb.min.extend(0.0);
        let max = obstacle.aabb.max.extend(0.0);
        let points = [
            min,
            Vec3::new(max.x, min.y, 0.0),
            max,
            Vec3::new(min.x, max.y, 0.0),
            min,
        ];
        gizmos.linestrip(points, bevy::color::palettes::css::BLUE);
    }

    for dino in query.iter() {
        let min = dino.aabb.min.extend(0.0);
        let max = dino.aabb.max.extend(0.0);
        let points = [
            min,
            Vec3::new(max.x, min.y, 0.0),
            max,
            Vec3::new(min.x, max.y, 0.0),
            min,
        ];
        gizmos.linestrip(points, bevy::color::palettes::css::RED);
    }
}
