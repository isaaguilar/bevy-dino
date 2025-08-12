use crate::app::{AppState, DisplayLanguage, RESOLUTION_HEIGHT, RESOLUTION_WIDTH, RUNNING_SPEED};
use crate::assets::custom::CustomAssets;
use crate::assets::lexi::game_over::GameOverLex;
use crate::camera;
use crate::util::handles::BODY_FONT;
use bevy::ecs::system::Commands;
use bevy::input::ButtonInput;
use bevy::input::common_conditions::input_just_pressed;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::platform::collections::HashMap;

use bevy::sprite::Sprite;
use bevy::ui::{AlignItems, Display, FlexDirection, Node, PositionType, Val};
use bevy::{audio, prelude::*};
use bevy_aspect_ratio_mask::Hud;
use bevy_http_client::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputPlugin, TextInputTextColor, TextInputTextFont, TextInputValue,
};
use rand::Rng;
use serde::Deserialize;

const LEADERBOARD_URL: &'static str = env!("LEADERBOARD_URL");

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_event::<SceneChange>()
        .add_event::<RenderHighScores>()
        .add_event::<PostHighScore>()
        .add_plugins((TextInputPlugin, HttpClientPlugin))
        .insert_resource(GeneratedPlatformObstacles::default())
        .insert_resource(GeneratedNonPlatformObstacles::default())
        .insert_resource(AppleBasket::default())
        .insert_resource(TotalPoints::default())
        .insert_resource(GameTimer::default())
        .insert_resource(TargetHeight::default())
        .insert_resource(GameStatus::default())
        .insert_resource(HighScores::default())
        .insert_resource(PendingSceneChange::default())
        .insert_resource(SfxMusicVolume::default())
        .add_systems(Startup, global_volume_set)
        .add_systems(OnEnter(AppState::Game), (sfx_setup, setup))
        .add_systems(OnEnter(AppState::GameOver), (game_over_scoreboard,))
        .add_systems(Startup, camera::game_camera)
        .add_systems(
            Update,
            (
                update_timeboard,
                apple_collect,
                clock_collect,
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
        .add_systems(Update, post_high_score.run_if(on_event::<PostHighScore>))
        .add_systems(Update, game_over.run_if(on_event::<SceneChange>))
        .add_systems(Update, scene_transition)
        .add_systems(FixedUpdate, (fade_out_and_despawn, fade_in_music))
        .add_systems(Update, (handle_response, handle_error, button_system))
        .add_systems(
            Update,
            (update_high_scoreboard).run_if(in_state(AppState::HighScores)),
        )
        .add_systems(OnEnter(AppState::GameOver), waiting_music)
        .add_systems(OnEnter(AppState::Menu), (waiting_music, volume_toggle_hud))
        .add_systems(OnEnter(AppState::HighScores), waiting_music)
        .add_systems(OnEnter(AppState::Credits), setup_credits)
        .add_systems(
            Update,
            press_space_to_start.run_if(
                in_state(GameState::NotRunning)
                    .and(in_state(AppState::Game))
                    .and(input_just_pressed(KeyCode::Space)),
            ),
        )
        .add_systems(Update, music_toggle)
        .add_systems(OnEnter(AppState::HighScores), setup_high_score_board);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Running,
    #[default]
    NotRunning,
}

#[derive(Component)]
pub struct GameMusic;

#[derive(Component)]
pub struct SpaceToStart;

pub fn press_space_to_start(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<SpaceToStart>>,
) {
    for entity in query {
        commands.entity(entity).despawn()
    }
    game_state.set(GameState::Running);
}

pub fn global_volume_set(mut volume: ResMut<GlobalVolume>) {
    info!("Set Vol");
    volume.volume = bevy::audio::Volume::Linear(0.50); // Sets global volume to 50%
}

#[derive(Resource)]
pub struct SfxMusicVolume {
    pub music: bool,
    pub sfx: bool,
}

impl Default for SfxMusicVolume {
    fn default() -> Self {
        Self {
            music: true,
            sfx: true,
        }
    }
}

pub fn toggle_music_on_click(
    _: Trigger<Pointer<Click>>,
    mut sfx_music_volume: ResMut<SfxMusicVolume>,
    mut icon: Query<&mut ImageNode, With<VolumeToggleMusicMarker>>,
) {
    sfx_music_volume.music = !sfx_music_volume.music;

    if let Ok(mut sprite) = icon.single_mut() {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if sfx_music_volume.music {
                atlas.index = 0;
            } else {
                atlas.index = 1;
            }
        }
    }
}

