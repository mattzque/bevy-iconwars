use bevy::audio::VolumeLevel;
use bevy::prelude::*;
use bevy::{audio::PlaybackMode, audio::Volume};

use super::icons::events::{IconCaptureEvent, PlayerDamageEvent, PlayerFollowEvent};
use super::{icons::events::ProjectileSpawnEvent, states::GameState};

#[derive(Resource)]
pub struct AudioFileResource {
    pub music: Vec<Handle<AudioSource>>,
    pub shoot: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub capture: Handle<AudioSource>,
    pub damage: Handle<AudioSource>,
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_sound_effects_system, play_sound_effects_system)
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

#[derive(Component)]
pub struct SoundEffectTag;

fn despawn_sound_effects_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioSink), With<SoundEffectTag>>,
) {
    for (entity, sink) in query.iter_mut() {
        if sink.empty() {
            commands.entity(entity).despawn();
        }
    }
}

fn play_sound_effects_system(
    mut commands: Commands,
    mut projectile_spawn_events: EventReader<ProjectileSpawnEvent>,
    mut capture_events: EventReader<IconCaptureEvent>,
    mut follow_events: EventReader<PlayerFollowEvent>,
    mut damage_events: EventReader<PlayerDamageEvent>,
    resource: Res<AudioFileResource>,
) {
    for _ in projectile_spawn_events.read() {
        commands.spawn((
            AudioBundle {
                source: resource.shoot.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            SoundEffectTag,
        ));
    }

    for _ in follow_events.read() {
        commands.spawn((
            AudioBundle {
                source: resource.hit.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            SoundEffectTag,
        ));
    }

    for _ in capture_events.read() {
        commands.spawn((
            AudioBundle {
                source: resource.capture.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            SoundEffectTag,
        ));
    }

    for _ in damage_events.read() {
        commands.spawn((
            AudioBundle {
                source: resource.damage.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            SoundEffectTag,
        ));
    }
}
