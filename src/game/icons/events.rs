use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct PlayerFollowEvent {
    pub entity: Entity,
}

#[derive(Event, Debug)]
pub struct PlayerDamageEvent {
    pub amount: i32,
}

#[derive(Event, Debug)]
pub struct IconCaptureEvent {
    pub entity: Entity,
}

#[derive(Event, Debug)]
pub struct ProjectileSpawnEvent;
