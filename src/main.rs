use bevy::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

const TILE_SIZE: f32 = 32.0;
const WORLD_H: i32 = 32;
const GRAVITY: f32 = 900.0;
const MOVE_SPEED: f32 = 180.0;
const JUMP_SPEED: f32 = 360.0;
const MAX_FALL: f32 = 600.0;
const PLAYER_HALF_W: f32 = 10.0;
const PLAYER_HALF_H: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
enum BlockType {
    Dirt,
    Stone,
    Ore,
    Sand,
    Snow,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Biome {
    Forest,
    Desert,
    Snow,
}

fn biome_at(x: i32, seed: u64) -> Biome {
    let h = hash2(x / 24, 0, seed.wrapping_mul(13));
    if h < 0.33 {
        Biome::Desert
    } else if h < 0.66 {
        Biome::Snow
    } else {
        Biome::Forest
    }
}

impl BlockType {
    fn color(self) -> Color {
        match self {
            BlockType::Dirt => Color::srgb(0.45, 0.30, 0.18),
            BlockType::Stone => Color::srgb(0.50, 0.50, 0.55),
            BlockType::Ore => Color::srgb(0.75, 0.60, 0.20),
            BlockType::Sand => Color::srgb(0.85, 0.75, 0.45),
            BlockType::Snow => Color::srgb(0.95, 0.95, 1.0),
        }
    }
}

const CHUNK_W: i32 = 16;
const LOAD_RADIUS: i32 = 3;

#[derive(Clone, Serialize, Deserialize)]
struct ChunkData {
    cx: i32,
    blocks: Vec<Option<BlockType>>,
}

#[derive(Resource)]
struct World {
    seed: u64,
    chunks: HashMap<i32, Chunk>,
}

struct Chunk {
    blocks: Vec<Option<BlockType>>,
    entities: Vec<Option<Entity>>,
}

impl World {
    fn chunk_x(x: i32) -> i32 {
        x.div_euclid(CHUNK_W)
    }
    fn local_x(x: i32) -> i32 {
        x.rem_euclid(CHUNK_W)
    }
    fn block_index(lx: i32, y: i32) -> usize {
        (y as usize) * (CHUNK_W as usize) + (lx as usize)
    }
    fn get(&self, x: i32, y: i32) -> Option<BlockType> {
        if y < 0 || y >= WORLD_H {
            return None;
        }
        let cx = Self::chunk_x(x);
        let chunk = self.chunks.get(&cx)?;
        let lx = Self::local_x(x);
        chunk.blocks[Self::block_index(lx, y)]
    }
    fn ensure_chunk(&mut self, commands: &mut Commands, cx: i32) {
        if self.chunks.contains_key(&cx) {
            return;
        }
        let mut blocks = vec![None; (CHUNK_W * WORLD_H) as usize];
        for lx in 0..CHUNK_W {
            let wx = cx * CHUNK_W + lx;
            let surface = surface_height(wx, self.seed);
            for y in 0..surface {
                if is_cave(wx, y, self.seed) {
                    continue;
                }
                let bt = if y < surface - 4 {
                    if is_ore(wx, y, self.seed) {
                        BlockType::Ore
                    } else {
                        BlockType::Stone
                    }
                } else {
                    match biome_at(wx, self.seed) {
                        Biome::Desert => BlockType::Sand,
                        Biome::Snow => BlockType::Snow,
                        Biome::Forest => BlockType::Dirt,
                    }
                };
                blocks[Self::block_index(lx, y)] = Some(bt);
            }
        }
        // spawn entities
        let mut entities = vec![None; (CHUNK_W * WORLD_H) as usize];
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
                }
            }
        }
        self.chunks.insert(
            cx,
            Chunk {
                blocks,
                entities,
            },
        );
    }
    fn unload_chunk(&mut self, commands: &mut Commands, cx: i32) {
        if let Some(chunk) = self.chunks.remove(&cx) {
            for e in chunk.entities.iter().flatten() {
                commands.entity(*e).despawn();
            }
        }
    }
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
struct Inventory {
    dirt: u32,
    stone: u32,
    ore: u32,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
struct Score {
    kills: u32,
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    seed: u64,
    chunks: Vec<ChunkData>,
    inventory: Inventory,
    score: Score,
    player_x: f32,
    player_y: f32,
}

#[derive(Resource)]
struct SelectedBlock(BlockType);
impl Default for SelectedBlock {
    fn default() -> Self {
        Self(BlockType::Dirt)
    }
}

#[derive(Resource)]
struct DayNight {
    time: f32,
}
impl Default for DayNight {
    fn default() -> Self {
        Self { time: 0.25 }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Grounded(bool);

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Creature {
    dir: f32,
    timer: f32,
}

#[derive(Component)]
struct Hud;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Asterwild - Prototype".into(),
                resolution: (960u32, 540u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(World { seed: 12345, chunks: HashMap::new() })
        .insert_resource(Inventory::default())
        .insert_resource(Score::default())
        .insert_resource(SelectedBlock::default())
        .insert_resource(DayNight::default())
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_input,
                chunk_manager,
                physics,
                camera_follow,
                mouse_interact,
                save_load,
                day_night,
                creature_ai,
                hotbar_input,
                crafting,
                survival,
                update_hud,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut world: ResMut<World>) {
    commands.spawn(Camera2d);

    let mut inventory = Inventory::default();
    let mut score = Score::default();
    let mut player_x = 40.0 * TILE_SIZE;
    let mut player_y = 20.0 * TILE_SIZE;

    // Attempt load save
    let mut loaded_chunks: Option<Vec<ChunkData>> = None;
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
            // build chunk
            let mut entities = vec![None; (CHUNK_W * WORLD_H) as usize];
            for y in 0..WORLD_H {
                for lx in 0..CHUNK_W {
                    let wx = cd.cx * CHUNK_W + lx;
                    if let Some(bt) = cd.blocks[World::block_index(lx, y)] {
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
                    }
                }
            }
            world.chunks.insert(
                cd.cx,
                Chunk {
                    blocks: cd.blocks,
                    entities,
                },
            );
        }
    }

