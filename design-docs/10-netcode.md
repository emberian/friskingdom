# 10 — Multiplayer & Netcode

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom supports both local multiplayer (split-screen and shared-screen) and online multiplayer using rollback netcode. The rollback architecture is built on GGRS (a Rust GGPO implementation) with Matchbox for WebRTC-based peer-to-peer signaling. Deterministic simulation is the foundation — both clients run identical game logic so only inputs need to be synchronized.

Related docs: [01-disc-physics](01-disc-physics.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md), [04-player-controls](04-player-controls.md)

## Architecture Overview

```
┌──────────────────┐        WebRTC         ┌──────────────────┐
│    Client A       │◄────────────────────►│    Client B       │
│                    │    (peer-to-peer)     │                    │
│  ┌──────────┐     │                       │     ┌──────────┐  │
│  │  Input    │     │    Input exchange     │     │  Input    │  │
│  │  System   │────►│◄────────────────────►│◄────│  System   │  │
│  └──────────┘     │    (per frame)        │     └──────────┘  │
│       │            │                       │          │         │
│       ▼            │                       │          ▼         │
│  ┌──────────┐     │                       │     ┌──────────┐  │
│  │Simulation│     │   Identical logic     │     │Simulation│  │
│  │  (60Hz)  │     │   Same inputs →       │     │  (60Hz)  │  │
│  │          │     │   Same outputs        │     │          │  │
│  └──────────┘     │                       │     └──────────┘  │
│       │            │                       │          │         │
│       ▼            │                       │          ▼         │
│  ┌──────────┐     │                       │     ┌──────────┐  │
│  │Rendering │     │   Independent         │     │Rendering │  │
│  │  (var)   │     │   (not synced)        │     │  (var)   │  │
│  └──────────┘     │                       │     └──────────┘  │
└──────────────────┘                       └──────────────────┘
```

## Rollback Netcode

### Authority Model
FrisKingdom online matches use **deterministic peer-to-peer rollback**:
- No gameplay-authoritative server during a live match
- Both peers run the same simulation and exchange only inputs
- Desync is treated as a determinism/sync fault, not "client A is the authority"
- Ranked and unranked use the same simulation model

### How Rollback Works
1. Each frame, Client A sends its input to Client B, and vice versa
2. If Client A hasn't received Client B's input for frame N, it **predicts** (assumes last known input continues)
3. Both clients advance the simulation using available/predicted inputs
4. When the actual input arrives (e.g., 3 frames late), the game:
   a. **Rolls back** to the last confirmed frame
   b. Re-simulates forward with correct inputs
   c. Arrives back at the current frame with corrected state
5. If the prediction was correct, no visible change occurs
6. If the prediction was wrong, a small "correction" is applied (usually invisible for 1–3 frame discrepancies)

### GGRS Integration

```rust
// In FkNetcodePlugin
use bevy_ggrs::prelude::*;

pub struct FkNetcodePlugin;

impl Plugin for FkNetcodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(GgrsPlugin::<GgrsConfig>::default())
            .set_rollback_schedule_fps(60)
            .add_systems(ReadInputs, read_local_inputs)
            .add_systems(
                GgrsSchedule,
                (
                    apply_inputs,
                    ai_strategic_system,
                    ai_tactical_system,
                    ai_reactive_system,
                    player_movement_system,
                    disc_physics_system,
                    collision_system,
                    game_rules_system,
                ).chain()
            )
            .rollback_component_with_clone::<Position>()
            .rollback_component_with_clone::<Velocity>()
            .rollback_component_with_clone::<DiscPhysicsState>()
            .rollback_component_with_clone::<Stamina>()
            .rollback_component_with_clone::<SwagMeter>()
            .rollback_component_with_clone::<AiController>()
            .rollback_resource_with_clone::<StallCount>()
            .rollback_resource_with_clone::<Scoreboard>()
            .rollback_resource_with_clone::<GameClock>()
            .rollback_resource_with_clone::<DeterministicRng>();
    }
}
```

### GGRS Configuration
```rust
pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = NetworkInput;
    type State = u8;          // not used (we use Bevy component snapshots)
    type Address = PeerId;    // Matchbox peer ID
}

#[derive(Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct NetworkInput {
    pub move_x: i8,           // -128 to 127 (quantized from -1.0 to 1.0)
    pub move_y: i8,
    pub aim_x: i8,
    pub aim_y: i8,
    pub buttons: u16,         // bitfield for all buttons
    pub throw_power: u8,      // 0–255 (quantized from 0.0 to 1.0)
    pub _padding: u8,         // alignment
}
// Total: 8 bytes per player per frame
```

