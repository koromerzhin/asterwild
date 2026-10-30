# Decision 0001 — Engine: Bevy

## Problem

Choose a game engine for Asterwild (2D space exploration/building). Candidate: Bevy.

## Options studied

- Bevy 0.20 (Rust, ECS, wgpu).
- (Alternative) Macroquad, ggez, custom engine — not pursued yet.

## Decision

Retain Bevy 0.20 as primary engine, subject to Phase 1 prototype validation.

## Reasons

- Native Rust, matches project language.
- ECS fits entity-heavy world.
- 2D rendering, sprites, UI, audio, asset loading built-in.
- Builds and runs on Fedora 44 (Vulkan present, ALSA/X11/Wayland libs present).
- Cached crates available offline.
- Active ecosystem.

## Consequences / limits

- Compile times significant (~25 min cold) on 8 GB RAM machine.
- Bevy API changes between versions; pin version.
- Must validate block terrain, chunks, procedural gen, save/load in prototype.
- If Bevy fails requirements, evaluate alternative before committing deeper.
EOF