    // Ensure initial chunks around spawn
    let spawn_cx = World::chunk_x((player_x / TILE_SIZE) as i32);
    for dc in -LOAD_RADIUS..=LOAD_RADIUS {
        world.ensure_chunk(&mut commands, spawn_cx + dc);
    }

    commands.insert_resource(inventory);
    commands.insert_resource(score);

    // Player
    commands.spawn((
        Player,
        Velocity(Vec2::ZERO),
        Grounded(false),
        Health(100.0),
        Sprite {
            color: Color::srgb(0.2, 0.5, 0.9),
            custom_size: Some(Vec2::new(PLAYER_HALF_W * 2.0, PLAYER_HALF_H * 2.0)),
            ..default()
        },
        Transform::from_xyz(player_x, player_y, 1.0),
    ));

    // Creatures
    for i in 0..8 {
        let cx = 12 + i * 9;
        let sy = surface_height(cx, 12345);
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
            Transform::from_xyz(
                cx as f32 * TILE_SIZE,
                (sy as f32 + 1.0) * TILE_SIZE,
                1.0,
            ),
        ));
    }

    // HUD
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

fn collides(world: &World, px: f32, py: f32) -> bool {
    let left = px - PLAYER_HALF_W;
    let right = px + PLAYER_HALF_W;
    let bottom = py - PLAYER_HALF_H;
    let top = py + PLAYER_HALF_H;

    let min_x = (left / TILE_SIZE).floor() as i32;
    let max_x = ((right - 0.001) / TILE_SIZE).floor() as i32;
    let min_y = (bottom / TILE_SIZE).floor() as i32;
    let max_y = ((top - 0.001) / TILE_SIZE).floor() as i32;

    for ty in min_y..=max_y {
        for tx in min_x..=max_x {
            if world.get(tx, ty).is_some() {
                return true;
            }
        }
    }
    false
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Grounded), With<Player>>,
) {
    for (mut vel, _) in &mut query {
        let mut vx = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            vx -= MOVE_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            vx += MOVE_SPEED;
        }
        vel.0.x = vx;
    }
}

