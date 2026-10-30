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
use resources::*;

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
        .insert_resource(DayNight::default())
        .add_systems(Startup, (setup::setup, systems::spawn_help))
        .add_systems(
            Update,
            (
                systems::chunk_manager,
                systems::player_input,
                systems::physics,
                systems::camera_follow,
                systems::mouse_interact,
                systems::save_load,
                systems::day_night,
                systems::creature_ai,
                systems::hotbar_input,
                systems::crafting,
                systems::survival,
                systems::attack,
                systems::upgrade_weapon,
                systems::toggle_help,
                systems::update_hud,
                systems::starfield_update,
                systems::night_overlay,
                systems::particle_update,
                systems::water_flow,
            )
                .chain(),
        )
        .run();
}
