use crate::app::{AppState, HALF_WIDTH_SPRITE, RESOLUTION_HEIGHT, RESOLUTION_WIDTH, RUNNING_SPEED};
use crate::assets::custom::CustomAssets;
use crate::camera;
use bevy::ecs::system::Commands;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::platform::time;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::camera::{OrthographicProjection, Projection};
use bevy::render::primitives::Aabb as BevyAabb;
use bevy::scene::ron::de;
use bevy::sprite::Sprite;
use bevy::ui::{AlignItems, Display, FlexDirection, Node, PositionType, Val};
use bevy_aspect_ratio_mask::Hud;
use rand::Rng; // Make sure you have bevy_gizmos in your dependencies

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), (setup, camera::game_camera))
        .add_systems(
            Update,
            (
                dino_gravity,
                arrow_move,
                camera::camera_tracking_system,
                camera::parallax_system,
            )
                .run_if(in_state(AppState::Game)),
        );
}

pub fn setup(mut commands: Commands, assets: Res<CustomAssets>, hud: Res<Hud>) {
    info!("Setting up the game");

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                top: Val::Px(55.0),
                align_items: AlignItems::Center,
                ..default()
            },))
            .with_children(|p| {
                p.spawn(Text("Press Left / Right To Move\n\n".into()));
                p.spawn(Text("Press Space to jump. Press again to whip.\n\n".into()));
                p.spawn(Text("Resizing window maintains aspect ratio".into()));
            });
    });

    commands.spawn((
        Dino::default(),
        Player,
        // Add an aabb around the dino for collision detection.
        Sprite {
            image: assets.dino.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.dino_layout.clone(),
                index: 0,
                ..default()
            }),
            ..default()
        },
    ));

    // Add a ground line at 10 above the bottom of the screen.
    // Make this a physics object for ground collisions.
    // Use rapier2d for physics. The ground should span the width of the screen.
    // commands.spawn((
    //     Sprite {
    //         color: bevy::color::palettes::css::GREEN.into(),
    //         custom_size: Some(Vec2::new(RESOLUTION_WIDTH * 2., 20.)),
    //         ..default()
    //     },
    //     Transform::from_xyz(0., -RESOLUTION_HEIGHT / 2. + 10., -1.),
    // ));

    // TODO Tile this automatically based on player postion
    for i in -2..=2 {
        for j in -2..=2 {
            commands.spawn((
                camera::Parallax {
                    x_coeff: 0.6,
                    y_coeff: 0.6,
                    x_offset: 600. * 4.,
                    y_offset: 480. * 4.,
                    x_tile: i,
                    y_tile: j,
                },
                Sprite {
                    image: assets.forest_tilemap.clone(),
                    ..default()
                },
                Transform::from_xyz(0., 0., -20.),
            ));
        }
    }
    commands.spawn((
        camera::Parallax {
            x_coeff: 0.6,
            y_coeff: 0.6,
            x_offset: 600.,
            y_offset: 480.,
            x_tile: 0,
            y_tile: 0,
        },
        Sprite {
            image: assets.forest_tilemap.clone(),
            ..default()
        },
        Transform::from_xyz(0., 0., -20.),
    ));

    // Add a platform on top of the tree
    commands.spawn((
        Platform,
        Sprite {
            color: bevy::color::palettes::css::GREEN.into(),
            custom_size: Some(Vec2::new(300., 20.)),
            ..default()
        },
        Transform::from_xyz(0., -RESOLUTION_HEIGHT / 2. + 20., -1.),
        Obstacle {
            aabb: Aabb2d::new(
                Vec2::new(0., -RESOLUTION_HEIGHT / 2. + 20.),
                Vec2::new(150., 10.0),
            ),
        },
    ));

    // Spawn a tree in on the right of the screen (offset 20 pixels) at ground level
    // and make it 380 pixels tall. Wrap an aabb around it for collision detection.
    commands.spawn((
        Sprite {
            color: bevy::color::palettes::css::BROWN.into(),
            custom_size: Some(Vec2::new(50., 380.)),
            ..default()
        },
        Transform::from_xyz(
            RESOLUTION_WIDTH / 2. - 20.,
            -RESOLUTION_HEIGHT / 2. + 190.,
            -1.,
        ),
        Obstacle {
            aabb: Aabb2d::new(
                Vec2::new(RESOLUTION_WIDTH / 2. - 20., -RESOLUTION_HEIGHT / 2. + 190.),
                Vec2::new(25., 190.0),
            ),
        },
    ));

    // Add a platform on top of the tree
    commands.spawn((
        Platform,
        Sprite {
            color: bevy::color::palettes::css::DARK_GREEN.into(),
            custom_size: Some(Vec2::new(120., 20.)),
            ..default()
        },
        Transform::from_xyz(
            RESOLUTION_WIDTH / 2. - 20.,
            -RESOLUTION_HEIGHT / 2. + 380.,
            -1.,
        ),
        Obstacle {
            aabb: Aabb2d::new(
                Vec2::new(RESOLUTION_WIDTH / 2. - 20., -RESOLUTION_HEIGHT / 2. + 380.),
                Vec2::new(60., 10.0),
            ),
        },
    ));

    // Spawn a tree in on the right of the screen (offset 20 pixels) at ground level
    // and make it 380 pixels tall. Wrap an aabb around it for collision detection.
    commands.spawn((
        Sprite {
            color: bevy::color::palettes::css::BROWN.into(),
            custom_size: Some(Vec2::new(50., 380.)),
            ..default()
        },
        Transform::from_xyz(0., RESOLUTION_HEIGHT / 2., -1.),
        Obstacle {
            aabb: Aabb2d::new(Vec2::new(0., RESOLUTION_HEIGHT / 2.), Vec2::new(25., 190.0)),
        },
    ));

    // Add a platform on top of the tree
    commands.spawn((
        Platform,
        Sprite {
            color: bevy::color::palettes::css::DARK_GREEN.into(),
            custom_size: Some(Vec2::new(120., 20.)),
            ..default()
        },
        Transform::from_xyz(0., RESOLUTION_HEIGHT / 2., -1.),
        Obstacle {
            aabb: Aabb2d::new(Vec2::new(0., RESOLUTION_HEIGHT / 2.), Vec2::new(60., 10.0)),
        },
    ));
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone)]
pub struct Platform;

