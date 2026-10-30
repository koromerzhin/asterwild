use crate::blocks::{self, BlockType};
use crate::components::*;
use crate::components::SplashUi;
use crate::components::MenuUi;
use crate::components::GameState;
use crate::components::MinimapCell;
use crate::resources::DayNight;
use crate::resources::*;
use crate::save::SaveData;
use crate::world::{
    CHUNK_W, Chunk, ChunkData, GRAVITY, JUMP_SPEED, LOAD_RADIUS, MAX_FALL, MOVE_SPEED,
    PLAYER_HALF_H, PLAYER_HALF_W, TILE_SIZE, WORLD_H, World,
};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

pub fn collides(world: &World, px: f32, py: f32) -> bool {
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
            if world.is_solid(tx, ty) {
                return true;
            }
        }
    }
    false
}

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Grounded, &mut Facing), With<Player>>,
) {
    for (mut vel, _, mut facing) in &mut query {
        let mut vx = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            vx -= MOVE_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            vx += MOVE_SPEED;
        }
        vel.0.x = vx;
        if vx < 0.0 {
            facing.0 = -1.0;
        } else if vx > 0.0 {
            facing.0 = 1.0;
        }
    }
}

pub fn physics(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    world: Res<World>,
    jetpack: Res<Jetpack>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Grounded, &mut JumpsLeft), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut vel, mut grounded, mut jumps) in &mut query {
        // ladder check
        let on_ladder = {
            let cx = (transform.translation.x / TILE_SIZE).floor() as i32;
            let cy = (transform.translation.y / TILE_SIZE).floor() as i32;
            world.get(cx, cy) == Some(BlockType::Ladder)
                || world.get(cx, cy + 1) == Some(BlockType::Ladder)
        };
        if on_ladder {
            let mut vy = 0.0;
            if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
                vy += 120.0;
            }
            if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
                vy -= 120.0;
            }
            vel.0.y = vy;
        } else {
            // gravity
            vel.0.y -= GRAVITY * dt;
        }
        if !on_ladder && vel.0.y < -MAX_FALL {
            vel.0.y = -MAX_FALL;
        }
        // jump
        let jump_pressed = keyboard.just_pressed(KeyCode::Space)
            || keyboard.just_pressed(KeyCode::KeyW)
            || keyboard.just_pressed(KeyCode::ArrowUp);
        if jump_pressed {
            if grounded.0 {
                vel.0.y = JUMP_SPEED;
                grounded.0 = false;
                jumps.0 = if jetpack.0 { 1 } else { 0 };
            } else if jetpack.0 && jumps.0 > 0 {
                vel.0.y = JUMP_SPEED;
                jumps.0 -= 1;
            }
        }
        if grounded.0 {
            jumps.0 = if jetpack.0 { 1 } else { 0 };
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

pub fn camera_follow(
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

pub fn mouse_interact(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut inventory: ResMut<Inventory>,
    mut score: ResMut<Score>,
    selected: Res<SelectedBlock>,
    audio: Res<AudioStream>,
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
                if let Some(g) = chunk.glows[idx].take() {
                    commands.entity(g).despawn();
                }
                if let Some(sink) = &audio.sink {
                    play_beep(sink, 440.0, 0.08);
                }
                chunk.blocks[idx] = None;
                match bt {
                    BlockType::Dirt => inventory.dirt += 1,
                    BlockType::Stone => inventory.stone += 1,
                    BlockType::Ore => inventory.ore += 1,
                    BlockType::Sand => inventory.dirt += 1,
                    BlockType::Snow => inventory.dirt += 1,
                    BlockType::Torch => inventory.dirt += 1,
                    BlockType::Ladder => inventory.dirt += 1,
                    BlockType::Water => {}
                    BlockType::Door | BlockType::DoorOpen => inventory.dirt += 1,
                    BlockType::Furnace => inventory.dirt += 1,
                }
                let px = tx as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let py = ty as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let col = bt.color();
                for k in 0..6 {
                    let vx = (blocks::hash2(k, ty, 123) - 0.5) * 120.0;
                    let vy = (blocks::hash2(tx, k, 456) * 80.0) + 40.0;
                    commands.spawn((
                        Particle { lifetime: 0.6, max_lifetime: 0.6 },
                        Sprite {
                            color: col,
                            custom_size: Some(Vec2::splat(4.0)),
                            ..default()
                        },
                        Velocity(Vec2::new(vx, vy)),
                        Transform::from_xyz(px, py, 0.0),
                    ));
                }
            }
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        // place selected if empty
        if world.get(tx, ty).is_none() {
            let bt = selected.0;
            let have = match bt {
                BlockType::Dirt | BlockType::Sand | BlockType::Snow | BlockType::Torch | BlockType::Ladder | BlockType::Water | BlockType::Door | BlockType::DoorOpen | BlockType::Furnace => {
                    &mut inventory.dirt
                }
                BlockType::Stone => &mut inventory.stone,
                BlockType::Ore => &mut inventory.ore,
            };
            let can_place = if bt == BlockType::Water { true } else { *have > 0 };
            if can_place {
                if bt != BlockType::Water {
                    *have -= 1;
                }
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
                let mut glow_ent = None;
                if bt == BlockType::Torch {
                    let g = commands
                        .spawn((
                            Sprite {
                                color: Color::srgba(1.0, 0.8, 0.3, 0.35),
                                custom_size: Some(Vec2::splat(TILE_SIZE * 3.0)),
                                ..default()
                            },
                            Transform::from_xyz(
                                tx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                ty as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                0.5,
                            ),
                        ))
                        .id();
                    glow_ent = Some(g);
                }
                if let Some(chunk) = world.chunks.get_mut(&cx) {
                    let lx = World::local_x(tx);
                    let idx = World::block_index(lx, ty);
                    chunk.blocks[idx] = Some(bt);
                    chunk.entities[idx] = Some(e);
                    chunk.glows[idx] = glow_ent;
                }
            }
        }
    }
}

