use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blocks::BlockType;

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub dirt: u32,
    pub stone: u32,
    pub ore: u32,
    pub ingot: u32,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Score {
    pub kills: u32,
    pub days: u32,
    pub deaths: u32,
}

#[derive(Resource)]
pub struct SelectedBlock(pub BlockType);
impl Default for SelectedBlock {
    fn default() -> Self {
        Self(BlockType::Dirt)
    }
}

#[derive(Resource, Default)]
pub struct AttackCooldown {
    pub timer: f32,
}

#[derive(Resource)]
pub struct HelpVisible(pub bool);
impl Default for HelpVisible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Resource, Default)]
pub struct WeaponLevel(pub u32);

#[derive(Resource)]
pub struct DayNight {
    pub time: f32,
}
impl Default for DayNight {
    fn default() -> Self {
        Self { time: 0.25 }
    }
}

#[derive(Resource, Default)]
pub struct Shield(pub bool);

#[derive(Resource)]
pub struct WeatherTimer {
    pub timer: f32,
}
impl Default for WeatherTimer {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

#[derive(Resource, Default)]
pub struct Jetpack(pub bool);

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(Resource, Default)]
pub struct AudioStream {
    pub sink: Option<rodio::MixerDeviceSink>,
}

#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: f32,
}
impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self { timer: 60.0 }
    }
}

#[derive(Resource)]
pub struct SplashTimer {
    pub timer: f32,
}
impl Default for SplashTimer {
    fn default() -> Self {
        Self { timer: 2.0 }
    }
}

#[derive(Resource, Default)]
pub struct SaveSlot(pub u8);

#[derive(Clone, Copy)]
pub enum AudioEvent {
    Break,
    Place,
    Jump,
    Attack,
    Hit,
    Creature,
}

#[derive(Resource, Default)]
pub struct AudioQueue {
    pub events: Vec<AudioEvent>,
}

#[derive(Resource, Default)]
pub struct Muted(pub bool);
