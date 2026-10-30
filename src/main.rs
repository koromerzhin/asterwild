use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const TILE_SIZE: f32 = 32.0;
const WORLD_W: i32 = 80;
const WORLD_H: i32 = 32;
const GRAVITY: f32 = 900.0;
const MOVE_SPEED: f32 = 180.0;
const JUMP_SPEED: f32 = 360.0;
const MAX_FALL: f32 = 600.0;
const PLAYER_HALF_W: f32 = 10.0;
const PLAYER_HALF_H: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum BlockType {
    Dirt,
    Stone,
    Ore,
}

impl BlockType {
    fn color(self) -> Color {
        match self {
            BlockType::Dirt => Color::srgb(0.45, 0.30, 0.18),
            BlockType::Stone => Color::srgb(0.50, 0.50, 0.55),
            BlockType::Ore => Color::srgb(0.75, 0.60, 0.20),
        }
    }
}

#[derive(Resource, Serialize, Deserialize)]
struct WorldGrid {
    width: i32,
    height: i32,
    blocks: Vec<Option<BlockType>>,
    #[serde(skip, default)]
    entities: Vec<Option<Entity>>,
}

impl WorldGrid {
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some((y as usize) * (self.width as usize) + (x as usize))
    }
    fn get(&self, x: i32, y: i32) -> Option<BlockType> {
        self.idx(x, y).and_then(|i| self.blocks[i])
    }
}

#[derive(Resource, Default)]
struct Inventory {
    dirt: u32,
    stone: u32,
    ore: u32,
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
        .insert_resource(Inventory::default())
        .insert_resource(DayNight::default())
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_input,
                physics,
                camera_follow,
                mouse_interact,
                save_load,
                day_night,
                creature_ai,
                update_hud,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let seed = 12345u64;

    // Generate terrain data from seed
    let mut blocks = vec![None; (WORLD_W * WORLD_H) as usize];
    for x in 0..WORLD_W {
        let surface = surface_height(x, seed);
        for y in 0..surface {
            if is_cave(x, y, seed) {
                continue;
            }
            let bt = if y < surface - 4 {
                if is_ore(x, y, seed) {
                    BlockType::Ore
                } else {
                    BlockType::Stone
                }
            } else {
                BlockType::Dirt
            };
            let idx = (y as usize) * (WORLD_W as usize) + (x as usize);
            blocks[idx] = Some(bt);
        }
    }

    let mut grid = WorldGrid {
        width: WORLD_W,
        height: WORLD_H,
        blocks,
        entities: vec![None; (WORLD_W * WORLD_H) as usize],
    };

    // Attempt to load saved modifications
    if let Ok(data) = std::fs::read_to_string("asterwild_save.json") {
        if let Ok(loaded) = serde_json::from_str::<WorldGrid>(&data) {
            if loaded.width == WORLD_W && loaded.height == WORLD_H {
                grid.blocks = loaded.blocks;
            }
        }
    }

    // Spawn block sprites
    for y in 0..WORLD_H {
        for x in 0..WORLD_W {
            if let Some(bt) = grid.get(x, y) {
                let e = commands
                    .spawn((
                        Sprite {
                            color: bt.color(),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(
                            x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            0.0,
                        ),
                    ))
                    .id();
                if let Some(i) = grid.idx(x, y) {
                    grid.entities[i] = Some(e);
                }
            }
        }
    }

    commands.insert_resource(grid);

    // Player
    commands.spawn((
        Player,
        Velocity(Vec2::ZERO),
        Grounded(false),
        Sprite {
            color: Color::srgb(0.2, 0.5, 0.9),
            custom_size: Some(Vec2::new(PLAYER_HALF_W * 2.0, PLAYER_HALF_H * 2.0)),
            ..default()
        },
        Transform::from_xyz(40.0 * TILE_SIZE, 20.0 * TILE_SIZE, 1.0),
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
            font_size: FontSize::Px(18.0),
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

fn collides(grid: &WorldGrid, px: f32, py: f32) -> bool {
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
            if grid.get(tx, ty).is_some() {
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
    grid: Res<WorldGrid>,
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
        if !collides(&grid, new_x, transform.translation.y) {
            transform.translation.x = new_x;
        } else {
            vel.0.x = 0.0;
        }
        // vertical
        let new_y = transform.translation.y + vel.0.y * dt;
        if !collides(&grid, transform.translation.x, new_y) {
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
    mut grid: ResMut<WorldGrid>,
    mut inventory: ResMut<Inventory>,
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
        // break
        if let Some(bt) = grid.get(tx, ty) {
            let idx = grid.idx(tx, ty).unwrap();
            if let Some(e) = grid.entities[idx].take() {
                commands.entity(e).despawn();
            }
            grid.blocks[idx] = None;
            match bt {
                BlockType::Dirt => inventory.dirt += 1,
                BlockType::Stone => inventory.stone += 1,
                BlockType::Ore => inventory.ore += 1,
            }
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        // place dirt if empty
        if grid.get(tx, ty).is_none() {
            if inventory.dirt > 0 {
                inventory.dirt -= 1;
                let bt = BlockType::Dirt;
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
                let idx = grid.idx(tx, ty).unwrap();
                grid.blocks[idx] = Some(bt);
                grid.entities[idx] = Some(e);
            }
        }
    }
}

fn update_hud(inventory: Res<Inventory>, mut q: Query<&mut Text, With<Hud>>) {
    for mut t in &mut q {
        t.0 = format!("Dirt: {}  Stone: {}  Ore: {}", inventory.dirt, inventory.stone, inventory.ore);
    }
}

fn save_load(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        if let Ok(json) = serde_json::to_string(&*grid) {
            let _ = std::fs::write("asterwild_save.json", json);
        }
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        if let Ok(data) = std::fs::read_to_string("asterwild_save.json") {
            if let Ok(loaded) = serde_json::from_str::<WorldGrid>(&data) {
                if loaded.width == grid.width && loaded.height == grid.height {
                    // despawn existing
                    for e in grid.entities.iter().flatten() {
                        commands.entity(*e).despawn();
                    }
                    grid.blocks = loaded.blocks;
                    grid.entities = vec![None; (grid.width * grid.height) as usize];
                    // respawn
                    for y in 0..grid.height {
                        for x in 0..grid.width {
                            if let Some(bt) = grid.get(x, y) {
                                let e = commands
                                    .spawn((
                                        Sprite {
                                            color: bt.color(),
                                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                                            ..default()
                                        },
                                        Transform::from_xyz(
                                            x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                            y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                            0.0,
                                        ),
                                    ))
                                    .id();
                                if let Some(i) = grid.idx(x, y) {
                                    grid.entities[i] = Some(e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn hash2(x: i32, y: i32, seed: u64) -> f32 {
    let mut n = seed
        .wrapping_add(x as u64 * 374761393)
        .wrapping_add(y as u64 * 668265263);
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

fn creature_collides(grid: &WorldGrid, px: f32, py: f32) -> bool {
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
            if grid.get(tx, ty).is_some() {
                return true;
            }
        }
    }
    false
}

fn creature_ai(
    time: Res<Time>,
    grid: Res<WorldGrid>,
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
            if grid.get(tx, ty).is_none() {
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
        if !creature_collides(&grid, new_x, transform.translation.y) {
            transform.translation.x = new_x;
        } else {
            creature.dir = -creature.dir;
            vel.0.x = 0.0;
        }
        // vertical
        let new_y = transform.translation.y + vel.0.y * dt;
        if !creature_collides(&grid, transform.translation.x, new_y) {
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