#[derive(Component, Debug, Clone)]
pub struct Obstacle {
    pub aabb: Aabb2d,
}

#[derive(Component, Debug, Clone)]
pub struct Dino {
    pub timer: Timer,
    pub idle_frame_forward: bool,
    pub grounded: bool,
    pub jump_acceleration_timer: Timer,
    pub velocity: Vec2,
    pub jumping: bool,
    pub jump_time: f32,
    pub jump_height: f32,
    pub attacking: bool,
    pub can_attack: bool,
    pub frame_hold_counter: Vec<(usize, u8, u8)>,
    pub aabb: Aabb2d,
}

impl Default for Dino {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.07, TimerMode::Repeating),
            idle_frame_forward: true,
            grounded: false,
            jump_acceleration_timer: Timer::from_seconds(0.5, TimerMode::Once),
            velocity: Vec2::ZERO,
            jumping: false,
            attacking: false,
            jump_height: 1500.0,
            frame_hold_counter: vec![(21, 0, 1)],
            can_attack: false,
            jump_time: 0.0,
            aabb: Aabb2d::new(Vec2::ZERO, Vec2::new(32., 32.)),
        }
    }
}

fn dino_gravity(
    mut dino: Query<(&mut Transform, &mut Dino), With<Sprite>>,
    platforms: Query<&Obstacle, With<Platform>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut dino)) = dino.single_mut() {
        let gravity = -1200.0;
        // Absolute ground y position
        let ground_y = -RESOLUTION_HEIGHT / 2. + 50.;

        // Apply gravity if not grounded
        if !dino.grounded {
            dino.velocity.y += gravity * time.delta_secs();
        }

        // Apply velocity
        let dy = dino.velocity.y * time.delta_secs();
        transform.translation.y += dy;
        dino.aabb.min.y += dy;
        dino.aabb.max.y += dy;

        if transform.translation.y < -500. {
            dino.velocity = Vec2::ZERO;
            transform.translation = Vec3::ZERO;
            dino.aabb = Aabb2d::new(Vec2::ZERO, Vec2::new(32., 32.));
        }

        // Check collision with absolute ground
        // if transform.translation.y <= ground_y {
        //     transform.translation.y = ground_y;
        //     dino.velocity.y = 0.0;
        //     dino.grounded = true;
        // } else {
        // Check collision with obstacles (platforms)
        let mut landed = false;
        for platform in platforms.iter() {
            // First check if this obstacle is withint the x range of the dino
            let dino_left = dino.aabb.min.x;
            let dino_right = dino.aabb.max.x;
            let obstacle_left = platform.aabb.min.x;
            let obstacle_right = platform.aabb.max.x;
            if dino_right < obstacle_left || dino_left > obstacle_right {
                continue;
            }

            // Now check if the dino is landing on the platform
            if dino.velocity.y <= 0.0 {
                if dino.aabb.min.y >= platform.aabb.max.y + 15.0 {
                    continue;
                } else if dino.aabb.min.y < platform.aabb.max.y {
                    continue;
                } else {
                    // Snap the dino to the top of the platform
                    let dino_height = dino.aabb.max.y - dino.aabb.min.y;
                    let dino_half_height = dino_height / 2.0;
                    let dino_new_y = platform.aabb.max.y + dino_half_height;
                    transform.translation.y = dino_new_y;
                    dino.aabb.min.y = platform.aabb.max.y;
                    dino.aabb.max.y = platform.aabb.max.y + dino_height;
                    dino.velocity.y = 0.0;
                    dino.grounded = true;
                    landed = true;
                }
            }
        }
        if !landed {
            dino.grounded = false;
        }
    }
    // }
}