pub fn toggle_sfx_on_click(
    _: Trigger<Pointer<Click>>,
    mut sfx_music_volume: ResMut<SfxMusicVolume>,
    mut icon: Query<&mut ImageNode, With<VolumeToggleSfxMarker>>,
) {
    sfx_music_volume.sfx = !sfx_music_volume.sfx;

    if let Ok(mut sprite) = icon.single_mut() {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if sfx_music_volume.sfx {
                atlas.index = 0;
            } else {
                atlas.index = 1;
            }
        }
    }
}

#[derive(Component)]
pub struct VolumeToggleMarker;

#[derive(Component)]
pub struct VolumeToggleMusicMarker;

#[derive(Component)]
pub struct VolumeToggleSfxMarker;

pub fn volume_toggle_hud(
    mut commands: Commands,
    hud: Res<Hud>,
    assets: Res<CustomAssets>,
    query: Query<(), With<VolumeToggleMarker>>,
) {
    if query.iter().next().is_some() {
        return;
    }
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                VolumeToggleMarker,
                VolumeToggleMusicMarker,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    left: Val::Px(15.0),
                    top: Val::Px(410.0),
                    width: Val::Px(18.),
                    height: Val::Px(18.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: assets.volume.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.volume_layout.clone(),
                        index: 0,
                    }),
                    ..default()
                },
            ))
            .observe(toggle_music_on_click);

        parent
            .spawn((
                VolumeToggleMarker,
                VolumeToggleSfxMarker,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    left: Val::Px(15.0),
                    top: Val::Px(440.0),
                    width: Val::Px(18.),
                    height: Val::Px(18.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: assets.sfx.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.sfx_layout.clone(),
                        index: 0,
                    }),
                    ..default()
                },
            ))
            .observe(toggle_sfx_on_click);
    });
}

#[derive(Component)]
pub struct MusicVolume(pub f32);

pub fn music_toggle(
    sfx_music_volume: Res<SfxMusicVolume>,
    music: Query<(&mut AudioSink, &MusicVolume)>,
) {
    for (mut audio, music_volume) in music {
        if !sfx_music_volume.music {
            audio.set_volume(audio::Volume::Linear(0.0));
        } else {
            audio.set_volume(audio::Volume::Linear(music_volume.0));
        }
    }
}

pub fn sfx_setup(
    mut commands: Commands,
    assets: Res<CustomAssets>,
    music: Query<&mut AudioSink, With<GameMusic>>,
    waiting_music_query: Query<Entity, With<WaitingMusic>>,
) {
    if music.single().is_err() {
        commands.spawn((
            GameMusic,
            MusicVolume(1.2),
            FadeInMusic::new(1.2),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.0)),
            AudioPlayer(assets.music.clone()),
        ));
    }

    if let Ok(entity) = waiting_music_query.single() {
        commands.entity(entity).despawn();
    }
}