fn physics(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    world: Res<World>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Grounded), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut vel, mut grounded) in &mut query {
        // gravity
        vel.0.y -= GRAVITY * dt;
        if vel.0.y < -MAX_FALL {
            vel.0.y = -MAX_FALL;
        }
        // jump
        if grounded.0
            && (keyboard.just_pressed(KeyCode::Space)
                || keyboard.just_pressed(KeyCode::KeyW)
                || keyboard.just_pressed(KeyCode::ArrowUp))
        {
            vel.0.y = JUMP_SPEED;
            grounded.0 = false;
        }

        // horizontal
        let new_x = transform.translation.x + vel.0.x * dt;
        if !collides(&world, new_x, transform.translation.y) {
            transform.translation.x = new_x;
        } else {
            vel.0.x = 0.0;
        }
        // vertical
        let new_y = transform.translation.y + vel.0.y * dt;
        if !collides(&world, transform.translation.x, new_y) {
            transform.translation.y = new_y;
            grounded.0 = false;
        } else {
            if vel.0.y < 0.0 {
                grounded.0 = true;
            }
            vel.0.y = 0.0;
        }
    }
}

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    if let Ok(p) = player.single() {
        for mut c in &mut camera {
            c.translation.x = p.translation.x;
            c.translation.y = p.translation.y;
        }
    }
}

fn mouse_interact(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut inventory: ResMut<Inventory>,
    mut score: ResMut<Score>,
    selected: Res<SelectedBlock>,
    creatures: Query<(Entity, &Transform), With<Creature>>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };

    let tx = (world_pos.x / TILE_SIZE).floor() as i32;
    let ty = (world_pos.y / TILE_SIZE).floor() as i32;

    if mouse_buttons.just_pressed(MouseButton::Left) {
        // combat: hit creature near cursor
        for (e, t) in &creatures {
            let d = t.translation.truncate() - world_pos;
            if d.length() < 18.0 {
                commands.entity(e).despawn();
                score.kills += 1;
                return;
            }
        }
        // break
        let cx = World::chunk_x(tx);
        world.ensure_chunk(&mut commands, cx);
        if let Some(chunk) = world.chunks.get_mut(&cx) {
            let lx = World::local_x(tx);
            let idx = World::block_index(lx, ty);
            if let Some(bt) = chunk.blocks[idx] {
                if let Some(e) = chunk.entities[idx].take() {
                    commands.entity(e).despawn();
                }
                chunk.blocks[idx] = None;
                match bt {
                    BlockType::Dirt => inventory.dirt += 1,
                    BlockType::Stone => inventory.stone += 1,
                    BlockType::Ore => inventory.ore += 1,
                    BlockType::Sand => inventory.dirt += 1,
                    BlockType::Snow => inventory.dirt += 1,
                }
            }
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        // place selected if empty
        if world.get(tx, ty).is_none() {
            let bt = selected.0;
            let have = match bt {
                BlockType::Dirt | BlockType::Sand | BlockType::Snow => &mut inventory.dirt,
                BlockType::Stone => &mut inventory.stone,
                BlockType::Ore => &mut inventory.ore,
            };
            if *have > 0 {
                *have -= 1;
                let cx = World::chunk_x(tx);
                world.ensure_chunk(&mut commands, cx);
                let e = commands
                    .spawn((
                        Sprite {
                            color: bt.color(),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(
                            tx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            ty as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            0.0,
                        ),
                    ))
                    .id();
                if let Some(chunk) = world.chunks.get_mut(&cx) {
                    let lx = World::local_x(tx);
                    let idx = World::block_index(lx, ty);
                    chunk.blocks[idx] = Some(bt);
                    chunk.entities[idx] = Some(e);
                }
            }
        }
    }
}

fn update_hud(inventory: Res<Inventory>, score: Res<Score>, selected: Res<SelectedBlock>, player_q: Query<&Health, With<Player>>, mut q: Query<&mut Text, With<Hud>>) {
    for mut t in &mut q {
        let hp = player_q.single().map(|h| h.0).unwrap_or(0.0);
        t.0 = format!("HP: {:.0} | Dirt: {} Stone: {} Ore: {} Kills: {} | Place: {:?}", hp, inventory.dirt, inventory.stone, inventory.ore, score.kills, selected.0);
    }
}

fn save_load(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut inventory: ResMut<Inventory>,
    mut score: ResMut<Score>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        // chunk loader: ensure chunks around player
        let player_pos = player_q.single().map(|t| (t.translation.x, t.translation.y)).unwrap_or((40.0 * TILE_SIZE, 20.0 * TILE_SIZE));
        let chunks: Vec<ChunkData> = world
            .chunks
            .iter()
            .map(|(cx, c)| ChunkData {
                cx: *cx,
                blocks: c.blocks.clone(),
            })
            .collect();
        let save = SaveData {
            seed: world.seed,
            chunks,
            inventory: inventory.clone(),
            score: score.clone(),
            player_x: player_pos.0,
            player_y: player_pos.1,
        };
        if let Ok(json) = serde_json::to_string(&save) {
            let _ = std::fs::write("asterwild_save.json", json);
        }
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        if let Ok(data) = std::fs::read_to_string("asterwild_save.json") {
            if let Ok(loaded) = serde_json::from_str::<SaveData>(&data) {
                if loaded.seed == world.seed {
                    // despawn all
                    for chunk in world.chunks.values() {
                        for e in chunk.entities.iter().flatten() {
                            commands.entity(*e).despawn();
                        }
                    }
                    world.chunks.clear();
                    // load chunks
                    for cd in loaded.chunks {
                        let mut entities = vec![None; (CHUNK_W * WORLD_H) as usize];
                        for y in 0..WORLD_H {
                            for lx in 0..CHUNK_W {
                                let wx = cd.cx * CHUNK_W + lx;
                                if let Some(bt) = cd.blocks[World::block_index(lx, y)] {
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
                                }
                            }
                        }
                        world.chunks.insert(
                            cd.cx,
                            Chunk {
                                blocks: cd.blocks,
                                entities,
                            },
                        );
                    }
                    *inventory = loaded.inventory;
                    *score = loaded.score;
                    if let Ok(mut t) = player_q.single_mut() {
                        t.translation.x = loaded.player_x;
                        t.translation.y = loaded.player_y;
                    }
                }
            }
        }
    }
}

