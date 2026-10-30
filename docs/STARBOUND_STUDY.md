# Starbound study (public sources only)

Studied from the locally installed Starbound (read-only) and its public Lua API documentation.
No reverse engineering, no binary analysis, no protected assets copied.

## Observable systems

### World & terrain
- Worlds are finite in height, wrap horizontally (world.xwrap, world.magnitude handles wrapping).
- Tile grid with foreground and background layers.
- Materials and mods can be placed on tiles; mods overlay materials.
- Tiles can be damaged by types: plantish, blockish, beamish, explosive, fire, tilling; harvest level gates drops.
- Protected tiles exist (world.isTileProtected).
- Liquids exist as quantities per tile (spawn/destroy liquid), with regional oceans.
- Gravity varies by position (dungeons can alter).
- Surface level, underground flag, surface layer, ocean level.
- Lighting and wind per position; breathable flag; environmental status effects.
- Day/night cycle with world.timeOfDay, dayLength, absolute day.

### Entities
- Entity types: mobile, creature, monster, NPC, object, item drop, projectile, vehicle, stagehand.
- Queries by rectangle/circle, with filters (withoutEntityId, includedTypes, name for objects).
- Spawn functions for monsters, NPCs, objects, items, projectiles, vehicles, stagehands, treasure pools.
- Pathfinding (platformer A*) available to scripts.

### Player
- Unique id, species, gender, admin.
- Generic properties.
- Ship upgrades: shipLevel, maxFuel, crewSize, capabilities.
- Universe flags.
- Scanned objects.

### Progression / exploration
- Threat level per world.
- Treasure pools with level and seed.
- Monster level and aggression parameters.
- NPC species/type/level.

## What makes it interesting
- Layered tiles (FG/BG) give depth and building flexibility.
- Liquid quantities and oceans create exploration variety.
- World wrapping enables seamless horizontal exploration.
- Procedural spawns tied to threat level and seed.

## Limits / opportunities for Asterwild
- Tile damage types could be simplified initially; Asterwild can start with one break tool.
- World wrapping adds complexity; Asterwild can start with finite worlds, consider wrapping later.
- Liquid simulation is heavy; defer fluids to later phase.
- Asterwild should have more distinct planet identity and non-linear progression.
- Asterwild art direction distinct (stylized 2D, not pixel art).

## Asterwild implications
- Start with FG layer only (BG later).
- Deterministic seed-based generation.
- Separate base generation from player modifications.
- Stable entity IDs for future multiplayer.
EOF