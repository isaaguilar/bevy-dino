use std::collections::HashMap;

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
    app.init_state::<GameState>()
        .add_event::<SceneChange>()
        .insert_resource(GeneratedPlatformObstacles::default())
        .insert_resource(GeneratedNonPlatformObstacles::default())
        .insert_resource(AppleBasket::default())
        .insert_resource(Health::default())
        .insert_resource(GameTimer::default())
        .insert_resource(TargetHeight::default())
        .insert_resource(GameStatus::default())
        .add_systems(OnEnter(AppState::Game), (setup, camera::game_camera))
        .add_systems(
            OnEnter(AppState::GameOver),
            (game_over_scoreboard, camera::game_camera),
        )
        .add_systems(
            Update,
            (
                update_timeboard,
                apple_collect,
                update_scoreboard,
                update_healthboard,
                update_heightboard,
                spawn_platforms,
                dino_gravity,
                arrow_move,
                camera::camera_tracking_system,
                camera::parallax_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Running))),
        )
        .add_systems(Update, game_over.run_if(on_event::<SceneChange>))
        .add_systems(Update, scene_transition)
        .add_systems(Update, restart.run_if(in_state(AppState::GameOver)));
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Running,
    NotRunning,
    Paused,
}

pub fn setup(mut commands: Commands, assets: Res<CustomAssets>, hud: Res<Hud>) {
    info!("Setting up the game");

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((Heightboard, Text("Press Left / Right To Move\n\n".into())));
                p.spawn((Timeboard, Text("Press Left / Right To Move\n\n".into())));
                p.spawn((
                    Healthboard,
                    Text("Press Space to jump. Press again to whip.\n\n".into()),
                ));
                p.spawn((
                    Scoreboard,
                    Text("Resizing window maintains aspect ratio".into()),
                ));
            });
    });

    commands.spawn((
        StateScoped(AppState::Game),
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
                StateScoped(AppState::Game),
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

    // Add a platform on top of the tree
    commands.spawn((
        StateScoped(AppState::Game),
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
}

fn spawn_platforms(
    mut commands: Commands,
    assets: Res<CustomAssets>,
    player_query: Query<&Transform, With<Player>>,
    mut platform_obstacle_tiles: ResMut<GeneratedPlatformObstacles>,
    mut non_platform_obstacle_tiles: ResMut<GeneratedNonPlatformObstacles>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };
    let mut rng = rand::rng();

    let current_x_tile = (transform.translation.x / RESOLUTION_WIDTH).floor() as i32;
    let current_y_tile = (transform.translation.y / RESOLUTION_HEIGHT).floor() as i32;

    for i in current_x_tile - 2..=current_x_tile + 2 {
        for j in current_y_tile - 2..=current_y_tile + 2 {
            // We're within a 600x480 box where we have to spawn obstacles (trees) and
            // obstacles + platforms (landings) based on a set of rules and randomness.

            // The obstacle-platform (landins) placement rules are:
            // 1. They should be at least 100 pixels apart from each other in distance.
            // 2. There should be at least:
            //      a. one platform max 50 pixels above trees.
            //      b. one platform max 400 pixels below trees at a max of 300 pixels away from either side of the tree.

            // The obstacle (tree) placement rules are:
            // 1. There should be a tree within a landing that's:
            //    a. max 50 pixels above and 150 pixels away from either side of the landing
            //    b. max 400 pixels below and 300 pixels away from either side of the landing.
            // 2. Trees should be at least 200 pixels apart from each other.

            if platform_obstacle_tiles.0.get(&(i, j)).is_some() {
                continue;
            }

            if non_platform_obstacle_tiles.0.get(&(i, j)).is_some() {
                continue;
            }

            let mut platform_obstacles = vec![];
            let mut non_platform_obstacles = vec![];

            // Let's loop within this tile to place landings and trees
            // There should be a minimum of 4 elements per tile
            loop {
                let total_obstacles = platform_obstacles.len() + non_platform_obstacles.len();
                if total_obstacles >= 4 {
                    let roll = rng.random_range(0..2);
                    if roll == 0 {
                        break;
                    }
                }

                if total_obstacles >= 8 {
                    // Max 8 elements per tile
                    break;
                }

                if total_obstacles == 0 {
                    let roll = rng.random_range(0..2);
                    if roll == 0 {
                        // Start by placing a platform at a random position within the tile
                        let platform_x = (i as f32 * RESOLUTION_WIDTH)
                            + rng.random_range(100.0..(RESOLUTION_WIDTH - 100.0));
                        let platform_y = (j as f32 * RESOLUTION_HEIGHT)
                            + rng.random_range(
                                -RESOLUTION_HEIGHT / 2.0 + 20.0..RESOLUTION_HEIGHT / 2.0 - 20.0,
                            );
                        let obstacle = Obstacle {
                            aabb: Aabb2d::new(
                                Vec2::new(platform_x, platform_y),
                                Vec2::new(50., 10.0),
                            ),
                        };

                        commands.spawn((
                            StateScoped(AppState::Game),
                            Platform,
                            Sprite {
                                image: assets.leaves.clone(),
                                // color: bevy::color::palettes::css::GREEN.into(),
                                // custom_size: Some(Vec2::new(100., 20.)),
                                ..default()
                            },
                            Transform::from_xyz(platform_x, platform_y, -1.),
                            obstacle.clone(),
                        ));
                        platform_obstacles.push(obstacle)
                    } else {
                        // Place a tree at a random position within the tile
                        let platform_x = (i as f32 * RESOLUTION_WIDTH)
                            + rng.random_range(50.0..(RESOLUTION_WIDTH - 50.0));
                        let platform_y = (j as f32 * RESOLUTION_HEIGHT)
                            + rng.random_range(
                                -RESOLUTION_HEIGHT / 2.0 + 190.0..RESOLUTION_HEIGHT / 2.0 - 190.0,
                            );
                        let obstacle = Obstacle {
                            aabb: Aabb2d::new(
                                Vec2::new(platform_x, platform_y),
                                Vec2::new(25., 190.0),
                            ),
                        };

                        let roll = rng.random_range(0..2);
                        let tree_sprite = if roll == 0 {
                            // Randomly add an apple tree
                            // ()
                            assets.tree.clone()
                        } else {
                            // ()
                            assets.tree.clone()
                        };

                        let mut tree = commands.spawn((
                            StateScoped(AppState::Game),
                            obstacle.clone(),
                            Sprite {
                                image: tree_sprite,
                                // color: bevy::color::palettes::css::BROWN.into(),
                                custom_size: Some(Vec2::new(50., 380.)),
                                ..default()
                            },
                            Transform::from_xyz(platform_x, platform_y, -5.),
                        ));
                        non_platform_obstacles.push(obstacle);

                        if roll == 0 {
                            // Randomly add an apple tree
                            tree.with_child((
                                Apple {
                                    aabb: Aabb2d::new(
                                        Vec2::new(platform_x, platform_y + 380.0 / 2. + 8.0),
                                        Vec2::new(8.0, 8.0),
                                    ),
                                },
                                Transform::from_xyz(0., 380.0 / 2.0 + 8.0, -5.),
                                Sprite {
                                    color: bevy::color::palettes::css::RED.into(),
                                    custom_size: Some(Vec2::new(16., 16.)),
                                    ..default()
                                },
                            ));
                        }
                    }
                }

                if platform_obstacles.is_empty() && non_platform_obstacles.is_empty() {
                    continue;
                }

                let roll = rng.random_range(0..2);
                if roll == 0 {
                    // Add more elements relative to existing ones
                    // First try adding a platform relative to existing platforms
                    for existing_platform in &platform_obstacles {
                        let roll = rng.random_range(0..50);
                        if roll == 0 {
                            // Skip adding more platforms sometimes
                            continue;
                        }
                        let platform_x_offset = rng.random_range(-150.0..150.0);
                        let platform_y_offset = rng.random_range(-400.0..400.0);
                        let platform_x = existing_platform.aabb.min.x + platform_x_offset;
                        let platform_y = existing_platform.aabb.min.y + platform_y_offset;
                        let obstacle = Obstacle {
                            aabb: Aabb2d::new(
                                Vec2::new(platform_x, platform_y),
                                Vec2::new(50., 10.0),
                            ),
                        };
                        commands.spawn((
                            StateScoped(AppState::Game),
                            Platform,
                            Sprite {
                                image: assets.leaves.clone(),
                                // color: bevy::color::palettes::css::GREEN.into(),
                                // custom_size: Some(Vec2::new(100., 20.)),
                                ..default()
                            },
                            Transform::from_xyz(platform_x, platform_y, -1.),
                            obstacle.clone(),
                        ));
                        platform_obstacles.push(obstacle);
                        break;
                    }

                    // Next try adding a platform relative to existing non-platform obstacles (trees)

                    continue;
                } else {
                    // Add tree relative to existing obstacles
                    for existing_obstacle in &non_platform_obstacles {
                        let roll = rng.random_range(0..50);
                        if roll == 0 {
                            // Skip adding more platforms sometimes
                            continue;
                        }
                        let obstacle_x_offset = rng.random_range(-300.0..300.0);
                        let obstacle_y_offset = rng.random_range(-400.0..150.0);
                        let obstacle_x = existing_obstacle.aabb.min.x + obstacle_x_offset;
                        let obstacle_y = existing_obstacle.aabb.min.y + obstacle_y_offset;
                        let obstacle = Obstacle {
                            aabb: Aabb2d::new(
                                Vec2::new(obstacle_x, obstacle_y),
                                Vec2::new(25., 190.0),
                            ),
                        };
                        commands.spawn((
                            StateScoped(AppState::Game),
                            obstacle.clone(),
                            Sprite {
                                image: assets.tree.clone(),
                                // color: bevy::color::palettes::css::BROWN.into(),
                                custom_size: Some(Vec2::new(50., 380.)),
                                ..default()
                            },
                            Transform::from_xyz(obstacle_x, obstacle_y, -5.),
                        ));
                        non_platform_obstacles.push(obstacle);
                        break;
                    }
                }
            }

            platform_obstacle_tiles.0.insert((i, j), platform_obstacles);

            non_platform_obstacle_tiles
                .0
                .insert((i, j), non_platform_obstacles);
        }
    }
}

#[derive(Resource)]
pub struct GameTimer(pub Timer);

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(200., TimerMode::Once))
    }
}

