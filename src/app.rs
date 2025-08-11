use bevy::{
    asset::AssetMetaCheck, color::palettes::css::ORANGE, prelude::*, render::camera::ScalingMode,
    window::WindowResolution,
};
use bevy_aspect_ratio_mask::{AspectRatioPlugin, Hud, Resolution};

pub const RESOLUTION_WIDTH: f32 = 600.0;
pub const RESOLUTION_HEIGHT: f32 = 480.0;
pub const HALF_WIDTH_SPRITE: f32 = 10.;
pub const AFTER_LOADING_STATE: AppState = AppState::Menu;
pub const RUNNING_SPEED: f32 = 250.0;

use crate::{assets, game, menu, util};

const TITLE: &str = "The Dino Game";

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    Preload,
    Loading,
    Menu,
    Game,
    GameOver,
    HighScores,
    Credits,
}

pub fn start() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.into(),
                        name: Some(TITLE.into()),
                        resolution: WindowResolution::new(
                            RESOLUTION_WIDTH, // Window size doesn't matter here. It can be resized and the aspect ratio is kept with the defined resolution
                            RESOLUTION_HEIGHT,
                        ),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .insert_resource(DisplayLanguage::default())
        .insert_resource(InteractionRateLimit::default())
        .add_plugins((
            AspectRatioPlugin {
                resolution: Resolution {
                    width: RESOLUTION_WIDTH,
                    height: RESOLUTION_HEIGHT,
                },
                ..default()
            },
            menu::Menu,
            assets::plugin,
            game::plugin,
            util::plugin,
            #[cfg(feature = "dev")]
            crate::dev_tools::plugin,
        ))
        .run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct DisplayLanguage(pub String);

impl DisplayLanguage {
    fn default() -> Self {
        Self("english".into())
    }
}

#[derive(Component)]
pub struct DialogDisplay(pub String);

#[derive(Resource)]
pub struct InteractionRateLimit(pub Timer);

impl Default for InteractionRateLimit {
    fn default() -> Self {
        Self(Timer::from_seconds(0.20, TimerMode::Once))
    }
}