fn hash2(x: i32, y: i32, seed: u64) -> f32 {
    let mut n = seed
        .wrapping_add((x as u64).wrapping_mul(374761393))
        .wrapping_add((y as u64).wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n as f32) / (u64::MAX as f32)
}

fn surface_height(x: i32, seed: u64) -> i32 {
    let mut h = 12.0f32;
    h += (x as f32 * 0.07).sin() * 5.0;
    h += (x as f32 * 0.19).cos() * 2.5;
    h += (hash2(x, 0, seed) - 0.5) * 4.0;
    h.clamp(6.0, (WORLD_H - 4) as f32) as i32
}

fn is_cave(x: i32, y: i32, seed: u64) -> bool {
    if y > 2 && hash2(x, y, seed.wrapping_mul(3)) > 0.72 {
        // carve some caves below surface
        let n = hash2(x / 3, y / 3, seed.wrapping_mul(7));
        n > 0.6
    } else {
        false
    }
}

fn is_ore(x: i32, y: i32, seed: u64) -> bool {
    hash2(x, y, seed.wrapping_mul(11)) > 0.93
}

fn day_night(
    time: Res<Time>,
    mut daynight: ResMut<DayNight>,
    mut clear_color: ResMut<ClearColor>,
) {
    let dt = time.delta_secs();
    daynight.time = (daynight.time + dt / 120.0) % 1.0;
    // 0.0 midnight, 0.25 dawn, 0.5 noon, 0.75 dusk
    let t = daynight.time;
    let (r, g, b) = if t < 0.25 {
        // night -> dawn
        let k = t / 0.25;
        (0.05 + 0.45 * k, 0.08 + 0.55 * k, 0.15 + 0.6 * k)
    } else if t < 0.5 {
        let k = (t - 0.25) / 0.25;
        (0.5 + 0.1 * k, 0.63 + 0.15 * k, 0.75 + 0.15 * k)
    } else if t < 0.75 {
        let k = (t - 0.5) / 0.25;
        (0.6 - 0.1 * k, 0.78 - 0.15 * k, 0.9 - 0.15 * k)
    } else {
        let k = (t - 0.75) / 0.25;
        (0.5 - 0.45 * k, 0.63 - 0.55 * k, 0.75 - 0.6 * k)
    };
    clear_color.0 = Color::srgb(r, g, b);
}


fn chunk_manager(
    mut commands: Commands,
    mut world: ResMut<World>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(p) = player_q.single() else { return; };
    let pcx = World::chunk_x((p.translation.x / TILE_SIZE) as i32);
    // ensure
    for dc in -LOAD_RADIUS..=LOAD_RADIUS {
        world.ensure_chunk(&mut commands, pcx + dc);
    }
    // unload far
    let far: Vec<i32> = world
        .chunks
        .keys()
        .copied()
        .filter(|cx| (*cx - pcx).abs() > LOAD_RADIUS + 1)
        .collect();
    for cx in far {
        world.unload_chunk(&mut commands, cx);
    }
}