**Button bitfield**:
```
Bit 0:  Sprint
Bit 1:  Throw release
Bit 2:  Layout/bid
Bit 3:  Switch player
Bit 4:  Backhand
Bit 5:  Forehand
Bit 6:  Hammer
Bit 7:  Scoober
Bit 8:  Modifier (LT)
Bit 9:  Fake
Bit 10: Call for disc
Bit 11: Jump
Bit 12-15: D-pad (play calls)
```

### Rollback Parameters
| Parameter | Value | Notes |
|-----------|-------|-------|
| Max prediction frames | 8 | Maximum frames ahead without confirmed input |
| Input delay (local) | 2 frames | Smooths out jitter, adds 33ms latency |
| Max rollback frames | 8 | Maximum frames to re-simulate |
| Desync detection | Every 60 frames | Checksum comparison |
| Acceptable RTT | <150ms | Above this, input delay increases |

## Deterministic Simulation Requirements

For rollback to work, both clients must produce **identical** game state given identical inputs. This requires:

### Floating Point Determinism
- All simulation math uses `f64` (in frisbee-physics crate) or deterministic `f32` operations
- No `fast_math` or platform-specific optimizations in simulation code
- Trigonometric functions: use Taylor series approximations or lookup tables (not platform `libm`)
- Square root: use standard Rust `f64::sqrt()` (IEEE 754 compliant)
- Division by zero: explicitly guarded in all paths

### PRNG Determinism
- All random decisions use a seeded `StdRng` (ChaCha-based)
- The seed is agreed upon at match start
- The PRNG state is part of the rollback snapshot
- No use of `thread_rng()` or system entropy in simulation code

### Iteration Order
- ECS queries must produce consistent ordering
- Components are iterated by Entity ID (sorted)
- No `HashMap` iteration in simulation (use `BTreeMap` or sorted `Vec`)
- No floating-point-based sorting keys (use integer keys)

### What Must Be Deterministic
| System | Deterministic? | Notes |
|--------|---------------|-------|
| Player movement | Yes | Fixed timestep, same math |
| Disc physics | Yes | RK4 with f64, fixed 240Hz substep |
| AI decisions | Yes | Seeded PRNG, sorted queries |
| Wind field | Yes | Seeded noise, pure function |
| Stall count | Yes | Fixed increment rate |
| Collision/catch | Yes | Same detection logic, seeded catch probability |
| Game rules | Yes | Deterministic state transitions |
| Camera | No | Presentation only |
| Animation | No | Presentation only |
| Audio | No | Presentation only |
| Particles | No | Presentation only |

## Matchmaking

### Matchbox WebRTC Signaling
```rust
use bevy_matchbox::prelude::*;

fn start_matchmaking(mut commands: Commands) {
    let room_url = "wss://match.friskingdom.example/room";
    commands.insert_resource(
        MatchboxSocket::new_ggrs(room_url)
    );
}

fn check_peers(
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    if players.len() >= 2 {
        // Start GGRS session
        let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
            .with_num_players(2)
            .with_input_delay(2);

        for (i, player) in players.into_iter().enumerate() {
            session_builder = session_builder
                .add_player(player, i)
                .expect("failed to add player");
        }

        let channel = socket.take_channel(0).unwrap();
        let session = session_builder
            .start_p2p_session(channel)
            .expect("failed to start session");

        commands.insert_resource(bevy_ggrs::Session::P2P(session));
        next_state.set(AppState::InGame);
    }
}
```

### Matchmaking Modes
| Mode | Players | Description |
|------|---------|------------|
| Quick Match | 1v1 | Find opponent, play one game |
| Ranked Match | 1v1 | ELO-rated competitive play |
| Private Room | 1v1 | Share room code with friend |
| Local 1v1 | 2 (same machine) | Shared screen, two gamepads |
| Local Coop | 2 (same machine) | Both on same team vs AI |

### Lobby System
```rust
pub struct Lobby {
    pub room_code: String,       // 6-character code for private rooms
    pub host: PeerId,
    pub players: Vec<LobbyPlayer>,
    pub settings: MatchSettings,
    pub state: LobbyState,
}

pub struct LobbyPlayer {
    pub peer_id: PeerId,
    pub display_name: String,
    pub team_choice: Team,
    pub ready: bool,
}

pub struct MatchSettings {
    pub points_to_win: u32,
    pub time_cap: Option<u32>,
    pub venue: VenueType,
    pub wind: WindSetting,       // off, light, medium, heavy, random
    pub ai_difficulty: Difficulty,
}
```

## Local Multiplayer

### Shared Screen (Default)
- Two players share one screen with the broadcast camera
- Each player uses a separate gamepad
- Player indicators clearly differentiate which character each human controls
- Same deterministic simulation; no netcode needed

### Split Screen (Optional)
- Horizontal split: each player gets their own camera view
- Each camera independently follows their controlled player
- Performance target: maintain 60 FPS with two viewports (reduce particle count, grass density)