pub fn update_hud(
    inventory: Res<Inventory>,
    score: Res<Score>,
    selected: Res<SelectedBlock>,
    weapon: Res<WeaponLevel>,
    shield: Res<Shield>,
    jetpack: Res<Jetpack>,
    slot: Res<SaveSlot>,
    world: Res<World>,
    player_q: Query<(&Health, &Transform), With<Player>>,
    mut q: Query<&mut Text, With<Hud>>,
) {
    for mut t in &mut q {
        let (hp, biome, px, py) = player_q
            .single()
            .map(|(h, tr)| (h.0, blocks::biome_at((tr.translation.x / TILE_SIZE) as i32, world.seed), tr.translation.x, tr.translation.y))
            .unwrap_or((0.0, blocks::Biome::Forest, 0.0, 0.0));
        let bname = match biome {
            blocks::Biome::Forest => "Forest",
            blocks::Biome::Desert => "Desert",
            blocks::Biome::Snow => "Snow",
        };
        t.0 = format!(
            "HP: {:.0} Sh: {} Jp: {} | Day: {} Deaths: {} Biome: {} | Pos: {:.0},{:.0} | Dirt: {} Stone: {} Ore: {} Ingot: {} Kills: {} Wpn: {} Slot: {} | Place: {:?}",
            hp,
            if shield.0 { "Y" } else { "N" },
            if jetpack.0 { "Y" } else { "N" },
            score.days, score.deaths, bname,
            px, py,
            inventory.dirt,
            inventory.stone,
            inventory.ore,
            inventory.ingot,
            score.kills,
            weapon.0,
            slot.0,
            selected.0
        );
    }
}