#[derive(Resource)]
pub struct TargetHeight(pub f32);

impl Default for TargetHeight {
    fn default() -> Self {
        Self(9822.) // Makes 10k when the dino lands on the starting platform
    }
}

#[derive(Component)]
pub struct Scoreboard;

#[derive(Component)]
pub struct Healthboard;

#[derive(Component)]
pub struct Timeboard;

#[derive(Component)]
pub struct Heightboard;

#[derive(Resource)]
pub struct Health(pub u32);

impl Default for Health {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Resource, Default)]
pub struct GeneratedPlatformObstacles(pub HashMap<(i32, i32), Vec<Obstacle>>);

#[derive(Resource, Default)]
pub struct GeneratedNonPlatformObstacles(pub HashMap<(i32, i32), Vec<Obstacle>>);

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone)]
pub struct Platform;

#[derive(Component, Debug, Clone)]
pub struct Obstacle {
    pub aabb: Aabb2d,
}

#[derive(Component)]
pub struct Apple {
    pub aabb: Aabb2d,
}

#[derive(Resource, Default)]
pub struct AppleBasket(u32);

#[derive(Component, Debug, Clone)]
pub struct Dino {
    pub timer: Timer,
    pub idle_frame_forward: bool,
    pub grounded: bool,
    pub velocity: Vec2,
    pub jumping: bool,
    pub jump_time: f32,
    pub jump_height: f32,
    pub attacking: bool,
    pub can_attack: bool,
    pub frame_hold_counter: Vec<(usize, u8, u8)>,
    pub aabb: Aabb2d,
    pub health: i32,
}

