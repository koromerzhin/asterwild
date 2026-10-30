use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Creature {
    pub dir: f32,
    pub timer: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Facing(pub f32);

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct HelpText;
