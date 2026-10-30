use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Dirt,
    Stone,
    Ore,
    Sand,
    Snow,
    Torch,
    Ladder,
    Water,
    Door,
    DoorOpen,
    Furnace,
}

impl BlockType {
    pub fn color(self) -> Color {
        match self {
            BlockType::Dirt => Color::srgb(0.45, 0.30, 0.18),
            BlockType::Stone => Color::srgb(0.50, 0.50, 0.55),
            BlockType::Ore => Color::srgb(0.75, 0.60, 0.20),
            BlockType::Sand => Color::srgb(0.85, 0.75, 0.45),
            BlockType::Snow => Color::srgb(0.95, 0.95, 1.0),
            BlockType::Torch => Color::srgb(0.9, 0.6, 0.2),
            BlockType::Ladder => Color::srgb(0.6, 0.4, 0.2),
            BlockType::Water => Color::srgba(0.2, 0.4, 0.9, 0.6),
            BlockType::Door => Color::srgb(0.4, 0.25, 0.1),
            BlockType::DoorOpen => Color::srgb(0.4, 0.25, 0.1),
            BlockType::Furnace => Color::srgb(0.3, 0.3, 0.35),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Forest,
    Desert,
    Snow,
}

pub fn biome_at(x: i32, seed: u64) -> Biome {
    let h = hash2(x / 24, 0, seed.wrapping_mul(13));
    if h < 0.33 {
        Biome::Desert
    } else if h < 0.66 {
        Biome::Snow
    } else {
        Biome::Forest
    }
}

pub fn hash2(x: i32, y: i32, seed: u64) -> f32 {
    let mut n = seed
        .wrapping_add((x as u64).wrapping_mul(374761393))
        .wrapping_add((y as u64).wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n as f32) / (u64::MAX as f32)
}

pub fn surface_height(x: i32, seed: u64) -> i32 {
    let mut h = 12.0f32;
    h += (x as f32 * 0.07).sin() * 5.0;
    h += (x as f32 * 0.19).cos() * 2.5;
    h += (hash2(x, 0, seed) - 0.5) * 4.0;
    h.clamp(6.0, 28.0) as i32
}

pub fn is_cave(x: i32, y: i32, seed: u64) -> bool {
    if y > 2 && hash2(x, y, seed.wrapping_mul(3)) > 0.72 {
        let n = hash2(x / 3, y / 3, seed.wrapping_mul(7));
        n > 0.6
    } else {
        false
    }
}

pub fn is_ore(x: i32, y: i32, seed: u64) -> bool {
    hash2(x, y, seed.wrapping_mul(11)) > 0.93
}