// In arrow_move, add a query for the tree's Aabb:
pub fn arrow_move(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut dino: Query<(&mut Transform, &mut Sprite, &mut Dino), With<Sprite>>,
    mut obstacles: Query<&Obstacle>,
) {
    let mut rng = rand::rng();
    if let Ok((mut transform, mut sprite, mut dino)) = dino.single_mut() {
        dino.timer.tick(time.delta());

        // Start jump
        if keyboard_input.just_pressed(KeyCode::Space) && dino.grounded {
            dino.jumping = true;
            dino.jump_time = 0.0;
            dino.grounded = false;
            dino.attacking = false;
            dino.can_attack = false;
            dino.jump_height = 1500.0;
        }

        let gravity = -1200.0_f32;
        let max_jump_time = 0.22; // seconds, tune for feel
        let jump_acceleration = (2.0 * dino.jump_height * gravity.abs()).sqrt() * max_jump_time;

        // Continue jump while holding space and not exceeding max jump time
        if dino.jumping && keyboard_input.pressed(KeyCode::Space) && dino.jump_time < max_jump_time
        {
            dino.velocity.y = jump_acceleration;
            dino.jump_time += time.delta_secs();
        } else {
            dino.jumping = false;
        }

        // Horizontal movement input
        let mut target_velocity_x = 0.0;
        if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            sprite.flip_x = false;
            target_velocity_x = RUNNING_SPEED;
        } else if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            sprite.flip_x = true;
            target_velocity_x = -RUNNING_SPEED;
        }

        // Dampening factor (0.0 = instant, 1.0 = no change)
        let dampening = 0.95;
        dino.velocity.x = dino.velocity.x * dampening + target_velocity_x * (1.0 - dampening);

        // Save previous position
        let prev_x = transform.translation.x;

        // Apply velocity to position
        let dx = dino.velocity.x * time.delta_secs();
        transform.translation.x += dx;
        dino.aabb.min.x += dx;
        dino.aabb.max.x += dx;

        let x_collision = obstacles
            .iter()
            .any(|obstacle| dino.aabb.intersects(&obstacle.aabb));

        // if x_collision {
        //     // Simple collision response: reset to previous position
        //     transform.translation.x -= dx;
        //     dino.aabb.min.x -= dx;
        //     dino.aabb.max.x -= dx;
        //     dino.velocity.x = 0.0;
        // }

        if dino.jumping || !dino.grounded {
            if keyboard_input.just_pressed(KeyCode::Space) && !dino.attacking && dino.can_attack {
                // Run animation 18-24 for attack

                // Simulate an impact for todo code
                dino.attacking = true;

                if x_collision {
                    dino.jump_time = 0.0;
                    dino.jumping = true;
                }

                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = 18;
                };
            } else if dino.attacking {
                if dino.timer.just_finished() {
                    if let Some(atlas) = sprite.texture_atlas.as_mut() {
                        if atlas.index == 24 {
                            atlas.index = 23;
                            dino.attacking = false; // End attack animation
                        } else {
                            // Change this to a loop instead of selecting index 0

                            for hold in dino.frame_hold_counter.iter_mut() {
                                if hold.0 == atlas.index {
                                    // Hold this frame for N extra ticks
                                    if hold.1 < hold.2 {
                                        hold.1 += 1;
                                    } else {
                                        hold.1 = 0;
                                        atlas.index += 1;
                                    }
                                } else {
                                    atlas.index += 1;
                                }
                            }
                        }
                    };
                }
            } else if dino.velocity.y > -200.0 {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = 25; // Jumping frame
                };
            } else {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = 23; // Falling frame
                };
            }
            dino.can_attack = true;
        } else if keyboard_input.any_pressed([
            KeyCode::ArrowRight,
            KeyCode::KeyD,
            KeyCode::ArrowLeft,
            KeyCode::KeyA,
        ]) {
            // Walking state
            if dino.timer.just_finished() {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    let index = atlas.index.clamp(0, 11);
                    if index == 11 {
                        atlas.index = 0;
                    } else {
                        atlas.index = index + 1;
                    }
                };
            }
        } else {
            // Idle state
            if dino.timer.just_finished() {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    // First check if atlas.index is between 12-17
                    if atlas.index < 12 || atlas.index > 17 {
                        atlas.index = 12;
                    }

                    let index = atlas.index.clamp(12, 17);

                    // Handle idle animation frames, with a random chance to loop back to start
                    // to make it less repetitive.
                    if index == 17 {
                        // Gen a 1 in 10 change to reset to 12
                        dino.idle_frame_forward = false;
                        let roll: u8 = rng.random_range(0..40);
                        if roll == 0 {
                            atlas.index = 16;
                        }
                    } else if index == 16 {
                        if dino.idle_frame_forward {
                            atlas.index = 17;
                        } else {
                            dino.idle_frame_forward = true;
                            atlas.index = 12;
                        }
                    } else {
                        let roll: u8 = rng.random_range(0..10);
                        if roll == 0 {
                            atlas.index = index + 1;
                        }
                    }
                };
            }
        }
    }
}