impl Default for Dino {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.07, TimerMode::Repeating),
            idle_frame_forward: true,
            grounded: false,
            velocity: Vec2::ZERO,
            jumping: false,
            attacking: false,
            jump_height: 1500.0,
            frame_hold_counter: vec![(21, 0, 1)],
            can_attack: false,
            jump_time: 0.0,
            health: 100,
            aabb: Aabb2d::new(Vec2::ZERO, Vec2::new(32., 32.)),
        }
    }
}

#[derive(Component)]
pub struct Transition {
    pub timer: Timer,
    pub last_frame: usize,
    pub pause_frame: usize,
}

impl Transition {
    pub fn new(last_frame: usize, pause_frame: usize) -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            last_frame: last_frame,
            pause_frame: pause_frame,
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

        // if transform.translation.y < -5000. {
        //     dino.velocity = Vec2::ZERO;
        //     transform.translation = Vec3::ZERO;
        //     dino.aabb = Aabb2d::new(Vec2::ZERO, Vec2::new(32., 32.));
        // }

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

                    if dino.velocity.y < -1500.0 {
                        let damage = 100 / 5 * ((dino.velocity.y / 500.).abs().floor() as i32 - 2);
                        dino.health -= damage;
                    }

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
    obstacles: Query<&Obstacle>,
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

// Collect apples and add to applebasket
fn apple_collect(
    mut commands: Commands,
    mut apple_basket: ResMut<AppleBasket>,
    apples: Query<(Entity, &Apple)>,
    dino_query: Query<&Dino>,
) {
    let Ok(dino) = dino_query.single() else {
        return;
    };
    for (entity, apple) in apples {
        if apple.aabb.intersects(&dino.aabb) {
            apple_basket.0 += 1;
            // Do an animation
            commands.entity(entity).despawn();
        }
    }
}

fn update_scoreboard(
    mut scoreboard: Query<&mut Text, With<Scoreboard>>,
    apple_basket: Res<AppleBasket>,
) {
    let Ok(mut scoreboard_text) = scoreboard.single_mut() else {
        return;
    };

    scoreboard_text.0 = apple_basket.0.to_string() + " apples";
}

fn update_healthboard(
    mut commands: Commands,
    mut scoreboard: Query<&mut Text, With<Healthboard>>,
    dino_query: Query<&Dino>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut scoreboard_text) = scoreboard.single_mut() else {
        return;
    };

