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
}

impl BlockType {
    fn color(self) -> Color {
        match self {
            BlockType::Dirt => Color::srgb(0.45, 0.30, 0.18),
            BlockType::Stone => Color::srgb(0.50, 0.50, 0.55),
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
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Grounded(bool);

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
        .add_systems(Startup, setup)
        .add_systems(Update, (player_input, physics, camera_follow, mouse_interact, save_load, update_hud))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Generate terrain data
    let mut blocks = vec![None; (WORLD_W * WORLD_H) as usize];
    for x in 0..WORLD_W {
        let h = 8 + ((x as f32 * 0.15).sin() * 3.0) as i32 + (x as i32 % 5);
        let surface = h.clamp(4, WORLD_H - 4);
        for y in 0..surface {
            let bt = if y < surface - 3 { BlockType::Stone } else { BlockType::Dirt };
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
                let e = commands.spawn((
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
                )).id();
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

    // HUD
    commands.spawn((
        Hud,
        Text::new("Blocks: 0"),
        TextFont { font_size: FontSize::Px(18.0), ..default() },
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
        if grounded.0 && (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp)) {
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
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((camera, cam_transform)) = camera_q.single() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

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
            }
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        // place dirt if empty
        if grid.get(tx, ty).is_none() {
            if inventory.dirt > 0 {
                inventory.dirt -= 1;
                let bt = BlockType::Dirt;
                let e = commands.spawn((
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
                )).id();
                let idx = grid.idx(tx, ty).unwrap();
                grid.blocks[idx] = Some(bt);
                grid.entities[idx] = Some(e);
            }
        }
    }
}

fn update_hud(
    inventory: Res<Inventory>,
    mut q: Query<&mut Text, With<Hud>>,
) {
    for mut t in &mut q {
        t.0 = format!("Dirt: {}  Stone: {}", inventory.dirt, inventory.stone);
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
                                let e = commands.spawn((
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
                                )).id();
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
