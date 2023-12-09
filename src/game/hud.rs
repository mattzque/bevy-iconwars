use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{ecs::system::Command, render::view::RenderLayers};

use super::icons::events::PlayerFollowEvent;
use super::icons::health::{PlayerHealth, PlayerScore};
use super::icons::{IconSheetRef, IconType, Type};
use super::{camera::CAMERA_LAYER_UI, settings::SettingsResource, states::GameState};

#[derive(Resource)]
pub struct FontResource {
    pub title: Handle<Font>,
    pub text: Handle<Font>,
    pub text2: Handle<Font>,
}

#[derive(Component)]
pub struct FollowScreenTag;

pub struct FollowScreen {
    pub text: String,
}

impl Command for FollowScreen {
    fn apply(self, world: &mut World) {
        world.resource_scope::<FontResource, ()>(|world, resource| {
            world
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    FollowScreenTag,
                    ScreenTag,
                    RenderLayers::layer(CAMERA_LAYER_UI),
                ))
                .with_children(|child| {
                    child
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    min_width: Val::Px(700.0),
                                    // height: Val::Px(200.0),
                                    padding: UiRect::all(Val::Px(90.0)),
                                    margin: UiRect::bottom(Val::Px(128.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                background_color: Color::hex("#22272eDD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                ..Default::default()
                            },
                            RenderLayers::layer(CAMERA_LAYER_UI),
                        ))
                        .with_children(|child| {
                            child.spawn((
                                TextBundle::from_section(
                                    self.text,
                                    TextStyle {
                                        font: resource.title.clone(),
                                        font_size: 48.0,
                                        color: Color::hex("#6b9894").unwrap(),
                                    },
                                ),
                                RenderLayers::layer(CAMERA_LAYER_UI),
                            ));
                        });
                });
        });
    }
}

#[derive(Component)]
pub struct ScreenTag;

#[derive(Component)]
pub struct ScoreScreenTextTag;

#[derive(Component)]
pub struct HealthBarTag;

pub struct ScoreScreen {
    pub score: u32,
    pub health: i32,
    pub health_total: i32,
}

impl ScoreScreen {
    pub fn score_line(&self) -> String {
        format!("{}", self.score)
    }

    pub fn health_percent(&self) -> f32 {
        (self.health as f32 / self.health_total as f32) * 100.0
    }
}

impl Command for ScoreScreen {
    fn apply(self, world: &mut World) {
        world.resource_scope::<FontResource, ()>(|world, resource| {
            world.resource_scope::<DiagnosticsStore, ()>(|world, diagnostics| {
                let mut contents = format!("Score: {}", self.score_line()); // \nFPS: {:.2}", self.score_line(), diagn);
                if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                    if let Some(value) = fps.smoothed() {
                        contents.push_str(&format!("\n{:.2} FPS", value));
                    }
                }

                if let Ok(mut text) = world
                    .query_filtered::<&mut Text, With<ScoreScreenTextTag>>()
                    .get_single_mut(world)
                {
                    text.sections[0].value = contents;
                    if let Ok(mut style) = world
                        .query_filtered::<&mut Style, With<HealthBarTag>>()
                        .get_single_mut(world)
                    {
                        style.width = Val::Percent(self.health_percent());
                    }
                } else {
                    world
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Vw(100.0),
                                    height: Val::Vh(100.0),
                                    position_type: PositionType::Relative,
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::FlexStart,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ScreenTag,
                            RenderLayers::layer(CAMERA_LAYER_UI),
                        ))
                        .with_children(|parent| {
                            // health bar
                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            width: Val::Vw(40.0),
                                            max_width: Val::Px(400.0),
                                            height: Val::Px(32.0),
                                            padding: UiRect::all(Val::Px(6.0)),
                                            position_type: PositionType::Absolute,
                                            bottom: Val::Px(64.0),
                                            left: Val::Px(32.0),
                                            ..Default::default()
                                        },
                                        background_color: Color::hex("#3c454f").unwrap().into(),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        NodeBundle {
                                            style: Style {
                                                width: Val::Percent(self.health_percent()),
                                                height: Val::Percent(100.0),
                                                ..Default::default()
                                            },
                                            background_color: Color::hex("#56837f").unwrap().into(),
                                            ..Default::default()
                                        },
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                        HealthBarTag,
                                    ));
                                });

                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            min_width: Val::Px(200.0),
                                            // height: Val::Px(200.0),
                                            padding: UiRect::all(Val::Px(32.0)),
                                            margin: UiRect::bottom(Val::Px(64.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::Center,
                                            ..Default::default()
                                        },
                                        background_color: Color::hex("#22272eDD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            contents,
                                            TextStyle {
                                                font: resource.text.clone(),
                                                font_size: 32.0,
                                                color: Color::hex("#6b9894").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                        ScoreScreenTextTag,
                                    ));
                                });
                        });
                }
            });
        });
    }
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ButtonKind {
    StartGame,
    ResumeGame,
    ToggleMusic,
    ToggleSound,
    BackToMainMenu,
    QuitGame,
}

