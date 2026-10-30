use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::blocks::{self, BlockType};

pub const TILE_SIZE: f32 = 32.0;
pub const CHUNK_W: i32 = 16;
pub const WORLD_H: i32 = 32;
pub const LOAD_RADIUS: i32 = 3;

pub const PLAYER_HALF_W: f32 = 10.0;
pub const PLAYER_HALF_H: f32 = 14.0;

pub const GRAVITY: f32 = 900.0;
pub const MOVE_SPEED: f32 = 180.0;
pub const JUMP_SPEED: f32 = 360.0;
pub const MAX_FALL: f32 = 600.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub cx: i32,
    pub blocks: Vec<Option<BlockType>>,
}

#[derive(Resource)]
pub struct World {
    pub seed: u64,
    pub chunks: HashMap<i32, Chunk>,
}

pub struct Chunk {
    pub blocks: Vec<Option<BlockType>>,
    pub entities: Vec<Option<Entity>>,
    pub glows: Vec<Option<Entity>>,
}

impl World {
    pub fn chunk_x(x: i32) -> i32 {
        x.div_euclid(CHUNK_W)
    }
    pub fn local_x(x: i32) -> i32 {
        x.rem_euclid(CHUNK_W)
    }
    pub fn block_index(lx: i32, y: i32) -> usize {
        (y as usize) * (CHUNK_W as usize) + (lx as usize)
    }
    pub fn get(&self, x: i32, y: i32) -> Option<BlockType> {
        if y < 0 || y >= WORLD_H {
            return None;
        }
        let cx = Self::chunk_x(x);
        let chunk = self.chunks.get(&cx)?;
        let lx = Self::local_x(x);
        chunk.blocks[Self::block_index(lx, y)]
    }
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        match self.get(x, y) {
            Some(BlockType::DoorOpen) => false,
            Some(_) => true,
            None => false,
        }
    }
    pub fn ensure_chunk(&mut self, commands: &mut Commands, cx: i32) {
        if self.chunks.contains_key(&cx) {
            return;
        }
        let seed = self.seed;
        let mut blocks = vec![None; (CHUNK_W * WORLD_H) as usize];
        for lx in 0..CHUNK_W {
            let wx = cx * CHUNK_W + lx;
            let surface = blocks::surface_height(wx, seed);
            for y in 0..surface {
                if blocks::is_cave(wx, y, seed) {
                    continue;
                }
                let bt = if y < surface - 4 {
                    if blocks::is_ore(wx, y, seed) {
                        BlockType::Ore
                    } else {
                        BlockType::Stone
                    }
                } else {
                    match blocks::biome_at(wx, seed) {
                        blocks::Biome::Desert => BlockType::Sand,
                        blocks::Biome::Snow => BlockType::Snow,
                        blocks::Biome::Forest => BlockType::Dirt,
                    }
                };
                blocks[Self::block_index(lx, y)] = Some(bt);
            }
        }
        let mut entities = vec![None; (CHUNK_W * WORLD_H) as usize];
        let mut glows = vec![None; (CHUNK_W * WORLD_H) as usize];
        for y in 0..WORLD_H {
            for lx in 0..CHUNK_W {
                let wx = cx * CHUNK_W + lx;
                if let Some(bt) = blocks[Self::block_index(lx, y)] {
                    let e = commands
                        .spawn((
                            Sprite {
                                color: bt.color(),
                                custom_size: Some(Vec2::splat(TILE_SIZE)),
                                ..default()
                            },
                            Transform::from_xyz(
                                wx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                0.0,
                            ),
                        ))
                        .id();
                    entities[Self::block_index(lx, y)] = Some(e);
                    if bt == BlockType::Torch {
                        let g = commands
                            .spawn((
                                Sprite {
                                    color: Color::srgba(1.0, 0.8, 0.3, 0.35),
                                    custom_size: Some(Vec2::splat(TILE_SIZE * 3.0)),
                                    ..default()
                                },
                                Transform::from_xyz(
                                    wx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                    y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                    0.5,
                                ),
                            ))
                            .id();
                        glows[Self::block_index(lx, y)] = Some(g);
                    }
                }
            }
        }
        self.chunks.insert(
            cx,
            Chunk {
                blocks,
                entities,
                glows,
            },
        );
    }
    pub fn unload_chunk(&mut self, commands: &mut Commands, cx: i32) {
        if let Some(chunk) = self.chunks.remove(&cx) {
            for e in chunk.entities.iter().flatten() {
                commands.entity(*e).despawn();
            }
            for g in chunk.glows.iter().flatten() {
                commands.entity(*g).despawn();
            }
        }
    }
}
