# FrisKingdom — Project Index

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Vision
FrisKingdom is a full-featured ultimate frisbee simulation built in Rust with the Bevy engine. It aims to be "FIFA but for ultimate frisbee" — combining deep disc physics, stickman-style visual charm, strategic depth, season management, and rollback-based online multiplayer. The game captures the spirit, athleticism, and community of ultimate frisbee while making the sport accessible and fun for gamers.

## Core Pillars
1. **Authentic Disc Flight** — A standalone `frisbee-physics` crate modeling real aerodynamics (lift, drag, precession, Magnus effect) with RK4 integration and optional GPU compute
2. **Stickman Swagger** — Minimalist stickman characters brought to life through procedural IK animation and a confidence/"swag" system
3. **Strategic Depth** — Full WFDF rules, offensive/defensive formations (vert stack, horiz stack, zone, clam), and a 3-layer AI system
4. **Season & Career** — Season mode with roster management, training, and an RPG-lite create-a-player career path
5. **Online Play** — Rollback netcode for smooth online matches with deterministic simulation

## Tech Stack
| Layer | Technology |
|-------|-----------|
| Language | Rust (2021 edition, stable) |
| Engine | Bevy 0.18 |
| GPU Backend | wgpu (via Bevy) |
| Physics | Custom `frisbee-physics` crate |
| Netcode | GGRS (rollback) + Matchbox (WebRTC signaling) |
| Audio | `bevy_kira_audio` + custom procedural synthesis |
| UI | `bevy_egui` (dev/debug) + custom Bevy UI (game menus) |
| Build | Cargo workspace with multiple crates |
| CI/CD | GitHub Actions |

## Cargo Workspace Structure
```
friskingdom/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── frisbee-physics/        # Standalone disc aerodynamics crate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── disc.rs         # DiscState, DiscParams
│   │   │   ├── aerodynamics.rs # Force computations
│   │   │   ├── integration.rs  # RK4 solver
│   │   │   ├── throws.rs       # Throw catalog & initial conditions
│   │   │   └── wind.rs         # WindField trait
│   │   └── Cargo.toml
│   ├── fk-core/                # Shared types, components, events
│   ├── fk-game/                # Main game logic systems
│   ├── fk-ai/                  # AI decision-making
│   ├── fk-render/              # Rendering, camera, VFX
│   ├── fk-audio/               # Audio systems
│   ├── fk-netcode/             # Multiplayer & rollback
│   ├── fk-ui/                  # UI/UX systems
│   ├── fk-season/              # Season & career management
│   ├── fk-tutorial/            # Tutorial & onboarding
│   ├── fk-debug/               # Dev-only debug tools/overlays
│   └── fk-test-utils/          # Shared testing harness utilities
├── assets/                     # Game assets
│   ├── shaders/
│   ├── fonts/
│   ├── textures/
│   └── audio/
├── design-docs/                # This documentation
└── tools/                      # Dev tools, replay viewer, etc.
```

## Glossary
| Term | Definition |
|------|-----------|
| **Backhand** | Standard throw with arm crossing body; most common throw in ultimate |
| **Forehand (Flick)** | Throw with wrist snap, arm stays on same side |
| **Hammer** | Overhead throw that flies inverted; used to break marks |
| **Scoober** | Upside-down forehand released overhead |
| **Thumber** | Overhead throw gripped with thumb on inside of rim |
| **Blade** | Vertical throw with minimal spin axis wobble |
| **Push Pass** | Short-range throw with pushing motion |
| **Chicken Wing** | Unorthodox throw releasing behind the back |
| **Stall Count** | 10-second countdown; thrower must release before "stall 10" |
| **Turnover** | Change of possession (drop, out-of-bounds, stall-out, interception) |
| **Vert Stack** | Offensive formation with cutters stacked vertically downfield |
| **Horiz Stack** | Offensive formation with cutters spread horizontally |
| **Zone Defense** | Non-man defense using walls, wings, and deeps |
| **Clam** | Hybrid defense mixing man and zone principles |
| **Callahan** | Interception in the opponent's end zone for a score |
| **Greatest** | Saving a disc from going out of bounds while airborne |
| **Layout** | Diving bid for the disc |
| **Spirit of the Game** | Self-officiating principle central to ultimate |
| **Force** | Defensive mark directing thrower to one side |
| **Break** | Throw to the side the mark is trying to prevent |
| **Handler** | Player who primarily throws; usually in the backfield |
| **Cutter** | Player who primarily runs routes to get open |
| **Pull** | Opening throw similar to a kickoff |
| **Brick** | When a pull lands out of bounds; offense starts at the brick mark |
| **Swag System** | FrisKingdom's procedural confidence modifier affecting animations and gameplay |
| **RK4** | 4th-order Runge-Kutta numerical integration method |
| **Rollback Netcode** | Multiplayer technique that predicts inputs and corrects on mismatch |

