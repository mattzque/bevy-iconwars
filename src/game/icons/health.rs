use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::game::world::WorldBoundaryResource;
use crate::game::{settings::SettingsResource, states::GameState};

use super::capture::IconFollowers;
use super::components::{IconPlayerCircle, IconType, Type};
use super::events::PlayerDamageEvent;
use super::{
    components::IconTransform, resources::SpatialIndexResource, IconPlayerController, ICON_SIZE,
};

#[derive(Resource, Debug, Default)]
pub struct PlayerScore {
    pub score: u32,
}

#[derive(Resource, Debug)]
pub struct PlayerHealth {
    pub health: i32,
    pub max_health: i32,
}

#[derive(Resource, Default)]
pub struct PlayerDamageCooldown {
    pub timer: Option<Timer>,
}

// TODO: the more followers you have when dropping them off, the more points you get!
//       but colliding with an icon also damages you more the more followers you have
//       so you have to balance the risk/reward of dropping off more followers at once

pub struct PlayerHealthPlugin;

impl Plugin for PlayerHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDamageEvent>();
        app.insert_resource(PlayerDamageCooldown::default());
        app.add_systems(OnEnter(GameState::MainMenu), reset_resources);
        app.add_systems(
            Update,
            (
                // icons that touch player will damage them
                damage_player_system,
                render_damage_feedback_system,
            )
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn reset_resources(mut commands: Commands, settings: Res<SettingsResource>) {
    commands.insert_resource(PlayerHealth {
        health: settings.player_max_health,
        max_health: settings.player_max_health,
    });
    commands.insert_resource(PlayerScore::default());
    commands.insert_resource(PlayerDamageCooldown::default());
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn damage_player_system(
    player: Query<&IconTransform, With<IconPlayerController>>,
    icon_types: Query<&IconType, Without<IconPlayerController>>,
    index: Res<SpatialIndexResource>,
    time: Res<Time>,
    mut cooldown: ResMut<PlayerDamageCooldown>,
    settings: Res<SettingsResource>,
    mut health: ResMut<PlayerHealth>,
    boundaries: Res<WorldBoundaryResource>,
    mut events: EventWriter<PlayerDamageEvent>,
    mut state: ResMut<NextState<GameState>>,
    followers: Res<IconFollowers>,
) {
    let player_transform = player.single();

    if boundaries.in_dropzone(player_transform.position) {
        return;
    }

    if let Some(timer) = cooldown.timer.as_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            return;
        }
    }

    for result in index.0.query(player_transform.position, ICON_SIZE) {
        if let Ok(icon_type) = icon_types.get(result.key) {
            if icon_type.0 == Type::Free || icon_type.0 == Type::Follower {
                // player damage!
                let damage = settings.player_damage_amount
                    + (followers.followers.len() as f32
                        * settings.player_damage_follower_multiplier) as i32;

                health.health -= damage;

                events.send(PlayerDamageEvent { amount: damage });

                if health.health <= 0 {
                    state.set(GameState::GameOver);
                }

                // set cooldown timer:
                cooldown.timer = Some(Timer::from_seconds(
                    settings.player_damage_cooldown,
                    TimerMode::Once,
                ));
            }
        }
    }
}

fn render_damage_feedback_system(
    mut events: EventReader<PlayerDamageEvent>,
    mut player_circle: Query<&mut Stroke, With<IconPlayerCircle>>,
    time: Res<Time>,
    mut last_damage_taken_at: Local<Option<f32>>,
) {
    if let Some(last_damage_taken_at_) = *last_damage_taken_at {
        let duration = 0.4;
        let elapsed = time.elapsed_seconds() - last_damage_taken_at_;

        // potentially set fill with transparency to red?
        let mut stroke = player_circle.single_mut();

        if elapsed > duration {
            // reset
            stroke.color = Color::hex("#444c56").unwrap();
            stroke.options.line_width = 1.0;
            *last_damage_taken_at = None;
        } else {
            stroke.color = Color::hex("#dd4c56").unwrap();
            stroke.options.line_width = (elapsed / duration) * 10.0;
        }
    }

    for PlayerDamageEvent { amount: _ } in events.read() {
        *last_damage_taken_at = Some(time.elapsed_seconds());
    }
}
