use crate::app::AppState;
use crate::game::Dino;
use crate::game::Obstacle;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        draw_aabb_gizmos.run_if(in_state(AppState::Game)),
    );
}

pub fn draw_aabb_gizmos(mut gizmos: Gizmos, query: Query<&Dino>, obstacles: Query<&Obstacle>) {
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
