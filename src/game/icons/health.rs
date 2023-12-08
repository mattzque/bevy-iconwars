use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::game::world::WorldBoundaryResource;
use crate::game::{settings::SettingsResource, states::GameState};

use super::components::{IconPlayerCircle, IconType, Type};
use super::{
    components::IconTransform, resources::SpatialIndexResource, IconPlayerController, ICON_SIZE,
};

pub const TOTAL_HEALTH: i32 = 1000;

#[derive(Resource, Debug)]
pub struct PlayerHealth {
    pub health: i32,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            health: TOTAL_HEALTH,
        }
    }
}

#[derive(Resource, Default)]
pub struct PlayerDamageCooldown {
    pub timer: Option<Timer>,
}

#[derive(Event, Debug)]
pub struct PlayerDamageEvent {
    pub amount: i32,
}

pub struct PlayerHealthPlugin;

impl Plugin for PlayerHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDamageEvent>();
        app.insert_resource(PlayerHealth::default());
        app.insert_resource(PlayerDamageCooldown::default());
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
        let icon_type = icon_types.get(result.key).unwrap();
        if icon_type.0 != Type::Player {
            // player damage!
            let damage = settings.player_damage_amount;

            health.health -= damage;

            info!("Health: {}", health.health);
            events.send(PlayerDamageEvent { amount: damage });

            // set cooldown timer:
            cooldown.timer = Some(Timer::from_seconds(
                settings.player_damage_cooldown,
                TimerMode::Once,
            ));
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