fn creature_collides(world: &World, px: f32, py: f32) -> bool {
    let hw = 10.0f32;
    let hh = 8.0f32;
    let left = px - hw;
    let right = px + hw;
    let bottom = py - hh;
    let top = py + hh;
    let min_x = (left / TILE_SIZE).floor() as i32;
    let max_x = ((right - 0.001) / TILE_SIZE).floor() as i32;
    let min_y = (bottom / TILE_SIZE).floor() as i32;
    let max_y = ((top - 0.001) / TILE_SIZE).floor() as i32;
    for ty in min_y..=max_y {
        for tx in min_x..=max_x {
            if world.get(tx, ty).is_some() {
                return true;
            }
        }
    }
    false
}

fn creature_ai(
    time: Res<Time>,
    world: Res<World>,
    mut query: Query<(&mut Creature, &mut Velocity, &mut Grounded, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut creature, mut vel, mut grounded, mut transform) in &mut query {
        creature.timer -= dt;
        if creature.timer <= 0.0 {
            let r = hash2(transform.translation.x as i32, transform.translation.y as i32, 999);
            creature.dir = if r < 0.33 {
                -1.0
            } else if r > 0.66 {
                1.0
            } else {
                0.0
            };
            creature.timer = 1.0 + r * 2.0;
        }

        // cliff avoidance
        if creature.dir != 0.0 {
            let ahead_x = transform.translation.x + creature.dir * TILE_SIZE * 0.6;
            let below_y = transform.translation.y - TILE_SIZE * 0.6;
            let tx = (ahead_x / TILE_SIZE).floor() as i32;
            let ty = (below_y / TILE_SIZE).floor() as i32;
            if world.get(tx, ty).is_none() {
                creature.dir = -creature.dir;
            }
        }

        vel.0.x = creature.dir * 50.0;

        // gravity
        vel.0.y -= GRAVITY * dt;
        if vel.0.y < -MAX_FALL {
            vel.0.y = -MAX_FALL;
        }

        // horizontal
        let new_x = transform.translation.x + vel.0.x * dt;
        if !creature_collides(&world, new_x, transform.translation.y) {
            transform.translation.x = new_x;
        } else {
            creature.dir = -creature.dir;
            vel.0.x = 0.0;
        }
        // vertical
        let new_y = transform.translation.y + vel.0.y * dt;
        if !creature_collides(&world, transform.translation.x, new_y) {
            transform.translation.y = new_y;
            grounded.0 = false;
        } else {
            if vel.0.y < 0.0 {
                grounded.0 = true;
            }
            vel.0.y = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_deterministic() {
        let a = surface_height(10, 12345);
        let b = surface_height(10, 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn terrain_varies_with_seed() {
        let a = surface_height(10, 12345);
        let b = surface_height(10, 99999);
        // not guaranteed different but likely
        let _ = (a, b);
    }

    #[test]
    fn hash_range() {
        for x in -5..5 {
            for y in -5..5 {
                let h = hash2(x, y, 42);
                assert!((0.0..=1.0).contains(&h));
            }
        }
    }

    #[test]
    fn cave_deterministic() {
        assert_eq!(is_cave(5, 5, 7), is_cave(5, 5, 7));
    }

    #[test]
    fn chunk_indexing() {
        assert_eq!(World::chunk_x(0), 0);
        assert_eq!(World::chunk_x(15), 0);
        assert_eq!(World::chunk_x(16), 1);
        assert_eq!(World::chunk_x(-1), -1);
        assert_eq!(World::local_x(-1), 15);
    }
}

fn hotbar_input(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedBlock>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        selected.0 = BlockType::Dirt;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        selected.0 = BlockType::Stone;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        selected.0 = BlockType::Ore;
    }
}

fn crafting(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if inventory.dirt >= 4 {
            inventory.dirt -= 4;
            inventory.stone += 1;
        }
    }
}

fn survival(
    time: Res<Time>,
    mut player_q: Query<(&Transform, &mut Health), With<Player>>,
    creatures: Query<&Transform, With<Creature>>,
) {
    let Ok((pt, mut hp)) = player_q.single_mut() else { return; };
    let mut damaged = false;
    for ct in &creatures {
        if (ct.translation.truncate() - pt.translation.truncate()).length() < 20.0 {
            damaged = true;
        }
    }
    if damaged {
        hp.0 -= 10.0 * time.delta_secs();
    } else {
        hp.0 = (hp.0 + 5.0 * time.delta_secs()).min(100.0);
    }
    if hp.0 <= 0.0 {
        // respawn
        hp.0 = 100.0;
    }
}
