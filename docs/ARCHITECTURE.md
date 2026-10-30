# Architecture — Asterwild

## Engine

Bevy 0.20 (validated: builds on Fedora 44, Vulkan present, winit/wayland/x11/alsa available).

## Principles

- Separate rendering/engine from gameplay, procedural generation, world simulation, entities,
  items/inventory, progression, interactions, saves, game data, UI.
- Avoid circular dependencies.
- Avoid unnecessary abstractions; each module has a real responsibility.
- ECS via Bevy; components/systems/resources.
- Stable entity IDs for important entities (future multiplayer).
- Serializable world state.
- Gameplay rules independent of UI.

## Planned crates/modules (minimal first)

- `asterwild` binary: app bootstrap, plugins.
- (later) `asterwild_core`, `asterwild_world`, `asterwild_procgen`, etc. Only split when justified.

## Save format

Versioned. Never overwrite a valid save without safety mechanism. Migrations for old saves.

## Current module layout

```
src/
  main.rs   # app bootstrap + minimal prototype systems
```
EOF