use bevy::audio::VolumeLevel;
use bevy::prelude::*;
use bevy::{audio::PlaybackMode, audio::Volume};

use super::icons::events::{IconCaptureEvent, PlayerDamageEvent, PlayerFollowEvent};
use super::{icons::events::ProjectileSpawnEvent, states::GameState};

#[derive(Resource, Default)]
pub struct AudioSettingsResource {
    pub mute_music: bool,
    pub mute_effects: bool,
}

impl AudioSettingsResource {
    pub fn music_volume(&self) -> f32 {
        if self.mute_music {
            0.0
        } else {
            0.7
        }
    }

    pub fn effects_volume(&self) -> f32 {
        if self.mute_effects {
            0.0
        } else {
            1.0
        }
    }
}

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
        app.insert_resource(AudioSettingsResource::default());
        app.add_systems(OnEnter(GameState::MainMenu), init_music_playback_system);
        app.add_systems(
            Update,
            (despawn_sound_effects_system, play_sound_effects_system)
                .run_if(in_state(GameState::GameRunning)),
        );
        app.add_systems(
            Update,
            update_sound_volume.run_if(resource_changed::<AudioSettingsResource>()),
        );
        app.add_systems(Update, update_music_playback_system);
    }
}

#[derive(Component)]
pub struct SoundEffectTag;

#[derive(Component)]
pub struct MusicPlayer(usize);

fn init_music_playback_system(
    mut commands: Commands,
    query: Query<Entity, With<MusicPlayer>>,
    resource: Res<AudioFileResource>,
    audio_settings: Res<AudioSettingsResource>,
) {
    if query.is_empty() {
        commands.spawn((
            AudioBundle {
                source: resource.music.first().unwrap().clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(audio_settings.music_volume())),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            MusicPlayer(0),
        ));
    }
}

fn update_sound_volume(
    mut music: Query<&mut AudioSink, (With<MusicPlayer>, Without<SoundEffectTag>)>,
    mut effects: Query<&mut AudioSink, (With<SoundEffectTag>, Without<MusicPlayer>)>,
    audio_settings: Res<AudioSettingsResource>,
) {
    for sink in music.iter_mut() {
        sink.set_volume(audio_settings.music_volume());
    }
    for sink in effects.iter_mut() {
        sink.set_volume(audio_settings.effects_volume());
    }
}

fn update_music_playback_system(
    mut query: Query<
        (&mut AudioSink, &mut Handle<AudioSource>, &mut MusicPlayer),
        With<MusicPlayer>,
    >,
    resource: Res<AudioFileResource>,
) {
    if let Ok((sink, mut handle, mut player)) = query.get_single_mut() {
        if sink.empty() {
            player.0 += 1;
            if player.0 + 1 >= resource.music.len() {
                player.0 = 0;
            }
            if let Some(h) = resource.music.get(player.0) {
                *handle = h.clone();
            }
        }
    }
}

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
    audio_settings: Res<AudioSettingsResource>,
) {
    for _ in projectile_spawn_events.read() {
        commands.spawn((
            AudioBundle {
                source: resource.shoot.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::Absolute(VolumeLevel::new(audio_settings.effects_volume())),
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
                    volume: Volume::Absolute(VolumeLevel::new(audio_settings.effects_volume())),
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
                    volume: Volume::Absolute(VolumeLevel::new(audio_settings.effects_volume())),
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
                    volume: Volume::Absolute(VolumeLevel::new(audio_settings.effects_volume())),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            },
            SoundEffectTag,
        ));
    }
}