## Cross-Reference Map

| Topic | Primary Doc | Related Docs |
|-------|-----------|-------------|
| Disc aerodynamics | [01-disc-physics](01-disc-physics.md) | [02-wind-field](02-wind-field.md), [04-player-controls](04-player-controls.md) |
| Wind simulation | [02-wind-field](02-wind-field.md) | [01-disc-physics](01-disc-physics.md), [06-field-and-environment](06-field-and-environment.md) |
| Rules & strategy | [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md) | [09-ai-system](09-ai-system.md), [04-player-controls](04-player-controls.md) |
| Player input | [04-player-controls](04-player-controls.md) | [01-disc-physics](01-disc-physics.md), [05-stickman-animation](05-stickman-animation.md) |
| Animation & style | [05-stickman-animation](05-stickman-animation.md) | [07-rendering](07-rendering.md), [04-player-controls](04-player-controls.md) |
| Venues & fields | [06-field-and-environment](06-field-and-environment.md) | [02-wind-field](02-wind-field.md), [07-rendering](07-rendering.md) |
| Rendering pipeline | [07-rendering](07-rendering.md) | [05-stickman-animation](05-stickman-animation.md), [06-field-and-environment](06-field-and-environment.md) |
| Engine architecture | [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md) | All documents |
| AI opponents | [09-ai-system](09-ai-system.md) | [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md) |
| Multiplayer | [10-netcode](10-netcode.md) | [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md), [01-disc-physics](01-disc-physics.md) |
| Season & career | [11-season-management](11-season-management.md) | [13-ui-ux](13-ui-ux.md), [09-ai-system](09-ai-system.md) |
| Audio | [12-audio](12-audio.md) | [07-rendering](07-rendering.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md) |
| UI/UX | [13-ui-ux](13-ui-ux.md) | [04-player-controls](04-player-controls.md), [11-season-management](11-season-management.md) |
| Tutorial | [14-tutorial-onboarding](14-tutorial-onboarding.md) | [04-player-controls](04-player-controls.md), [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md) |
| Roadmap | [15-technical-roadmap](15-technical-roadmap.md) | All documents |

## Design Principles
1. **Simulation First** — Physics and rules drive gameplay; animations follow state, not vice versa
2. **Deterministic Core** — The game loop must be deterministic for rollback netcode compatibility
3. **Crate Isolation** — `frisbee-physics` has zero Bevy dependency; other crates minimize cross-dependencies
4. **Data-Driven** — Throw parameters, AI weights, player stats, and venue definitions live in config/data files
5. **Progressive Complexity** — Tutorial system gradually introduces mechanics; AI difficulty scales smoothly

## Documentation Governance
- Going forward, any touched/new design doc should include a metadata block: `doc_version`, `last_validated_commit`, `last_updated_utc`
- Any change to workspace versions/crate topology must update `00-index.md` in the same PR
- Netcode/determinism-impacting changes must update [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md) and [10-netcode](10-netcode.md) together

## Document Status Tracker
| Doc | Status | Notes |
|-----|--------|-------|
| 00-index | Complete | This document |
| 01-disc-physics | Complete | Core physics model with real aero coefficients |
| 02-wind-field | Complete | GPU compute wind system |
| 03-ultimate-rules-gameplay | Complete | Full WFDF rules mapping |
| 04-player-controls | Complete | Gamepad + KB/M layouts |
| 05-stickman-animation | Complete | Swag system + procedural IK |
| 06-field-and-environment | Complete | Venue types + procedural gen |
| 07-rendering | Complete | Camera system + VFX pipeline |
| 08-bevy-ecs-architecture | Complete | Full ECS component/system layout |
| 09-ai-system | Complete | 3-layer AI architecture |
| 10-netcode | Complete | Rollback architecture |
| 11-season-management | Complete | Season + RPG-lite career |
| 12-audio | Complete | Procedural audio design |
| 13-ui-ux | Complete | HUD + menu system |
| 14-tutorial-onboarding | Complete | Progressive tutorial flow |
| 15-technical-roadmap | Complete | 5-phase implementation plan |