pub fn save_load(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut inventory: ResMut<Inventory>,
    mut score: ResMut<Score>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut autosave: ResMut<AutoSaveTimer>,
    slot: Res<SaveSlot>,
) {
    autosave.timer -= time.delta_secs();
    let auto = autosave.timer <= 0.0;
    if auto {
        autosave.timer = 60.0;
    }
    if keyboard.just_pressed(KeyCode::KeyS) || auto {
        // chunk loader: ensure chunks around player
        let player_pos = player_q
            .single()
            .map(|t| (t.translation.x, t.translation.y))
            .unwrap_or((40.0 * TILE_SIZE, 20.0 * TILE_SIZE));
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
            let _ = std::fs::write(format!("asterwild_save_{}.json", slot.0), json);
        }
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        if let Ok(data) = std::fs::read_to_string(format!("asterwild_save_{}.json", slot.0)) {
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

pub fn day_night(
    time: Res<Time>,
    mut daynight: ResMut<DayNight>,
    mut clear_color: ResMut<ClearColor>,
    mut score: ResMut<Score>,
) {
    let dt = time.delta_secs();
    let prev = daynight.time;
    daynight.time = (daynight.time + dt / 120.0) % 1.0;
    if daynight.time < prev {
        score.days += 1;
    }
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

pub fn chunk_manager(
    mut commands: Commands,
    mut world: ResMut<World>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(p) = player_q.single() else {
        return;
    };
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

pub fn creature_collides(world: &World, px: f32, py: f32) -> bool {
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
            if world.is_solid(tx, ty) {
                return true;
            }
        }
    }
    false
}

pub fn creature_ai(
    time: Res<Time>,
    world: Res<World>,
    mut query: Query<(&mut Creature, &mut Velocity, &mut Grounded, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut creature, mut vel, mut grounded, mut transform) in &mut query {
        creature.timer -= dt;
        if creature.timer <= 0.0 {
            let r = blocks::hash2(
                transform.translation.x as i32,
                transform.translation.y as i32,
                999,
            );
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

pub fn attack(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cooldown: ResMut<AttackCooldown>,
    player_q: Query<(&Transform, &Facing), With<Player>>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    weapon: Res<WeaponLevel>,
    creatures: Query<(Entity, &Transform), With<Creature>>,
) {
    cooldown.timer = (cooldown.timer - time.delta_secs()).max(0.0);
    if keyboard.just_pressed(KeyCode::KeyF) && cooldown.timer <= 0.0 {
        if let Ok((pt, facing)) = player_q.single() {
            let pp = pt.translation.truncate();
            for (e, t) in &creatures {
                let d = t.translation.truncate() - pp;
                let range = 48.0 + weapon.0 as f32 * 16.0;
                if d.length() < range && d.x * facing.0 >= 0.0 {
                    commands.entity(e).despawn();
                    score.kills += 1;
                }
            }
        }
        cooldown.timer = 0.5;
    }
}

pub fn upgrade_weapon(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut weapon: ResMut<WeaponLevel>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) && inventory.ore >= 5 {
        inventory.ore -= 5;
        weapon.0 += 1;
    }
}

pub fn hotbar_input(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedBlock>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        selected.0 = BlockType::Dirt;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        selected.0 = BlockType::Stone;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        selected.0 = BlockType::Ore;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        selected.0 = BlockType::Water;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        selected.0 = BlockType::Furnace;
    }
}

pub fn crafting(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if inventory.dirt >= 4 {
            inventory.dirt -= 4;
            inventory.stone += 1;
        }
    }
}

pub fn survival(
    time: Res<Time>,
    mut player_q: Query<(&mut Transform, &mut Health), With<Player>>,
    creatures: Query<&Transform, (With<Creature>, Without<Player>)>,
    shield: Res<Shield>,
    mut score: ResMut<Score>,
) {
    let Ok((mut pt, mut hp)) = player_q.single_mut() else {
        return;
    };
    let mut damaged = false;
    for ct in &creatures {
        if (ct.translation.truncate() - pt.translation.truncate()).length() < 20.0 {
            damaged = true;
        }
    }
    if damaged {
        let dmg = if shield.0 { 5.0 } else { 10.0 };
        hp.0 -= dmg * time.delta_secs();
    } else {
        hp.0 = (hp.0 + 5.0 * time.delta_secs()).min(100.0);
    }
    if hp.0 <= 0.0 {
        hp.0 = 100.0;
        pt.translation.x = 40.0 * TILE_SIZE;
        pt.translation.y = 20.0 * TILE_SIZE;
        score.deaths += 1;
    }
}

pub fn spawn_help(mut commands: Commands) {
    commands.spawn((
        HelpText,
        Text::new(
            "Controls: A/D move, Space/W jump, Mouse L break/hit, Mouse R place, 1/2/3 select block, C craft, F attack, S save, L load, H toggle help",
        ),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(8),
            left: px(8),
            ..default()
        },
    ));
}

pub fn toggle_help(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut vis: ResMut<HelpVisible>,
    mut q: Query<&mut Visibility, With<HelpText>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        vis.0 = !vis.0;
    }
    for mut v in &mut q {
        *v = if vis.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn starfield_update(time: Res<Time>, camera_q: Query<&Transform, (With<Camera2d>, Without<Star>)>, mut stars: Query<(&Star, &mut Transform, &mut Sprite), Without<Camera2d>>) {
    let cam_pos = camera_q.single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);
    for (star, mut t, mut sprite) in &mut stars {
        let pos = star.base + cam_pos * 0.3;
        t.translation.x = pos.x;
        t.translation.y = pos.y;
        let alpha = 0.5 + 0.5 * (time.elapsed_secs() + star.phase).sin();
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.2, 1.0));
    }
}