## State Synchronization & Desync Detection

### Checksum Verification
Every 60 frames (~1 second), both clients compute a checksum of the game state and compare:

```rust
pub fn compute_game_checksum(
    players: &Query<(Entity, &Position, &Velocity, &Stamina), With<Player>>,
    disc: &Query<&DiscPhysicsState, With<Disc>>,
    scoreboard: &Res<Scoreboard>,
    stall: &Res<StallCount>,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash all player positions (sorted by entity ID for consistency)
    let mut player_data: Vec<_> = players.iter().collect();
    player_data.sort_by_key(|(entity, _, _, _)| *entity);
    for (_entity, pos, vel, stamina) in &player_data {
        pos.0.x.to_bits().hash(&mut hasher);
        pos.0.y.to_bits().hash(&mut hasher);
        pos.0.z.to_bits().hash(&mut hasher);
        vel.0.x.to_bits().hash(&mut hasher);
        vel.0.y.to_bits().hash(&mut hasher);
        vel.0.z.to_bits().hash(&mut hasher);
    }

    // Hash disc state
    if let Ok(disc_state) = disc.get_single() {
        disc_state.state.position.x.to_bits().hash(&mut hasher);
        // ... hash all disc state fields
    }

    // Hash game state
    scoreboard.home_score.hash(&mut hasher);
    scoreboard.away_score.hash(&mut hasher);
    stall.count.to_bits().hash(&mut hasher);

    hasher.finish()
}
```

### Desync Recovery
If a checksum mismatch is detected:
1. **Immediate rollback retry**: Re-simulate from latest confirmed frame with received inputs
2. **Pause + snapshot exchange** (unranked only): Both peers exchange full snapshots for diagnostics, then resume only if checksums converge
3. **Hard failure**: If mismatch persists for N checks (e.g., 3), terminate match and upload desync report

Notes:
- No interpolation toward an "authoritative client" in online ranked play
- Persistent desync indicates a determinism defect or data mismatch and should fail closed

## Network Performance

### Bandwidth
- Input size: 8 bytes x 60 Hz = 480 bytes/second per player
- With overhead (headers, checksums): ~2 KB/s per player
- Total bandwidth: ~4 KB/s (extremely low)

### Latency Handling
| RTT | Input Delay | Experience |
|-----|------------|-----------|
| 0–30ms | 2 frames (33ms) | Perfect, no noticeable delay |
| 30–80ms | 2 frames | Great, prediction rarely wrong |
| 80–150ms | 3 frames (50ms) | Good, occasional small corrections |
| 150–250ms | 4–5 frames | Playable, visible corrections on direction changes |
| 250ms+ | Not recommended | Warning shown to players |

### Spectator Mode
- Spectators receive inputs from both players with a 30-frame delay (0.5s buffer)
- No rollback needed — spectators always have confirmed inputs
- Spectators can use any camera mode freely

## Anti-Cheat

### Determinism-Based Verification
Since both clients run the same simulation:
- If checksums consistently diverge, the cause is either a determinism bug or tampered client logic
- Input validation: reject physically impossible inputs (e.g., move_x/move_y magnitude >1)
- Speed hack detection: inputs arriving faster than real-time

### Limitations
- Peer-to-peer means no authoritative server; sophisticated cheats could be hard to detect
- Ranked mode may eventually need a relay server for better anti-cheat
- Client-side: all opponent positions are known (no fog of war needed in frisbee), so wallhacks are not a concern

## Session Flow

```
1. MainMenu → "Online Match"
2. Select mode (Quick / Ranked / Private)
3. Enter matchmaking lobby
4. [Matchbox connects peers via WebRTC]
5. Both players ready up
6. Exchange match settings (host's settings are authoritative)
7. Exchange initial seed (for wind, AI, etc.)
8. Countdown (3... 2... 1...)
9. Game starts — GGRS session active
10. Play match (rollback netcode handles sync)
11. Game ends → both clients show post-game screen
12. Option: rematch (return to step 8) or leave (return to step 2)
```

## Replay System

### Replay Recording
During online matches, all inputs are recorded:
```rust
pub struct ReplayRecording {
    pub match_settings: MatchSettings,
    pub initial_seed: u64,
    pub player_inputs: Vec<(u64, [NetworkInput; 2])>, // (frame, [p1_input, p2_input])
}
```

Since the simulation is deterministic, replaying the same inputs with the same seed reproduces the exact match. Replay files are tiny — a 15-point game is ~200KB of input data.

### Replay Playback
- Load replay file
- Initialize game state with match settings and seed
- Feed recorded inputs frame-by-frame
- Free camera control during playback
- Slow-mo, fast-forward, rewind (rewind = reset to frame 0 and fast-forward)
- Frame-by-frame stepping
