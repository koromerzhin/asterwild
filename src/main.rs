mod blocks;
mod components;
mod resources;
mod save;
mod setup;
mod systems;
#[cfg(test)]
mod tests;
mod world;

use bevy::prelude::*;
use resources::DayNight;
use components::GameState;
use resources::*;
use resources::AudioStream;

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
        .insert_resource(world::World {
            seed: 12345,
            chunks: std::collections::HashMap::new(),
        })
        .insert_resource(Inventory::default())
        .insert_resource(Score::default())
        .insert_resource(SelectedBlock::default())
        .insert_resource(AttackCooldown::default())
        .insert_resource(HelpVisible::default())
        .insert_resource(WeaponLevel::default())
        .insert_resource(Shield::default())
        .insert_resource(Jetpack::default())
        .insert_resource(Paused::default())
        .insert_resource(WeatherTimer::default())
        .insert_resource(DayNight::default())
        .insert_resource(AutoSaveTimer::default())
        .insert_resource(SaveSlot::default())
        .insert_resource(SplashTimer::default())
        .init_state::<GameState>()
        .insert_resource(AudioStream::default())
        .add_systems(Startup, (setup::setup, systems::spawn_help, systems::init_audio))
        .add_systems(Update, systems::splash_update.run_if(in_state(GameState::Splash)))
        .add_systems(Update, systems::menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(Update, (systems::chunk_manager, systems::player_input, systems::physics, systems::camera_follow, systems::mouse_interact).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (systems::save_load, systems::day_night, systems::creature_ai, systems::hotbar_input, systems::crafting).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (systems::survival, systems::attack, systems::upgrade_weapon, systems::toggle_help, systems::update_hud).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (systems::starfield_update, systems::night_overlay, systems::particle_update, systems::toggle_door, systems::toggle_shield, systems::water_flow, systems::weather, systems::toggle_inventory, systems::update_inventory_ui, systems::smelt, systems::toggle_jetpack, systems::toggle_pause, systems::apply_pause, systems::update_minimap).run_if(in_state(GameState::Playing)))
        .run();
}