pub fn setup(
    mut commands: Commands,
    assets: Res<CustomAssets>,
    hud: Res<Hud>,
    mut game_state: ResMut<NextState<GameState>>,
    mut generated_platforms: ResMut<GeneratedPlatformObstacles>,
    mut generated_non_platforms: ResMut<GeneratedNonPlatformObstacles>,
    mut game_timer: ResMut<GameTimer>,
    mut total_points: ResMut<TotalPoints>,
    mut apple_basket: ResMut<AppleBasket>,
) {
    game_state.set(GameState::NotRunning);
    generated_platforms.0.clear();
    generated_non_platforms.0.clear();
    game_timer.0.reset();
    total_points.0 = 0;
    apple_basket.0 = 0;

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
                p.spawn((
                    SpaceToStart,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 15.),
                    Text("Press Spacebar to Start".into()),
                ));
            });

        parent.spawn((
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                left: Val::Px(8.0),
                top: Val::Px(15.0),
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                ..default()
            },
            ImageNode {
                image: assets.flag.clone(),
                ..default()
            },
        ));

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    left: Val::Px(28.0),
                    top: Val::Px(15.0),

                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    Heightboard,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent.spawn((
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                left: Val::Px(250.0),
                top: Val::Px(15.0),
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                ..default()
            },
            ImageNode {
                image: assets.clock.clone(),
                ..default()
            },
        ));

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(15.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    Timeboard,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent.spawn((
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                left: Val::Px(520.0),
                top: Val::Px(15.0),
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                ..default()
            },
            ImageNode {
                image: assets.appleicon.clone(),
                ..default()
            },
        ));

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    left: Val::Px(550.0),
                    top: Val::Px(15.0),

                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    Scoreboard,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent.spawn((
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                left: Val::Px(250.0),
                top: Val::Px(450.0),
                // width: Val::Px(18.),
                // height: Val::Px(18.),
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    HealthBar(0),
                    ImageNode {
                        image: assets.dinoicon.clone(),

                        ..default()
                    }
                ),
                (
                    HealthBar(1),
                    ImageNode {
                        image: assets.dinoicon.clone(),

                        ..default()
                    }
                ),
                (
                    HealthBar(2),
                    ImageNode {
                        image: assets.dinoicon.clone(),

                        ..default()
                    }
                ),
                (
                    HealthBar(3),
                    ImageNode {
                        image: assets.dinoicon.clone(),

                        ..default()
                    }
                ),
                (
                    HealthBar(4),
                    ImageNode {
                        image: assets.dinoicon.clone(),

                        ..default()
                    }
                ),
            ],
        ));
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

