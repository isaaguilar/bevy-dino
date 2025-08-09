use crate::app::{AppState, RESOLUTION_HEIGHT, RESOLUTION_WIDTH};
use crate::game::Player;
use bevy::core_pipeline::bloom::{Bloom, BloomPrefilter};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

#[derive(Component, Default)]
pub struct GameCamera {
    pub selected_game_level: GameLevelDimensions,
}

#[derive(Component, Default)]
pub struct GameLevelDimensions {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub fn game_camera(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    if let Ok(_) = camera_query.single_mut() {
        return;
    }

    commands
        .spawn((
            StateScoped(AppState::Game),
            GameCamera {
                selected_game_level: GameLevelDimensions {
                    left: -1000000.,   // Camera views -180 pixels left
                    top: 1000000.,     // Camera views 90 pixels up (top)
                    right: 1000000.,   // Camera views 1600 + 180 pixels right
                    bottom: -1000000., // Camera views 90 pixels down (bottom)
                },
                ..default()
            },
            Camera2d::default(),
            // Camera {
            //     hdr: true, // 1. HDR is required for bloom
            //     clear_color: ClearColorConfig::Custom(Color::BLACK),
            //     ..default()
            // },
            // Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            // Bloom {
            //     intensity: 0.0045,
            //     prefilter: BloomPrefilter {
            //         threshold: 0.14,
            //         threshold_softness: 0.32,
            //     },
            //     ..default() // low_frequency_boost: todo!(),
            //                 // low_frequency_boost_curvature: todo!(),
            //                 // high_pass_frequency: todo!(),
            //                 // prefilter: todo!(),
            //                 // composite_mode: todo!(),
            //                 // max_mip_dimension: todo!(),
            //                 // scale: todo!(),
            // }, // 3. Enable bloom for the camera
            // DebandDither::Enabled, // Optional: bloom causes gradients which cause banding
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: RESOLUTION_WIDTH,
                    min_height: RESOLUTION_HEIGHT,
                },
                scale: 1.0,
                near: -1000.,
                far: 1000.,
                ..OrthographicProjection::default_2d()
            }),
        ))
        .insert(Transform::from_xyz(0., 0., 0.));
}

pub fn camera_tracking_system(
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform), Without<Player>>,
) {
    // TODO track two players that have a diff < screen height else game over

    let camera_above_center_const = 75.0;
    // let player_transform = match {
    //     Ok(t) => t,
    //     Err(_) => return,
    // };
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    let player_average_position = &player_transform.translation;

    let (game_camera, mut camera_transform) = match camera_query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    if camera_transform.translation.is_nan() {
        info!("Init camera");
        camera_transform.translation.x = player_average_position.x;
        camera_transform.translation.y = player_average_position.y + camera_above_center_const;
        return;
    }

    let (max, min) = (
        game_camera.selected_game_level.top,
        game_camera.selected_game_level.bottom,
    );
    let m = 1.0_f32;
    let k = 8.5_f32;
    let b = 2.0 * (m * k).sqrt();
    let camera_position = camera_transform.translation.y;
    let player_position = player_average_position.y + camera_above_center_const;
    let delta = camera_position - player_position;
    if delta.abs() < 0.10 {
        let final_position = player_average_position.y + camera_above_center_const;
        if final_position < max && final_position > min {
            camera_transform.translation.y = player_average_position.y + camera_above_center_const;
        }
    } else {
        let mut v = delta / time.delta_secs();

        let a = -b * v - k * delta;
        v += a * time.delta_secs();
        let d = v * time.delta_secs();
        let final_position = player_position + d;

        // Add conditional here for max y
        if final_position < max && final_position > min {
            camera_transform.translation.y = final_position;
        }
    }

    let (max, min) = (
        game_camera.selected_game_level.right,
        game_camera.selected_game_level.left,
    );
    let m = 1.0_f32;
    let k = 8.5_f32;
    let b = 2.0 * (m * k).sqrt();
    let camera_position = camera_transform.translation.x;
    let player_position = player_average_position.x;
    let delta = camera_position - player_position;
    if delta.abs() < 0.10 {
        let final_position = player_average_position.x;
        if final_position < max && final_position > min {
            camera_transform.translation.x = player_average_position.x;
        }
    } else {
        let mut v = delta / time.delta_secs();

        let a = -b * v - k * delta;
        v += a * time.delta_secs();
        let d = v * time.delta_secs();
        let final_position = player_position + d;

        // Add conditional here for max y

        if final_position < max && final_position > min {
            camera_transform.translation.x = final_position;
        }
    }

    let max_x = game_camera.selected_game_level.right * 0.9;
    let min_x = game_camera.selected_game_level.left * 0.9;
    let max_y = game_camera.selected_game_level.top * 0.9;
    let min_y = game_camera.selected_game_level.bottom * 0.9;

    if player_transform.translation.x > max_x {
        player_transform.translation.x = min_x;
        camera_transform.translation.x += 2. * min_x;
    }
    if player_transform.translation.y > max_y {
        player_transform.translation.y = min_y;
        camera_transform.translation.y += 2. * min_y;
    }
    if player_transform.translation.x < min_x {
        player_transform.translation.x = max_x;
        camera_transform.translation.x += 2. * max_x;
    }
    if player_transform.translation.y < min_y {
        player_transform.translation.y = max_y;
        camera_transform.translation.y += 2. * max_y;
    }
}

#[derive(Component)]
pub struct Parallax {
    pub x_coeff: f32,
    pub y_coeff: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_tile: i32,
    pub y_tile: i32,
}

pub fn parallax_system(
    camera_query: Query<&Transform, With<GameCamera>>,
    mut background_query: Query<(&mut Transform, &Parallax), Without<GameCamera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (mut background_transform, parallax) in background_query.iter_mut() {
        background_transform.translation.x = camera_transform.translation.x * parallax.x_coeff
            + parallax.x_offset * (parallax.x_tile as f32);
        background_transform.translation.y = camera_transform.translation.y * parallax.y_coeff
            + parallax.y_offset * (parallax.y_tile as f32);
    }
}