pub struct ButtonChildBuilder {
    label: &'static str,
    kind: ButtonKind,
}

impl ButtonChildBuilder {
    fn spawn(&self, parent: &mut WorldChildBuilder, resource: &FontResource) {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(350.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(0.0)),
                        padding: UiRect::all(Val::Px(16.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // border_color: BorderColor(Color::hex("#444c56").unwrap()),
                    background_color: Color::hex("#2d333b").unwrap().into(), // hex("#6b9894DD").unwrap().into(),
                    ..default()
                },
                self.kind,
                RenderLayers::layer(CAMERA_LAYER_UI),
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        self.label.to_uppercase(),
                        TextStyle {
                            font: resource.text.clone(),
                            font_size: 32.0,
                            color: Color::hex("#7fc1bb").unwrap(),
                        },
                    ),
                    RenderLayers::layer(CAMERA_LAYER_UI),
                ));
            });
    }
}

pub struct TitleScreen {}

impl TitleScreen {
    const CREDITS: &'static str = "Game by @mattzq for Bevy Jam #4
Font Awesome Icons by Fonticons, Inc. (CC-BY-4.0)
Music by Brylie Christopher Oxley (CC-BY-4.0)
SFX by rubberduck (CC0)
Gasoek One by Jiashuo Zhang and JAMO (OFL)
DM Sans by Colophon Foundry, Jonny Pinhorn and Indian Type Foundry (OFL)

and Bevy (https://bevyengine.org/)";
    const INSTRUCTIONS: &'static str = "Instructions:
Shoot icons to make them follow you.
You take damage if they touch you.
Bring them to the dropzone (the center area) to score points.
You make more points the more followers you bring at once.
You take more damage the more followers you have.
    
Controls:
Up / A - Move Forward 
Down / S - Move Backward 
Left / A - Strafe Left
Right / D - Strafe Right
Space / Left Click - Shoot
Mouse Wheel - Zoom
Escape - Pause Menu";
}

impl Command for TitleScreen {
    fn apply(self, world: &mut World) {
        world.resource_scope::<FontResource, ()>(|world, resource| {
            world
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::hex("#12141844").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                        ..Default::default()
                    },
                    ScreenTag,
                    RenderLayers::layer(CAMERA_LAYER_UI),
                ))
                .with_children(|child| {
                    child
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    min_width: Val::Px(200.0),
                                    // height: Val::Px(200.0),
                                    padding: UiRect::all(Val::Px(32.0)),
                                    margin: UiRect::bottom(Val::Px(64.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(16.0),
                                    ..Default::default()
                                },
                                background_color: Color::hex("#121418CC").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                ..Default::default()
                            },
                            RenderLayers::layer(CAMERA_LAYER_UI),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "ICON WARS!",
                                    TextStyle {
                                        font: resource.title.clone(),
                                        font_size: 128.0,
                                        color: Color::hex("#6b9894").unwrap(),
                                    },
                                ),
                                RenderLayers::layer(CAMERA_LAYER_UI),
                            ));

                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            min_width: Val::Px(200.0),
                                            // height: Val::Px(200.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(8.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::FlexStart,
                                            ..Default::default()
                                        },
                                        background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            Self::INSTRUCTIONS,
                                            TextStyle {
                                                font: resource.text2.clone(),
                                                font_size: 21.0,
                                                color: Color::hex("#adbacb").unwrap(),
                                            },
                                        )
                                        .with_text_alignment(TextAlignment::Center),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                    ));
                                });

                            ButtonChildBuilder {
                                label: "Start Game",
                                kind: ButtonKind::StartGame,
                            }
                            .spawn(parent, &resource);
                            ButtonChildBuilder {
                                label: "Music Off",
                                kind: ButtonKind::ToggleMusic,
                            }
                            .spawn(parent, &resource);
                            ButtonChildBuilder {
                                label: "Sound Off",
                                kind: ButtonKind::ToggleSound,
                            }
                            .spawn(parent, &resource);

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                ButtonChildBuilder {
                                    label: "Quit Game",
                                    kind: ButtonKind::QuitGame,
                                }
                                .spawn(parent, &resource);
                            }

                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            min_width: Val::Px(200.0),
                                            // height: Val::Px(200.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(8.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::FlexStart,
                                            ..Default::default()
                                        },
                                        background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            Self::CREDITS,
                                            TextStyle {
                                                font: resource.text2.clone(),
                                                font_size: 21.0,
                                                color: Color::hex("#adbacb").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                    ));
                                });
                        });
                });
        });
    }
}

