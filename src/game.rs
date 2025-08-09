use crate::app::{AppState, HALF_WIDTH_SPRITE, RESOLUTION_HEIGHT, RESOLUTION_WIDTH, RUNNING_SPEED};
use crate::assets::custom::CustomAssets;
use bevy::ecs::system::Commands;
use bevy::input::ButtonInput;
use bevy::platform::time;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::camera::{OrthographicProjection, Projection};
use bevy::sprite::Sprite;
use bevy::ui::{AlignItems, Display, FlexDirection, Node, PositionType, Val};
use bevy_aspect_ratio_mask::Hud;
use rand::Rng;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), setup).add_systems(
        Update,
        (dino_gravity, arrow_move).run_if(in_state(AppState::Game)),
    );
}

pub fn setup(mut commands: Commands, assets: Res<CustomAssets>, hud: Res<Hud>) {
    info!("Setting up the game");
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: RESOLUTION_WIDTH,
                min_height: RESOLUTION_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
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
    commands.spawn((
        Sprite {
            color: bevy::color::palettes::css::GREEN.into(),
            custom_size: Some(Vec2::new(RESOLUTION_WIDTH * 2., 20.)),
            ..default()
        },
        Transform::from_xyz(0., -RESOLUTION_HEIGHT / 2. + 10., -1.),
    ));
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
        }
    }
}

fn dino_gravity(mut dino: Query<(&mut Transform, &mut Dino), With<Sprite>>, time: Res<Time>) {
    if let Ok((mut transform, mut dino)) = dino.single_mut() {
        let gravity = -1200.0;
        let ground_y = -RESOLUTION_HEIGHT / 2. + 50.;

        // Apply gravity if not grounded
        if !dino.grounded {
            dino.velocity.y += gravity * time.delta_secs();
        }

        // Apply velocity
        transform.translation.y += dino.velocity.y * time.delta_secs();

        // Ground collision
        if transform.translation.y <= ground_y {
            transform.translation.y = ground_y;
            dino.velocity.y = 0.0;
            dino.grounded = true;
        } else {
            dino.grounded = false;
        }
    }
}

pub fn arrow_move(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut dino: Query<(&mut Transform, &mut Sprite, &mut Dino), With<Sprite>>,
) {
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
        transform.translation.x += dino.velocity.x * time.delta_secs();

        // Wrap around screen
        if transform.translation.x > HALF_WIDTH_SPRITE + RESOLUTION_WIDTH / 2. {
            transform.translation.x = -HALF_WIDTH_SPRITE - RESOLUTION_WIDTH / 2.;
        } else if transform.translation.x < -HALF_WIDTH_SPRITE - RESOLUTION_WIDTH / 2. {
            transform.translation.x = HALF_WIDTH_SPRITE + RESOLUTION_WIDTH / 2.;
        }

        if dino.jumping || !dino.grounded {
            if keyboard_input.just_pressed(KeyCode::Space) && !dino.attacking && dino.can_attack {
                // Run animation 18-24 for attack
                dino.jump_time = 0.0;
                // dino.jump_height += 1500.0;
                dino.jumping = true;
                dino.attacking = true;
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
                    let mut rng = rand::rng();

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
