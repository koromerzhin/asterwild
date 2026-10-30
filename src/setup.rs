use bevy::prelude::*;

use crate::blocks::{self, BlockType};
use crate::components::{Creature, Grounded, Health, Hud, NightOverlay, Player, Star, Velocity};
use crate::resources::{Inventory, Score};
use crate::save::SaveData;
use crate::world::{self, CHUNK_W, Chunk, TILE_SIZE, WORLD_H, World};

pub fn setup(mut commands: Commands, mut world: ResMut<World>) {
    commands.spawn(Camera2d);

    let mut inventory = Inventory::default();
    let mut score = Score::default();
    let mut player_x = 40.0 * TILE_SIZE;
    let mut player_y = 20.0 * TILE_SIZE;

    let mut loaded_chunks: Option<Vec<crate::world::ChunkData>> = None;
    if let Ok(data) = std::fs::read_to_string("asterwild_save.json") {
        if let Ok(save) = serde_json::from_str::<SaveData>(&data) {
            if save.seed == world.seed {
                loaded_chunks = Some(save.chunks);
                inventory = save.inventory;
                score = save.score;
                player_x = save.player_x;
                player_y = save.player_y;
            }
        }
    }

    if let Some(chunks) = loaded_chunks {
        for cd in chunks {
            let blocks = cd.blocks.clone();
            let mut entities = vec![None; (CHUNK_W * WORLD_H) as usize];
            let mut glows = vec![None; (CHUNK_W * WORLD_H) as usize];
            for y in 0..WORLD_H {
                for lx in 0..CHUNK_W {
                    let wx = cd.cx * CHUNK_W + lx;
                    if let Some(bt) = blocks[World::block_index(lx, y)] {
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
                        entities[World::block_index(lx, y)] = Some(e);
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
                            glows[World::block_index(lx, y)] = Some(g);
                        }
                    }
                }
            }
            world.chunks.insert(
                cd.cx,
                Chunk {
                    blocks,
                    entities,
                    glows,
                },
            );
        }
    }

    let spawn_cx = World::chunk_x((player_x / TILE_SIZE) as i32);
    for dc in -world::LOAD_RADIUS..=world::LOAD_RADIUS {
        world.ensure_chunk(&mut commands, spawn_cx + dc);
    }

    commands.insert_resource(inventory);
    commands.insert_resource(score);

    commands.spawn((
        Player,
        Velocity(Vec2::ZERO),
        Grounded(false),
        Health(100.0),
        crate::components::Facing(1.0),
        Sprite {
            color: Color::srgb(0.2, 0.5, 0.9),
            custom_size: Some(Vec2::new(
                world::PLAYER_HALF_W * 2.0,
                world::PLAYER_HALF_H * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(player_x, player_y, 1.0),
    ));

    for i in 0..8 {
        let cx = 12 + i * 9;
        let sy = blocks::surface_height(cx, 12345);
        commands.spawn((
            Creature {
                dir: if i % 2 == 0 { 1.0 } else { -1.0 },
                timer: 0.0,
            },
            Velocity(Vec2::ZERO),
            Grounded(false),
            Sprite {
                color: Color::srgb(0.2, 0.7, 0.3),
                custom_size: Some(Vec2::new(20.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(cx as f32 * TILE_SIZE, (sy as f32 + 1.0) * TILE_SIZE, 1.0),
        ));
    }

    // Starfield
    for i in 0..300 {
        let x = (blocks::hash2(i as i32, 0, 999) * 2.0 - 1.0) * 40.0 * world::TILE_SIZE;
        let y = (blocks::hash2(0, i as i32, 777) * 2.0 - 1.0) * 20.0 * world::TILE_SIZE + 10.0 * world::TILE_SIZE;
        commands.spawn((
            Star {
                base: Vec2::new(x, y),
                phase: blocks::hash2(i as i32, i as i32, 555) * 6.28,
            },
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                custom_size: Some(Vec2::splat(2.0)),
                ..default()
            },
            Transform::from_xyz(x, y, -5.0),
        ));
    }

    commands.spawn((
        NightOverlay,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.05, 0.0),
            custom_size: Some(Vec2::splat(4000.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0),
    ));

    commands.spawn((
        Hud,
        Text::new("Blocks: 0"),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(8),
            ..default()
        },
    ));
}