#[derive(Component)]
pub struct HealthBar(pub u32);

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

                        let roll = rng.random_range(0..16);

                        let mut platform = commands.spawn((
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

                        if roll == 0 {
                            platform.with_child((
                                TimeExtender {
                                    aabb: Aabb2d::new(
                                        Vec2::new(platform_x, platform_y + 20.0 / 2. + 8.0),
                                        Vec2::new(8.0, 8.0),
                                    ),
                                },
                                Transform::from_xyz(0., 20.0 / 2.0 + 8.0, -5.),
                                Sprite {
                                    image: assets.clock.clone(),

                                    ..default()
                                },
                            ));
                        }
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
                                    image: assets.apple.clone(),
                                    // color: bevy::color::palettes::css::RED.into(),
                                    custom_size: Some(Vec2::new(30., 30.)),
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

#[derive(Component)]
pub struct TimeExtender {
    pub aabb: Aabb2d,
}

#[derive(Resource, Default)]
pub struct AppleBasket(u32);

#[derive(Resource, Default)]
pub struct TotalPoints(u32);

#[derive(Component, Debug, Clone)]
pub struct Dino {
    pub timer: Timer,
    pub walk_sound_effect_timer: Timer,
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
            walk_sound_effect_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
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
    mut commands: Commands,
    assets: Res<CustomAssets>,
    sfx_music_volume: Res<SfxMusicVolume>,
) {
    if let Ok((mut transform, mut dino)) = dino.single_mut() {
        let gravity = -1200.0;

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
                        let mut rng = rand::rng();
                        let roll = rng.random_range(1..3);
                        let sfx = if roll == 1 {
                            assets.thud1.clone()
                        } else if roll == 2 {
                            assets.thud2.clone()
                        } else {
                            assets.thud3.clone()
                        };

                        let vol = if sfx_music_volume.sfx { 2.0 } else { 0.0 };

                        commands.spawn((
                            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                            AudioPlayer(sfx),
                        ));

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
    mut commands: Commands,
    assets: Res<CustomAssets>,
    sfx_music_volume: Res<SfxMusicVolume>,
) {
    let mut rng = rand::rng();
    if let Ok((mut transform, mut sprite, mut dino)) = dino.single_mut() {
        dino.timer.tick(time.delta());
        dino.walk_sound_effect_timer.tick(time.delta());

        // Start jump
        if keyboard_input.just_pressed(KeyCode::Space) && dino.grounded {
            let roll = rng.random_range(1..2);
            let sfx = if roll == 1 {
                assets.boingjump1.clone()
            } else {
                assets.boingjump2.clone()
            };

            let vol = if sfx_music_volume.sfx { 0.5 } else { 0.0 };
            commands.spawn((
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                AudioPlayer(sfx),
            ));

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

                let roll = rng.random_range(1..4);
                let sfx = if roll == 1 {
                    assets.swoosh1.clone()
                } else if roll == 2 {
                    assets.swoosh2.clone()
                } else if roll == 3 {
                    assets.swoosh3.clone()
                } else {
                    assets.swoosh4.clone()
                };

                let vol = if sfx_music_volume.sfx { 0.5 } else { 0.0 };

                commands.spawn((
                    PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                    AudioPlayer(sfx),
                ));
                // Simulate an impact for todo code
                dino.attacking = true;

                if x_collision {
                    let roll = rng.random_range(1..3);
                    let sfx = if roll == 1 {
                        assets.impact1.clone()
                    } else if roll == 2 {
                        assets.impact2.clone()
                    } else {
                        assets.impact3.clone()
                    };
                    let vol = if sfx_music_volume.sfx { 0.25 } else { 0.0 };

                    commands.spawn((
                        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                        AudioPlayer(sfx),
                    ));
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
            if dino.walk_sound_effect_timer.just_finished() {
                let roll = rng.random_range(1..10);
                let sfx = if roll == 1 {
                    assets.walk1.clone()
                } else if roll == 2 {
                    assets.walk2.clone()
                } else if roll == 3 {
                    assets.walk3.clone()
                } else if roll == 4 {
                    assets.walk4.clone()
                } else if roll == 5 {
                    assets.walk5.clone()
                } else if roll == 6 {
                    assets.walk6.clone()
                } else if roll == 7 {
                    assets.walk7.clone()
                } else if roll == 8 {
                    assets.walk8.clone()
                } else if roll == 9 {
                    assets.walk9.clone()
                } else {
                    assets.walk10.clone()
                };

                let vol = if sfx_music_volume.sfx { 1.0 } else { 0.0 };

                commands.spawn((
                    PlaybackSettings::DESPAWN.with_volume(audio::Volume::Linear(vol)),
                    AudioPlayer(sfx),
                ));
            }
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
    assets: Res<CustomAssets>,
    sfx_music_volume: Res<SfxMusicVolume>,
) {
    let Ok(dino) = dino_query.single() else {
        return;
    };
    for (entity, apple) in apples {
        if apple.aabb.intersects(&dino.aabb) {
            let vol = if sfx_music_volume.sfx { 2.5 } else { 0.0 };

            commands.spawn((
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                AudioPlayer(assets.collect_sfx.clone()),
            ));
            apple_basket.0 += 1;
            // Do an animation
            commands.entity(entity).despawn();
        }
    }
}

// Extend the timer by collecting the clocks
fn clock_collect(
    mut commands: Commands,
    mut game_timer: ResMut<GameTimer>,
    clocks: Query<(Entity, &TimeExtender)>,
    dino_query: Query<&Dino>,
    assets: Res<CustomAssets>,
    sfx_music_volume: Res<SfxMusicVolume>,
) {
    let Ok(dino) = dino_query.single() else {
        return;
    };
    for (entity, clock) in clocks {
        if clock.aabb.intersects(&dino.aabb) {
            let vol = if sfx_music_volume.sfx { 2.5 } else { 0.0 };

            commands.spawn((
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
                AudioPlayer(assets.collect_sfx.clone()),
            ));
            let remaining = game_timer.0.remaining().as_secs_f32();
            game_timer.0 = Timer::from_seconds(remaining + 60., TimerMode::Once);
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

    scoreboard_text.0 = apple_basket.0.to_string();
}

fn update_healthboard(
    mut commands: Commands,
    health_icons: Query<(Entity, &HealthBar)>,
    dino_query: Query<&Dino>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(dino) = dino_query.single() else {
        return;
    };

    for (entity, dino_health_icon) in health_icons {
        if dino.health == 80 {
            if dino_health_icon.0 >= 4 {
                commands.entity(entity).despawn();
            }
        }
        if dino.health == 60 {
            if dino_health_icon.0 >= 3 {
                commands.entity(entity).despawn();
            }
        }
        if dino.health == 40 {
            if dino_health_icon.0 >= 2 {
                commands.entity(entity).despawn();
            }
        }
        if dino.health == 20 {
            if dino_health_icon.0 >= 1 {
                commands.entity(entity).despawn();
            }
        }
        if dino.health == 0 {
            if dino_health_icon.0 == 0 {
                commands.entity(entity).despawn();
            }
        }
    }

    if dino.health <= 0 {
        *game_status = GameStatus::Lose;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange(AppState::GameOver));
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
        commands.send_event(SceneChange(AppState::GameOver));
    }
}

fn update_heightboard(
    mut commands: Commands,

    target_height: Res<TargetHeight>,
    dino: Query<&Transform, With<Dino>>,
    mut height_board: Query<&mut Text, With<Heightboard>>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut heightboard_text) = height_board.single_mut() else {
        return;
    };

    let Ok(transform) = dino.single() else {
        return;
    };

    heightboard_text.0 = (target_height.0 - transform.translation.y)
        .ceil()
        .to_string();

    if target_height.0 - transform.translation.y <= 0.0 {
        *game_status = GameStatus::Win;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange(AppState::GameOver));
    }
}

fn game_over(
    mut reader: EventReader<SceneChange>,
    mut commands: Commands,
    mut pending_scene_change: ResMut<PendingSceneChange>,
    assets: Res<CustomAssets>,
) {
    for event in reader.read() {
        let data = event.0.clone();
        pending_scene_change.0 = Some(data);
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
}

#[derive(Component)]
pub struct FadeOutMusic;

#[derive(Component)]
pub struct FadeInMusic(pub bevy::audio::Volume);

impl FadeInMusic {
    pub fn new(vol: f32) -> Self {
        Self(audio::Volume::Linear(vol))
    }
}

fn fade_out_and_despawn(
    mut commands: Commands,
    music_query: Query<(Entity, &mut AudioSink), With<FadeOutMusic>>,
) {
    for (entity, mut audio_controls) in music_query {
        let current_volume = audio_controls.volume().to_linear();

        if current_volume < 0.01 {
            commands.entity(entity).despawn()
        } else {
            audio_controls.set_volume(bevy::audio::Volume::Linear(current_volume - 0.005));
        }
    }
}

fn fade_in_music(
    mut commands: Commands,
    music_query: Query<(Entity, &mut AudioSink, &FadeInMusic)>,
) {
    for (entity, mut audio_controls, fade_in_volume) in music_query {
        let current_volume = audio_controls.volume().to_linear();

        if current_volume >= fade_in_volume.0.to_linear() {
            commands.entity(entity).remove::<FadeInMusic>();
        } else {
            audio_controls.set_volume(bevy::audio::Volume::Linear(current_volume + 0.001));
        }
    }
}

fn scene_transition(
    mut commands: Commands,
    time: Res<Time>,
    pending_scene_change: Res<PendingSceneChange>,
    mut loading_state: ResMut<NextState<AppState>>,
    mut transition_ui: Query<(Entity, &mut ImageNode, &mut Transition)>,
    mut game_music: Query<Entity, (With<GameMusic>, Without<FadeOutMusic>)>,
    menu_music: Query<Entity, (With<WaitingMusic>, Without<FadeOutMusic>)>,
) {
    let Some(next_scene) = &pending_scene_change.0 else {
        return;
    };

    if *next_scene == AppState::GameOver {
        if let Ok(entity) = game_music.single_mut() {
            commands.entity(entity).insert(FadeOutMusic);
        }
    } else if *next_scene == AppState::Game {
        if let Ok(entity) = menu_music.single() {
            commands.entity(entity).insert(FadeOutMusic);
        }
    }

    for (entity, mut sprite, mut transition) in transition_ui.iter_mut() {
        transition.timer.tick(time.delta());

        if transition.timer.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index == transition.pause_frame - 1 {
                    atlas.index += 1;
                    loading_state.set(next_scene.clone());
                } else if atlas.index < transition.last_frame {
                    atlas.index += 1;
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct WaitingMusic;

fn waiting_music(
    mut commands: Commands,
    assets: Res<CustomAssets>,
    music: Query<(), With<WaitingMusic>>,
) {
    if music.single().is_err() {
        commands.spawn((
            WaitingMusic,
            MusicVolume(0.25),
            FadeInMusic::new(0.25),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.0)),
            AudioPlayer(assets.menu_music.clone()),
        ));
    }
}

fn game_over_scoreboard(
    mut commands: Commands,
    hud: Res<Hud>,
    game_status: Res<GameStatus>,
    apple_basket: Res<AppleBasket>,
    game_timer: Res<GameTimer>,
    language: Res<DisplayLanguage>,
    mut total_points: ResMut<TotalPoints>,
    game_over_options: Res<Assets<GameOverLex>>,
    assets: Res<CustomAssets>,
    sfx_music_volume: Res<SfxMusicVolume>,
) {
    let lex = if game_status.won() {
        let vol = if sfx_music_volume.sfx { 0.5 } else { 0.0 };

        commands.spawn((
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
            AudioPlayer(assets.win.clone()),
        ));
        get_lex_by_id(&game_over_options, "win")
    } else if game_status.lost() {
        let vol = if sfx_music_volume.sfx { 0.8 } else { 0.0 };
        commands.spawn((
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
            AudioPlayer(assets.lose.clone()),
        ));
        get_lex_by_id(&game_over_options, "lose")
    } else {
        return;
    };

    let display_text = lex.lex.from_language(&language.0);

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
                let apple_total = apples * 12;
                let time_total = time_left as u32 * 2;
                let cider_total = (apples / 10) * 500;
                let total = apple_total + time_total + cider_total;

                // Math
                // Apples = x12
                // Time = x2
                // Every 10th apple = Cider
                // Cider = 500 pts

                if game_status.won() {
                    p.spawn((
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text(format!("Total Apples: {} x 12 = {}", apples, apple_total)),
                    ));
                    p.spawn((
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text(format!(
                            "Total Cider: {} x 500 = {}",
                            apples / 10,
                            cider_total
                        )),
                    ));
                    p.spawn((
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text(format!(
                            "Time Remaining: {} x 2 = {}",
                            time_left, time_total
                        )),
                    ));
                    total_points.0 = total;
                    p.spawn(spacer());
                    p.spawn((
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text(display_text + " " + &total.to_string()),
                    ));
                    p.spawn(spacer());
                    p.spawn((
                        Node {
                            width: Val::Px(200.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        BorderColor(bevy::color::palettes::css::BLACK.into()),
                        BackgroundColor(bevy::color::palettes::css::WHITE.into()),
                        TextInput,
                        TextInputTextFont(
                            TextFont::from_font(BODY_FONT)
                                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        ),
                        TextInputTextColor(TextColor(bevy::color::palettes::css::BLACK.into())),
                    ));
                    p.spawn(spacer());
                    p.spawn((button(
                        get_lex_by_id(&game_over_options, "submit")
                            .lex
                            .from_language(&language.0),
                    ),))
                        .observe(submit_high_score);
                } else {
                    total_points.0 = 0;
                    p.spawn((
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text(display_text),
                    ));
                    p.spawn(spacer());
                    p.spawn(button("Continue".into()))
                        .observe(submit_high_score);
                }
            });
    });
}

fn spacer() -> impl Bundle {
    (
        TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
        Text("\n".into()),
    )
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button(text: String) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(text),
                TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color, _children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = bevy::color::palettes::css::RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

// Mouse click observers
pub fn submit_high_score(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.send_event(PostHighScore);
}

pub fn go_to_menu(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.send_event(SceneChange(AppState::Menu));
}

#[derive(Event)]
pub struct PostHighScore;

pub fn post_high_score(
    mut commands: Commands,
    mut ev_request: EventWriter<HttpRequest>,
    text_input_query: Query<&TextInputValue>,
    total_points: Res<TotalPoints>,
) {
    info!("posting high score");
    let name = match text_input_query.single() {
        Ok(t) => t.0.clone(),
        Err(_) => String::new(),
    };

    let score = total_points.0;
    let client = HttpClient::new();
    match client
        .post(LEADERBOARD_URL)
        .json(&serde_json::json!({"name": name, "score": score}))
        .try_build()
    {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => error!(?e),
    }

    commands.send_event(SceneChange(AppState::HighScores));
}

fn handle_response(
    mut ev_resp: EventReader<HttpResponse>,
    mut high_score_data: ResMut<HighScores>,
) {
    for response in ev_resp.read() {
        if let Ok(data) = response.json::<LeaderboardOutput>() {
            let high_scores = data.leaderboard;
            high_score_data.0 = high_scores;
        };
    }
}

fn setup_high_score_board(mut commands: Commands, hud: Res<Hud>) {
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::HighScores),
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
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("   High Scores\n----------------\n".into()),
                ));
                p.spawn((
                    HighScoreboard,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent
            .spawn((
                StateScoped(AppState::HighScores),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(2.0 * 480. - 100.),
                    width: Val::Px(2.0 * 600. - 200.),

                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn(button("Menu".into()));
            })
            .observe(go_to_menu);
    });
}

fn setup_credits(mut commands: Commands, hud: Res<Hud>) {
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::Credits),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    left: Val::Px(120.0),
                    align_items: AlignItems::Start,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("   Credits\n----------------\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Software Development\n--------------------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Artwork\n------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Music\n-----\nIsa Aguilar\n\n".into()),
                ));
            });

        parent
            .spawn((
                StateScoped(AppState::Credits),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(2.0 * 480. - 100.),
                    width: Val::Px(2.0 * 600. - 200.),

                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn(button("Menu".into()));
            })
            .observe(go_to_menu);
    });
}

