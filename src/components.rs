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


#[derive(Component)]
pub struct Star {
    pub base: Vec2,
    pub phase: f32,
}


#[derive(Component)]
pub struct NightOverlay;


#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub max_lifetime: f32,
}


#[derive(Component)]
pub struct InventoryUi;


#[derive(Component)]
pub struct JumpsLeft(pub u32);


#[derive(Component)]
pub struct PauseUi;


#[derive(Component)]
pub struct Minimap;

#[derive(Component)]
pub struct MinimapCell {
    pub dx: i32,
    pub dy: i32,
}


#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
    Playing,
}


#[derive(Component)]
pub struct MenuUi;


#[derive(Component)]
pub struct SplashUi;