    let Ok(dino) = dino_query.single() else {
        return;
    };

    scoreboard_text.0 = dino.health.to_string() + " health left";

    if dino.health <= 0 {
        *game_status = GameStatus::Lose;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange);
    }
}

fn update_timeboard(
    mut commands: Commands,
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timeboard: Query<&mut Text, With<Timeboard>>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_timer.0.tick(time.delta());
    let Ok(mut timeboard_text) = timeboard.single_mut() else {
        return;
    };
    timeboard_text.0 = game_timer.0.remaining_secs().ceil().to_string();

    if game_timer.0.finished() {
        *game_status = GameStatus::Lose;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange);
    }
}

fn update_heightboard(
    mut commands: Commands,
    time: Res<Time>,
    mut target_height: ResMut<TargetHeight>,
    mut dino: Query<&mut Transform, With<Dino>>,
    mut height_board: Query<&mut Text, With<Heightboard>>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut heightboard_text) = height_board.single_mut() else {
        return;
    };

    let Ok(mut transform) = dino.single_mut() else {
        return;
    };

    heightboard_text.0 = (target_height.0 - transform.translation.y)
        .ceil()
        .to_string()
        + " left to go";

    if target_height.0 - transform.translation.y <= 0.0 {
        *game_status = GameStatus::Win;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange);
    }
}

fn game_over(mut commands: Commands, assets: Res<CustomAssets>) {
    commands.spawn((
        // BackgroundColor(BLACK.into()),
        Transition::new(13, 6),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ZIndex(99),
        ImageNode {
            image: assets.circle_transition.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.circle_transition_layout.clone(),
                index: 0,
            }),

            ..default()
        },
    ));
}

fn scene_transition(
    mut commands: Commands,
    time: Res<Time>,
    mut loading_state: ResMut<NextState<AppState>>,
    mut transition_ui: Query<(Entity, &mut ImageNode, &mut Transition)>,
) {
    for (entity, mut sprite, mut transition) in transition_ui.iter_mut() {
        transition.timer.tick(time.delta());

        if transition.timer.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index == transition.pause_frame - 1 {
                    atlas.index += 1;
                    loading_state.set(AppState::GameOver);
                } else if atlas.index < transition.last_frame {
                    atlas.index += 1;
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn restart(
    mut loading_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut generated_platforms: ResMut<GeneratedPlatformObstacles>,
    mut generated_non_platforms: ResMut<GeneratedNonPlatformObstacles>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_timer: ResMut<GameTimer>,
    mut apple_basket: ResMut<AppleBasket>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        loading_state.set(AppState::Game);
        game_state.set(GameState::Running);
        generated_platforms.0.clear();
        generated_non_platforms.0.clear();
        game_timer.0.reset();
        apple_basket.0 = 0;
        return;
    }
}

fn game_over_scoreboard(
    mut commands: Commands,
    hud: Res<Hud>,

    apple_basket: Res<AppleBasket>,
    mut game_timer: ResMut<GameTimer>,
) {
    let apples = apple_basket.0;
    let time_left = game_timer.0.remaining_secs().ceil();

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::GameOver),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((Text("Total Apples: ".to_string() + &apples.to_string())));
                p.spawn((Text(
                    "Time Remaining: ".to_string() + &time_left.to_string(),
                ),));
                p.spawn((Text("Press R to restart ".to_string())));
            });
    });
}

#[derive(Resource, Default)]
pub enum GameStatus {
    #[default]
    InProgress,
    Lose,
    Win,
}

#[derive(Event)]
pub struct SceneChange;
