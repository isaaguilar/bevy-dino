use crate::app::AppState;
use crate::app::DialogDisplay;
use crate::app::DisplayLanguage;
use crate::app::RESOLUTION_WIDTH;
use crate::assets::lexi::menu::{Choice, MenuData};
use bevy::prelude::*;
use bevy_aspect_ratio_mask::Hud;

mod actions;
mod inputs;
mod layouts;

pub struct Menu;

impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveMenu::default());
        app.add_event::<ChangeMenu>();

        app.add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(Update, move_choice_marker.run_if(in_state(AppState::Menu)))
            .add_systems(
                Update,
                change_menu.run_if(on_event::<ChangeMenu>.and(in_state(AppState::Menu))),
            )
            .add_systems(OnExit(AppState::Menu), leave_menu);

        app.add_plugins(inputs::plugin);
    }
}

#[derive(Resource, Default, Clone)]
pub struct ActiveMenu {
    pub opt: Option<MenuData>,
}

impl ActiveMenu {
    pub fn reset(&mut self) {
        *self = Self { ..default() };
    }
}

#[derive(Component)]
struct SelectionMarker(Choice);

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
struct CurrentSelection(Option<Choice>);

impl Default for CurrentSelection {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Event)]
struct ChangeMenu(String);

impl ChangeMenu {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Component)]
struct Language(String);

impl Language {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Component)]
struct GoToMenu(String);

impl GoToMenu {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

fn menu_setup(
    mut commands: Commands,
    mut bg: ResMut<ClearColor>,
    display_language: ResMut<DisplayLanguage>,
) {
    info!("Menu");
    info!(language = display_language.0);
    bg.0 = Color::srgb(0.2, 0.2, 0.2);
    // commands.spawn((StateScoped(AppState::Menu), Camera2d::default()));

    commands.send_event(ChangeMenu::new("main menu"));
    return;
}

fn change_menu(
    mut changes: EventReader<ChangeMenu>,
    mut commands: Commands,
    display_language: ResMut<DisplayLanguage>,
    mut dialog_message: ResMut<ActiveMenu>,
    menu_data: Res<Assets<MenuData>>,
    dialog_display_query: Query<(Entity, &DialogDisplay), With<DialogDisplay>>,
    hud: Res<Hud>,
) {
    info!("Spawning menu");
    let hud_entity = hud.0;

    let Some(event) = changes.read().next() else {
        return;
    };

    let menu_id = &event.0;

    dialog_message.opt = menu_data
        .into_inner()
        .iter()
        .filter(|(_, data)| data.id == menu_id.clone())
        .map(|(_, data)| data.clone())
        .next();

    let dialog = match &dialog_message.opt {
        Some(d) => d,
        None => {
            for (entity, _) in dialog_display_query.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    };

    for (entity, dialog_display) in dialog_display_query.iter() {
        if dialog_display.0 != dialog.id {
            commands.entity(entity).despawn();
        } else {
            return;
        }
    }

    let padding_x = RESOLUTION_WIDTH / 32.0;
    commands.entity(hud_entity).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::Menu),
                DialogDisplay(dialog.id.clone()),
                layouts::menu_layout(RESOLUTION_WIDTH - 2. * padding_x),
            ))
            .with_children(|p| {
                let text = dialog.lex.from_language(&display_language.0);

                p.spawn(layouts::header_layout(&text));

                match &dialog.choices {
                    Some(choices) => {
                        for (_index, choice) in choices.iter().enumerate() {
                            let text = choice.choice.lex.from_language(&display_language.0);

                            let mut button =
                                p.spawn((layouts::button_layout(&text, choice.clone()),));
                            button.observe(inputs::mouse_move);
                            button.observe(inputs::mouse_over);

                            match &choice.choice.action {
                                Some(action) => match action.as_str() {
                                    "start_game" => {
                                        button.observe(inputs::click_start_game);
                                    }
                                    "show_credits" => {
                                        button.observe(inputs::click_show_credits);
                                    }
                                    "show_leaderboard" => {
                                        button.observe(inputs::click_show_leaderboard);
                                    }
                                    "english" | "spanish" => {
                                        button
                                            .insert(Language::new(action))
                                            .observe(inputs::click_language_selection);
                                    }
                                    _ => {}
                                },
                                None => {}
                            }

                            match &choice.choice.next_id {
                                Some(id) => {
                                    button
                                        .insert(GoToMenu::new(id))
                                        .observe(inputs::click_menu_selection);
                                }
                                None => {}
                            }
                        }
                    }
                    None => {}
                }
            });
    });

    return;
}

fn move_choice_marker(
    display_language: ResMut<DisplayLanguage>,
    dialog_message: Res<ActiveMenu>,
    current_selection: Res<CurrentSelection>,
    mut button: Query<&mut BackgroundColor>,
    mut selections: Query<(&mut TextSpan, &ChildOf), With<layouts::MenuOption>>,
) {
    let Some(current_choice) = &current_selection.0 else {
        return;
    };

    let dialog = match &dialog_message.opt {
        Some(d) => d,
        None => {
            return;
        }
    };

    let choices = match &dialog.choices {
        Some(choices) => {
            if choices.is_empty() {
                return;
            }
            choices
        }
        None => return,
    };

    for (idx, choice) in choices.iter().enumerate() {
        let text = choice.choice.lex.from_language(&display_language.0);

        if current_choice.id == choice.id.clone() {
            for (text_idx, (mut text_span, parent)) in selections.iter_mut().enumerate() {
                if idx == text_idx {
                    *text_span = TextSpan::new(format!("> {}", text.clone()));
                    if let Ok(mut bg) = button.get_mut(parent.0) {
                        bg.0 = bevy::color::palettes::css::DARK_SLATE_GREY.into();
                    }
                }
            }
        } else {
            for (text_idx, (mut text_span, parent)) in selections.iter_mut().enumerate() {
                if idx == text_idx {
                    *text_span = TextSpan::new(format!(" {}", text.clone()));
                    if let Ok(mut bg) = button.get_mut(parent.0) {
                        bg.0 = bevy::color::palettes::css::CADET_BLUE.into();
                    }
                }
            }
        }
    }
}

fn leave_menu(mut dialog_message: ResMut<ActiveMenu>) {
    dialog_message.reset();
}