pub fn night_overlay(
    daynight: Res<DayNight>,
    camera_q: Query<&Transform, (With<Camera2d>, Without<NightOverlay>)>,
    mut overlay_q: Query<(&mut Sprite, &mut Transform), (With<NightOverlay>, Without<Camera2d>)>,
) {
    let brightness = (std::f32::consts::TAU * (daynight.time - 0.5)).cos();
    let alpha = (0.7 * (1.0 - brightness) / 2.0).clamp(0.0, 0.75);
    for (mut sprite, mut t) in &mut overlay_q {
        sprite.color = Color::srgba(0.0, 0.0, 0.05, alpha);
        if let Ok(ct) = camera_q.single() {
            t.translation.x = ct.translation.x;
            t.translation.y = ct.translation.y;
        }
    }
}

pub fn particle_update(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite, &mut Velocity)>,
) {
    let dt = time.delta_secs();
    for (e, mut p, mut t, mut sprite, mut vel) in &mut query {
        p.lifetime -= dt;
        if p.lifetime <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        vel.0.y -= 400.0 * dt;
        t.translation.x += vel.0.x * dt;
        t.translation.y += vel.0.y * dt;
        let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

pub fn water_flow(
    mut commands: Commands,
    mut world: ResMut<World>,
) {
    for (cx, chunk) in world.chunks.iter_mut() {
        // process bottom to top
        for y in 1..WORLD_H as usize {
            for lx in 0..CHUNK_W as usize {
                let idx = World::block_index(lx as i32, y as i32);
                if chunk.blocks[idx] == Some(BlockType::Water) {
                    let below_idx = World::block_index(lx as i32, y as i32 - 1);
                    if chunk.blocks[below_idx].is_none() {
                        // move down
                        chunk.blocks[below_idx] = Some(BlockType::Water);
                        chunk.blocks[idx] = None;
                        // update entities
                        let wx = *cx * CHUNK_W + lx as i32;
                        if let Some(e) = chunk.entities[idx].take() {
                            commands.entity(e).despawn();
                        }
                        let new_e = commands.spawn((
                            Sprite {
                                color: BlockType::Water.color(),
                                custom_size: Some(Vec2::splat(TILE_SIZE)),
                                ..default()
                            },
                            Transform::from_xyz(
                                wx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                (y as i32 - 1) as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                0.0,
                            ),
                        )).id();
                        chunk.entities[below_idx] = Some(new_e);
                        if let Some(g) = chunk.glows[idx].take() {
                            commands.entity(g).despawn();
                        }
                        chunk.glows[below_idx] = None;
                    }
                }
            }
        }
    }
}

pub fn toggle_door(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<World>,
    player_q: Query<&Transform, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(pt) = player_q.single() else { return; };
    let px = (pt.translation.x / TILE_SIZE).floor() as i32;
    let py = (pt.translation.y / TILE_SIZE).floor() as i32;
    for (dx, dy) in [(1i32,0i32),(-1,0),(0,1),(0,-1)] {
        let x = px + dx;
        let y = py + dy;
        let cx = World::chunk_x(x);
        world.ensure_chunk(&mut commands, cx);
        if let Some(chunk) = world.chunks.get_mut(&cx) {
            let lx = World::local_x(x);
            let idx = World::block_index(lx, y);
            match chunk.blocks[idx] {
                Some(BlockType::Door) => chunk.blocks[idx] = Some(BlockType::DoorOpen),
                Some(BlockType::DoorOpen) => chunk.blocks[idx] = Some(BlockType::Door),
                _ => {}
            }
        }
    }
}


pub fn toggle_shield(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>, mut shield: ResMut<Shield>) {
    if keyboard.just_pressed(KeyCode::KeyR) && !shield.0 && inventory.ore >= 3 {
        inventory.ore -= 3;
        shield.0 = true;
    }
}

pub fn weather(
    time: Res<Time>,
    mut timer: ResMut<WeatherTimer>,
    daynight: Res<DayNight>,
    camera_q: Query<&Transform, (With<Camera2d>, Without<Particle>)>,
    mut commands: Commands,
) {
    // rain at night
    let is_night = daynight.time < 0.25 || daynight.time > 0.75;
    if !is_night {
        return;
    }
    timer.timer -= time.delta_secs();
    if timer.timer > 0.0 {
        return;
    }
    timer.timer = 0.05;
    let Ok(ct) = camera_q.single() else { return; };
    let cam = ct.translation.truncate();
    for i in 0..3 {
        let x = cam.x + (i as f32 - 1.0) * 100.0 + (time.elapsed_secs() * 37.0).sin() * 50.0;
        let y = cam.y + 200.0;
        commands.spawn((
            Particle { lifetime: 1.5, max_lifetime: 1.5 },
            Sprite {
                color: Color::srgba(0.7, 0.8, 1.0, 0.5),
                custom_size: Some(Vec2::new(1.0, 6.0)),
                ..default()
            },
            Velocity(Vec2::new(-20.0, -200.0)),
            Transform::from_xyz(x, y, 15.0),
        ));
    }
}

pub fn toggle_inventory(keyboard: Res<ButtonInput<KeyCode>>, mut q: Query<&mut Visibility, With<InventoryUi>>) {
    // reuse HelpVisible? Better separate. We'll just toggle visibility directly.
    if keyboard.just_pressed(KeyCode::KeyI) {
        for mut v in &mut q {
            *v = match *v {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

pub fn update_inventory_ui(
    inventory: Res<Inventory>,
    score: Res<Score>,
    weapon: Res<WeaponLevel>,
    shield: Res<Shield>,
    mut q: Query<&mut Text, With<InventoryUi>>,
) {
    for mut t in &mut q {
        t.0 = format!(
            "Inventory\nDirt: {}\nStone: {}\nOre: {}\nKills: {}\nWeapon: {}\nShield: {}",
            inventory.dirt, inventory.stone, inventory.ore, score.kills, weapon.0, shield.0
        );
    }
}

pub fn smelt(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if keyboard.just_pressed(KeyCode::KeyG) && inventory.ore >= 2 {
        inventory.ore -= 2;
        inventory.ingot += 1;
    }
}

pub fn toggle_jetpack(keyboard: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>, mut jetpack: ResMut<Jetpack>) {
    if keyboard.just_pressed(KeyCode::KeyJ) && !jetpack.0 && inventory.ingot >= 3 {
        inventory.ingot -= 3;
        jetpack.0 = true;
    }
}

pub fn toggle_pause(keyboard: Res<ButtonInput<KeyCode>>, mut paused: ResMut<Paused>, mut q: Query<&mut Visibility, With<PauseUi>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
    }
    for mut v in &mut q {
        *v = if paused.0 { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub fn apply_pause(paused: Res<Paused>, mut time: ResMut<Time<Virtual>>) {
    if paused.0 {
        time.set_relative_speed(0.0);
    } else {
        time.set_relative_speed(1.0);
    }
}

pub fn update_minimap(
    world: Res<World>,
    player_q: Query<&Transform, With<Player>>,
    mut q: Query<(&MinimapCell, &mut BackgroundColor)>,
) {
    let Ok(pt) = player_q.single() else { return; };
    let px = (pt.translation.x / TILE_SIZE).floor() as i32;
    let py = (pt.translation.y / TILE_SIZE).floor() as i32;
    for (cell, mut bg) in &mut q {
        let x = px + cell.dx;
        let _y = py + cell.dy;
        let biome = blocks::biome_at(x, world.seed);
        let color = match biome {
            blocks::Biome::Forest => Color::srgb(0.2, 0.5, 0.2),
            blocks::Biome::Desert => Color::srgb(0.8, 0.7, 0.4),
            blocks::Biome::Snow => Color::srgb(0.9, 0.9, 1.0),
        };
        // mark player position
        if cell.dx == 0 && cell.dy == 0 {
            bg.0 = Color::srgb(1.0, 0.2, 0.2);
        } else {
            bg.0 = color;
        }
    }
}

pub fn init_audio(mut audio: ResMut<AudioStream>) {
    if let Ok(sink) = rodio::DeviceSinkBuilder::open_default_sink() {
        audio.sink = Some(sink);
    }
}

pub fn play_beep(sink: &rodio::MixerDeviceSink, freq: f32, dur: f32) {
    use rodio::source::{SineWave, Source};
    use std::time::Duration;
    let source = SineWave::new(freq)
        .take_duration(Duration::from_secs_f32(dur))
        .amplify(0.2);
    sink.mixer().add(source);
}

pub fn menu_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>, mut exit: MessageWriter<AppExit>, mut q: Query<&mut Visibility, With<MenuUi>>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
        for mut v in &mut q {
            *v = Visibility::Hidden;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn toggle_fullscreen(keyboard: Res<ButtonInput<KeyCode>>, mut window_q: Query<&mut Window>) {
    if keyboard.just_pressed(KeyCode::F11) {
        for mut w in &mut window_q {
            w.mode = if w.mode == WindowMode::Windowed {
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            } else {
                WindowMode::Windowed
            };
        }
    }
}

pub fn splash_update(time: Res<Time>, mut timer: ResMut<SplashTimer>, mut next_state: ResMut<NextState<GameState>>, mut q: Query<&mut Visibility, With<SplashUi>>) {
    timer.timer -= time.delta_secs();
    if timer.timer <= 0.0 {
        next_state.set(GameState::Menu);
        for mut v in &mut q {
            *v = Visibility::Hidden;
        }
    }
}

pub fn cycle_slot(keyboard: Res<ButtonInput<KeyCode>>, mut slot: ResMut<SaveSlot>) {
    if keyboard.just_pressed(KeyCode::F2) {
        slot.0 = (slot.0 + 1) % 3;
    }
}
