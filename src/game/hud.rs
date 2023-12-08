use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{ecs::system::Command, render::view::RenderLayers};

use super::icons::health::{PlayerHealth, TOTAL_HEALTH};
use super::icons::{IconType, Type};
use super::{camera::CAMERA_LAYER_UI, settings::SettingsResource, states::GameState};

#[derive(Resource)]
pub struct FontResource {
    pub title: Handle<Font>,
    pub text: Handle<Font>,
}

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
pub struct ScoreScreenTag;

#[derive(Component)]
pub struct HealthBarTag;

pub struct ScoreScreen {
    pub score: u32,
    pub score_total: u32,
    pub health: i32,
    pub health_total: i32,
}

impl ScoreScreen {
    pub fn score_line(&self) -> String {
        format!("{} / {}", self.score, self.score_total)
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
                    .query_filtered::<&mut Text, With<ScoreScreenTag>>()
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
                                        background_color: Color::hex("#2d333b").unwrap().into(),
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
                                        ScoreScreenTag,
                                    ));
                                });
                        });
                }
            });
        });
    }
}

pub struct TitleScreen {}

impl TitleScreen {
    const CREDITS: &'static str = "Credits
Game by @mattzq for Bevy Jam #4
Font Awesome Icons by Fonticons, Inc. (CC-BY-4.0)
Music by Brylie Christopher Oxley (CC-BY-4.0)
SFX by rubberduck (CC0)
Gasoek One by Jiashuo Zhang and JAMO (OFL)
DM Sans by Colophon Foundry, Jonny Pinhorn and Indian Type Foundry (OFL)

and Bevy (https://bevyengine.org/)";
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
                        background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
                        ..Default::default()
                    },
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
                                background_color: Color::hex("#121418DD").unwrap().into(), // Color::rgba(1.0, 0.0, 0.0, 0.1).into(),
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
                                ScoreScreenTag,
                            ));

                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(150.0),
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
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "Start Game",
                                            TextStyle {
                                                font: resource.text.clone(),
                                                font_size: 24.0,
                                                color: Color::hex("#6b9894").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                    ));
                                });

                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(150.0),
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
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "Music On",
                                            TextStyle {
                                                font: resource.text.clone(),
                                                font_size: 24.0,
                                                color: Color::hex("#6b9894").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                    ));
                                });

                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(150.0),
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
                                    RenderLayers::layer(CAMERA_LAYER_UI),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "Sound On",
                                            TextStyle {
                                                font: resource.text.clone(),
                                                font_size: 24.0,
                                                color: Color::hex("#6b9894").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
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
                                                font: resource.text.clone(),
                                                font_size: 16.0,
                                                color: Color::hex("#6b9894").unwrap(),
                                            },
                                        ),
                                        RenderLayers::layer(CAMERA_LAYER_UI),
                                        ScoreScreenTag,
                                    ));
                                });
                        });
                });
        });
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameLoading), init_hud);
        app.add_systems(Update, update_hud.run_if(in_state(GameState::GameRunning)));
    }
}

pub fn init_hud(
    mut commands: Commands,
    mut _settings: ResMut<SettingsResource>,
    health: ResMut<PlayerHealth>,
    items: Query<&IconType>,
) {
    // commands.add(FollowScreen { text: "RUST IS NOW FOLLOWING YOU".to_string() });
    // commands.add(TitleScreen {});
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
    commands.add(ScoreScreen {
        score,
        score_total: total,
        health: health.health,
        health_total: TOTAL_HEALTH,
    });
}

pub fn update_hud(mut commands: Commands, items: Query<&IconType>, health: ResMut<PlayerHealth>) {
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
    commands.add(ScoreScreen {
        score,
        score_total: total,
        health: health.health,
        health_total: TOTAL_HEALTH,
    });
}