fn update_high_scoreboard(
    high_score_data: Res<HighScores>,
    mut high_scoreboard: Query<&mut Text, With<HighScoreboard>>,
) {
    let Ok(mut text) = high_scoreboard.single_mut() else {
        return;
    };

    let mut leaders = high_score_data.0.clone();
    leaders.sort_by(|a, b| b.score.cmp(&a.score));

    let display_data = leaders
        .iter()
        .enumerate()
        .filter(|(idx, _data)| *idx < 10)
        .map(|(idx, data)| format!("#{} - {}: {}", idx + 1, data.name, data.score))
        .collect::<Vec<_>>()
        .join("\n\n");

    text.0 = display_data;
}

#[derive(Component)]
pub struct HighScoreboard;

#[derive(Event)]
pub struct RenderHighScores;

fn handle_error(mut ev_error: EventReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}

fn get_lex_by_id(assets: &Assets<GameOverLex>, id: &str) -> GameOverLex {
    assets
        .iter()
        .find(|(_, data)| data.id == id)
        .map(|(_, data)| data.clone())
        .unwrap_or_default()
}

#[derive(Deserialize, Debug)]
pub struct LeaderboardOutput {
    leaderboard: Vec<HighScoreData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HighScoreData {
    name: String,
    score: u32,
}

#[derive(Resource, Default, Debug)]
pub struct HighScores(pub Vec<HighScoreData>);

#[derive(Resource, Default, Eq, PartialEq)]
pub enum GameStatus {
    #[default]
    InProgress,
    Lose,
    Win,
}

impl GameStatus {
    fn won(&self) -> bool {
        *self == GameStatus::Win
    }

    fn lost(&self) -> bool {
        *self == GameStatus::Lose
    }
}

#[derive(Event)]
pub struct SceneChange(pub AppState);

#[derive(Resource, Default)]
pub struct PendingSceneChange(pub Option<AppState>);