pub struct GameOverScreen {
    pub score: u32,
    pub score_total: u32,
}

impl Command for GameOverScreen {
    fn apply(self, world: &mut World) {
        world.resource_scope::<FontResource, ()>(|world, resource| {
            world
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                        ..Default::default()
                    },
                    ScreenTag,
                    RenderLayers::layer(CAMERA_LAYER_UI),
                ))
                .with_children(|child| {
                    child
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    min_width: Val::Px(200.0),
                                    // height: Val::Px(200.0),
                                    padding: UiRect::all(Val::Px(32.0)),
                                    margin: UiRect::bottom(Val::Px(64.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(16.0),
                                    ..Default::default()
                                },
                                background_color: Color::hex("#121418CC").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                ..Default::default()
                            },
                            RenderLayers::layer(CAMERA_LAYER_UI),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "GAME OVER",
                                    TextStyle {
                                        font: resource.title.clone(),
                                        font_size: 128.0,
                                        color: Color::hex("#6b9894").unwrap(),
                                    },
                                ),
                                RenderLayers::layer(CAMERA_LAYER_UI),
                            ));

                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            min_width: Val::Px(200.0),
                                            // height: Val::Px(200.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(8.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::FlexStart,
                                            ..Default::default()
                                        },
                                        background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            format!(
                                                "You have lost the game!\nScore: {} of {}",
                                                self.score, self.score_total
                                            ),
                                            TextStyle {
                                                font: resource.text2.clone(),
                                                font_size: 21.0,
                                                color: Color::hex("#adbacb").unwrap(),
                                            },
                                        )
                                        .with_text_alignment(TextAlignment::Center),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                    ));
                                });

                            ButtonChildBuilder {
                                label: "Back to Main Menu",
                                kind: ButtonKind::BackToMainMenu,
                            }
                            .spawn(parent, &resource);
                        });
                });
        });
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), enter_main_menu_system);
        app.add_systems(OnEnter(GameState::GameOver), enter_game_over_system);
        app.add_systems(OnEnter(GameState::GameRunning), enter_game_running_system);

        app.add_systems(
            Update,
            update_button_interaction_system
                .run_if(in_state(GameState::MainMenu).or_else(in_state(GameState::GameOver))),
        );

        app.add_systems(
            Update,
            (update_hud_system, show_icon_follower_added_system)
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

pub fn enter_main_menu_system(mut commands: Commands, screens: Query<Entity, With<ScreenTag>>) {
    for screen_entity in screens.iter() {
        commands.entity(screen_entity).despawn_recursive();
    }
    commands.add(TitleScreen {});
}

pub fn enter_game_running_system(
    mut commands: Commands,
    mut _settings: ResMut<SettingsResource>,
    health: ResMut<PlayerHealth>,
    score: ResMut<PlayerScore>,
    items: Query<&IconType>,
    screens: Query<Entity, With<ScreenTag>>,
) {
    for screen_entity in screens.iter() {
        commands.entity(screen_entity).despawn_recursive();
    }
    commands.add(ScoreScreen {
        score: score.score,
        health: health.health,
        health_total: health.max_health,
    });
}

pub fn update_hud_system(
    mut commands: Commands,
    items: Query<&IconType>,
    health: ResMut<PlayerHealth>,
    score: ResMut<PlayerScore>,
) {
    commands.add(ScoreScreen {
        score: score.score,
        health: health.health,
        health_total: health.max_health,
    });
}

fn enter_game_over_system(
    mut commands: Commands,
    mut _settings: ResMut<SettingsResource>,
    items: Query<&IconType>,
    screens: Query<Entity, With<ScreenTag>>,
) {
    for screen_entity in screens.iter() {
        commands.entity(screen_entity).despawn_recursive();
    }

    let mut score = 0;
    let mut total = 0;
    items.for_each(|icon_type| {
        if icon_type.0 == Type::Player {
            return;
        }

        if icon_type.0 == Type::Captured {
            score += 1;
        }

        total += 1;
    });
    commands.add(GameOverScreen {
        score,
        score_total: total,
    });
}

#[allow(clippy::type_complexity)]
fn update_button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonKind),
        (Changed<Interaction>, With<Button>),
    >,
    mut exit: EventWriter<AppExit>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button_kind) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button_kind {
                ButtonKind::StartGame => {
                    state.set(GameState::GameRunning);
                }
                ButtonKind::ResumeGame => {
                    state.set(GameState::GameRunning);
                }
                ButtonKind::BackToMainMenu => {
                    state.set(GameState::MainMenu);
                }
                ButtonKind::ToggleMusic => todo!(),
                ButtonKind::ToggleSound => todo!(),
                ButtonKind::QuitGame => {
                    exit.send(AppExit);
                }
            },
            Interaction::Hovered => {
                *color = Color::hex("#3c454f").unwrap().into();
            }
            Interaction::None => {
                *color = Color::hex("#2d333b").unwrap().into();
            }
        }
    }
}

fn show_icon_follower_added_system(
    mut commands: Commands,
    mut events: EventReader<PlayerFollowEvent>,
    time: Res<Time>,
    mut last_shown_at: Local<Option<f32>>,
    icons: Query<&IconSheetRef>,
    screens: Query<Entity, With<FollowScreenTag>>,
) {
    if let Some(last_shown_at_) = *last_shown_at {
        let duration = 0.8;
        let elapsed = time.elapsed_seconds() - last_shown_at_;

        if elapsed > duration {
            for follow_entity in screens.iter() {
                commands.entity(follow_entity).despawn_recursive();
            }
        }
    }

    for PlayerFollowEvent { entity } in events.read() {
        let name = icons.get(*entity).unwrap().icon_name.to_uppercase();
        *last_shown_at = Some(time.elapsed_seconds());

        for follow_entity in screens.iter() {
            commands.entity(follow_entity).despawn_recursive();
        }

        commands.add(FollowScreen {
            text: format!("{} IS NOW FOLLOWING YOU", name),
        });
    }
}
