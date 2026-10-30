use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blocks::BlockType;

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub dirt: u32,
    pub stone: u32,
    pub ore: u32,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Score {
    pub kills: u32,
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
