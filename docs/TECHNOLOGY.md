# Technology — Asterwild

## Toolchain

- OS: Fedora Linux 44 (KDE Plasma)
- Rust: 1.99.1 (stable, Fedora package)
- Cargo: 1.99.1
- GCC/G++: 16.2.1
- Git: 2.56.0
- GitHub CLI: 2.97.0 (authenticated as koromerzhin)
- OpenCode: 2.2.2

## Engine candidate

Bevy 0.20.0 (cached in cargo registry). Validated:
- Builds successfully on this machine.
- Vulkan 1.4 instance available.
- ALSA, X11, Wayland libraries present.
- winit, wayland, x11, audio crates compile.

## System libraries present

- libasound (ALSA)
- libX11, libX11-xcb
- libwayland-client/cursor/egl/server
- Vulkan (instance 1.4.341)

## Constraints

- 8 GB RAM. Dev profile: opt-level=1 for crate, opt-level=3 for dependencies (already set).
- Avoid heavy local AI models.
- Avoid launching multiple builds simultaneously.

## Decisions

- Bevy retained as primary engine pending prototype (Phase 1).
- No REA. No reverse engineering of Starbound.
EOF