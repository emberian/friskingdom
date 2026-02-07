# 08 — Engine Architecture

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom is built as a Bevy ECS application using a Cargo workspace with multiple crates. This document defines the plugin architecture, core components, system scheduling, state machine, and event system. The architecture prioritizes determinism (for rollback netcode), modularity (crate isolation), and data-driven design.

Related docs: All other documents. This is the architectural backbone.

## Cargo Workspace

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/frisbee-physics",
    "crates/fk-core",
    "crates/fk-game",
    "crates/fk-ai",
    "crates/fk-render",
    "crates/fk-audio",
    "crates/fk-netcode",
    "crates/fk-ui",
    "crates/fk-season",
    "crates/fk-tutorial",
    "crates/fk-debug",
    "crates/fk-test-utils",
]
resolver = "2"

[workspace.dependencies]
bevy = "0.18"
bevy_ggrs = "0.17"
bevy_matchbox = "0.10"
bevy_kira_audio = "0.21"
bevy_egui = "0.32"
serde = { version = "1", features = ["derive"] }
ron = "0.8"
rand = "0.8"
```

### Crate Dependency Graph
```
frisbee-physics (standalone, no Bevy dependency)
     ^
fk-core (shared types, components, events -- depends on frisbee-physics + bevy)
     ^
  +--+------+------+------+------+------+------+
  |  |      |      |      |      |      |      |
fk-game  fk-ai  fk-render  fk-audio  fk-ui  fk-season  fk-tutorial
  |         |                              |
  +---------+                              |
       ^                                   |
  fk-netcode ------------------------------+
```

Key rules:
- `frisbee-physics` has ZERO Bevy dependency. Pure Rust + math.
- `fk-core` defines all shared types. Other crates depend on it but not on each other.
- `fk-netcode` depends on `fk-game` (to snapshot/restore state) and `fk-season` (for matchmaking context).

## Bevy Plugin Architecture

Each crate exposes a Bevy plugin:

```rust
// Main app setup
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Core plugins
        .add_plugins(FkCorePlugin)
        .add_plugins(FkGamePlugin)
        .add_plugins(FkAiPlugin)
        .add_plugins(FkRenderPlugin)
        .add_plugins(FkAudioPlugin)
        .add_plugins(FkUiPlugin)
        .add_plugins(FkSeasonPlugin)
        .add_plugins(FkTutorialPlugin)
        // Conditional plugins
        .add_plugins(FkNetcodePlugin) // only in multiplayer
        .run();
}
```

### Plugin Responsibilities

| Plugin | Crate | Responsibilities |
|--------|-------|-----------------|
| `FkCorePlugin` | fk-core | Register components, events, resources, game states |
| `FkGamePlugin` | fk-game | Game rules, disc physics integration, player movement, stall count, turnovers, scoring |
| `FkAiPlugin` | fk-ai | AI decision-making for all non-human-controlled players |
| `FkRenderPlugin` | fk-render | Camera, stickman rendering, disc VFX, environment, particles, post-processing |
| `FkAudioPlugin` | fk-audio | Sound effects, music, crowd audio, spatial audio |
| `FkUiPlugin` | fk-ui | HUD, menus, scoreboards, settings |
| `FkSeasonPlugin` | fk-season | Season mode, career mode, roster management, progression |
| `FkTutorialPlugin` | fk-tutorial | Tutorial flow, practice mode, contextual hints |
| `FkNetcodePlugin` | fk-netcode | Rollback netcode, input sync, matchmaking |

## Core Components (fk-core)

### Entity Archetypes

#### Player Entity
```rust
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub team: TeamMember,
    pub position: Position,
    pub velocity: Velocity,
    pub stats: PlayerStats,
    pub stamina: Stamina,
    pub animation: ProceduralAnimation,
    pub appearance: PlayerAppearance,
    pub swag: SwagMeter,
    pub role: PlayerRole,
}

#[derive(Component)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub handedness: Handedness,
}

#[derive(Component)]
pub struct TeamMember {
    pub team: Team,
}

#[derive(Component, Clone, Copy)]
pub struct Position(pub Vec3);

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct PlayerStats {
    pub speed: f32,           // 0-1, max sprint speed
    pub acceleration: f32,    // 0-1, how fast player reaches top speed
    pub throwing_power: f32,  // 0-1, max throw speed
    pub throwing_accuracy: f32,// 0-1, release angle consistency
    pub catching: f32,        // 0-1, catch success rate
    pub agility: f32,         // 0-1, direction change speed
    pub jumping: f32,         // 0-1, max jump height
    pub endurance: f32,       // 0-1, stamina pool size and regen
    pub disc_iq: f32,         // 0-1, AI decision quality (for AI-controlled)
}

#[derive(Component)]
pub struct Stamina {
    pub current: f32,         // 0-max
    pub max: f32,             // based on endurance stat
    pub regen_rate: f32,      // per second while not sprinting
    pub exhausted: bool,      // true when current == 0
}

#[derive(Component)]
pub enum PlayerRole {
    Handler,
    Cutter,
    Hybrid,
}
```

#### Disc Entity
```rust
#[derive(Bundle)]
pub struct DiscBundle {
    pub disc: Disc,
    pub position: Position,
    pub velocity: Velocity,
    pub disc_state: DiscPhysicsState,
    pub disc_visual: DiscVisual,
}

#[derive(Component)]
pub struct Disc;

#[derive(Component)]
pub struct DiscPhysicsState {
    pub state: frisbee_physics::DiscState,  // full physics state
    pub in_flight: bool,
    pub grounded: bool,
    pub held_by: Option<PlayerId>,
}

#[derive(Component)]
pub struct DiscVisual {
    pub spin_rate_visual: f32,  // for rendering spin blur
    pub trail_positions: VecDeque<Vec3>,  // for trail rendering
}
```

#### Field Entity
```rust
#[derive(Bundle)]
pub struct FieldBundle {
    pub field: Field,
    pub venue: VenueConfig,
    pub surface: SurfaceCondition,
    pub weather: WeatherState,
    pub wind: WindConfig,
}

#[derive(Component)]
pub struct Field {
    pub length: f32,          // 100m total
    pub width: f32,           // 37m
    pub end_zone_depth: f32,  // 18m
}
```

### Marker Components
```rust
#[derive(Component)] pub struct Controlled;           // currently controlled by human
#[derive(Component)] pub struct AiControlled;          // controlled by AI
#[derive(Component)] pub struct OnOffense;             // currently on offense
#[derive(Component)] pub struct OnDefense;             // currently on defense
#[derive(Component)] pub struct HasDisc;               // currently holding the disc
#[derive(Component)] pub struct Marker;                // the defender marking the thrower
#[derive(Component)] pub struct CutTarget;             // the player currently cutting
```

## Resources

```rust
#[derive(Resource)]
pub struct GameClock {
    pub match_time: f64,       // seconds since match start
    pub point_time: f64,       // seconds since current point start
    pub half: u8,              // 1 or 2
    pub point_number: u32,     // current point
}

#[derive(Resource)]
pub struct Scoreboard {
    pub home_score: u32,
    pub away_score: u32,
    pub points_to_win: u32,
    pub cap: Option<u32>,
}

#[derive(Resource)]
pub struct StallCount {
    pub count: f32,            // 0.0 to 10.0
    pub active: bool,
    pub marker_id: Option<PlayerId>,
}

#[derive(Resource)]
pub struct MatchWindField {
    pub field: CompositeWindField,
    pub config: MatchWindConfig,
}

#[derive(Resource)]
pub struct PlayerInput {
    // as defined in 04-player-controls
    pub move_dir: Vec2,
    pub aim_dir: Vec2,
    pub throw_power: f32,
    pub throw_type: Option<ThrowType>,
    pub sprint: bool,
    pub layout_bid: bool,
    pub switch_player: bool,
    pub call_play: Option<PlayCall>,
    pub context: InputContext,
}

#[derive(Resource)]
pub struct MatchConfig {
    pub points_to_win: u32,
    pub time_cap_seconds: Option<f64>,
    pub wind_enabled: bool,
    pub difficulty: Difficulty,
    pub venue: VenueConfig,
}
```

## Game States

```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    TeamSelect,
    VenueSelect,
    InGame,
    PostGame,
    SeasonHub,
    CareerHub,
    Settings,
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::InGame)]
pub enum GamePhase {
    PrePoint,        // team setup, substitutions
    Pull,            // pulling team throws off
    LivePlay,        // active gameplay
    DiscInFlight,    // disc is airborne (sub-phase of LivePlay)
    Stoppage,        // foul discussion, timeout
    PointScored,     // celebration, then transition to PrePoint
    HalfTime,        // between halves
    GameOver,        // final score, triggers PostGame
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::InGame)]
pub enum PauseState {
    Playing,
    Paused,
    Replay,
}
```

## System Scheduling

### System Sets
```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,              // Read and process input
    AiDecision,         // AI makes decisions
    Movement,           // Apply movement to players
    DiscPhysics,        // Step disc simulation
    Collision,          // Catch detection, ground collision
    GameRules,          // Stall count, turnovers, scoring
    Animation,          // Procedural animation update
    Camera,             // Camera positioning
    Audio,              // Audio triggers and updates
    Rendering,          // Visual updates (trails, particles, etc.)
    Ui,                 // UI updates
}
```

### Execution Order
```
Input -> AiDecision -> Movement -> DiscPhysics -> Collision -> GameRules -> Animation -> Camera -> Audio -> Rendering -> Ui
```

### Fixed Timestep vs Frame Rate

| System Set | Timestep | Rate | Reason |
|-----------|----------|------|--------|
| Input | Fixed | 60 Hz | Deterministic for netcode |
| AiDecision | Fixed | 60 Hz | Deterministic |
| Movement | Fixed | 60 Hz | Deterministic |
| DiscPhysics | Fixed | 60 Hz (4x sub-step = 240 Hz internal) | Accuracy + determinism |
| Collision | Fixed | 60 Hz | Deterministic |
| GameRules | Fixed | 60 Hz | Deterministic |
| Animation | Frame rate | Variable | Visual smoothness |
| Camera | Frame rate | Variable | Visual smoothness |
| Audio | Frame rate | Variable | Low latency |
| Rendering | Frame rate | Variable | GPU-bound |
| Ui | Frame rate | Variable | Responsive |

The fixed-timestep systems are the "simulation" -- everything that must be deterministic for rollback netcode. Frame-rate systems are presentation-only and can safely vary.

```rust
// In FkGamePlugin::build()
app.insert_resource(Time::<Fixed>::from_hz(60.0));

app.configure_sets(
    FixedUpdate,
    (
        GameSet::Input,
        GameSet::AiDecision,
        GameSet::Movement,
        GameSet::DiscPhysics,
        GameSet::Collision,
        GameSet::GameRules,
    ).chain()
);

app.configure_sets(
    Update,
    (
        GameSet::Animation,
        GameSet::Camera,
        GameSet::Audio,
        GameSet::Rendering,
        GameSet::Ui,
    ).chain()
);
```

## Event System

Events drive communication between systems without direct coupling:

```rust
// Disc events
pub struct DiscThrown {
    pub thrower: PlayerId,
    pub throw_type: ThrowType,
    pub initial_state: frisbee_physics::DiscState,
}
pub struct DiscCaught {
    pub catcher: PlayerId,
    pub position: Vec3,
    pub was_layout: bool,
    pub was_contested: bool,
}
pub struct DiscDropped {
    pub intended_catcher: Option<PlayerId>,
    pub position: Vec3,
}
pub struct DiscOutOfBounds {
    pub crossing_point: Vec3,
}
pub struct DiscBlocked {
    pub blocker: PlayerId,
    pub position: Vec3,
}

// Game flow events
pub struct PointScored {
    pub scoring_team: Team,
    pub scorer: PlayerId,
    pub assister: Option<PlayerId>,
    pub is_callahan: bool,
}
pub struct TurnoverOccurred {
    pub reason: TurnoverReason,
    pub position: Vec3,
    pub new_offense: Team,
}
pub struct PullThrown {
    pub puller: PlayerId,
    pub initial_state: frisbee_physics::DiscState,
}
pub struct StallOut {
    pub thrower: PlayerId,
}

// Player events
pub struct PlayerSwitched {
    pub new_controlled: PlayerId,
    pub previous: Option<PlayerId>,
}
pub struct LayoutAttempt {
    pub player: PlayerId,
    pub direction: Vec3,
}
pub struct PlayCalled {
    pub team: Team,
    pub play: PlayCall,
}

// Swag events
pub struct SwagEvent {
    pub player: PlayerId,
    pub delta: f32,
    pub reason: SwagReason,
}
```

### Event Flow Example: Scoring a Point
```
1. DiscThrown { thrower: handler_id, ... }
2. [Physics steps disc through flight]
3. DiscCaught { catcher: cutter_id, position: end_zone_pos, ... }
4. [GameRules system detects catch in end zone]
5. PointScored { scoring_team: Home, scorer: cutter_id, assister: handler_id, ... }
6. SwagEvent { player: cutter_id, delta: +0.25, reason: Goal }
7. SwagEvent { player: handler_id, delta: +0.15, reason: Assist }
8. [GamePhase transitions to PointScored]
9. [Celebration animations trigger]
10. [Scoreboard updates]
11. [Crowd audio reacts]
12. [After celebration, GamePhase transitions to PrePoint]
```

## Deterministic Simulation Core

For rollback netcode (see [10-netcode](10-netcode.md)), the simulation must be fully deterministic:

### Snapshot/Restore
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub frame: u64,
    pub players: Vec<PlayerSnapshot>,
    pub disc: DiscSnapshot,
    pub stall_count: f32,
    pub score: (u32, u32),
    pub game_phase: GamePhase,
    pub rng_state: [u8; 32],  // PRNG state for reproducibility
}

impl GameSnapshot {
    pub fn capture(world: &World) -> Self { /* query all rollback-relevant state */ }
    pub fn restore(&self, world: &mut World) { /* overwrite world state from snapshot */ }
    pub fn checksum(&self) -> u64 { /* hash for desync detection */ }
}
```

### What's Deterministic (Rollback-Safe)
- Player positions, velocities
- Disc physics state
- Stall count
- Score
- Game phase
- AI decisions (seeded PRNG)
- Wind field (seeded)

### What's NOT Deterministic (Presentation-Only)
- Camera position
- Animation bone transforms
- Particle positions
- Audio state
- UI state

### Deterministic Randomness Contract
Randomness is allowed in simulation systems (AI variation, catch outcomes, wind events), but only through a deterministic API that is snapshot-safe and order-independent.

```rust
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct DeterministicRng {
    pub match_seed: u64,
    pub frame: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RngStream {
    AiStrategic,
    AiTactical,
    CatchResolution,
    WindEvents,
    RulesEvents,
}

pub fn sample_unit(
    rng: &DeterministicRng,
    stream: RngStream,
    entity: Option<Entity>,
    salt: u32,
) -> f32 {
    // Stateless hash-based sampling:
    // same (seed, frame, stream, entity, salt) => same output on all clients.
    // This avoids order-dependent "consume next random number" bugs.
    let key = make_key(rng.match_seed, rng.frame, stream, entity, salt);
    hash_to_unit_f32(key)
}
```

Rules:
- No `thread_rng()` or ad-hoc `StdRng` in rollback systems.
- No mutable RNG iteration in entity loops.
- All simulation randomness must include stable keys (`frame`, stream, entity id, salt).
- `DeterministicRng` frame value is part of snapshot/restore.

## Data-Driven Configuration

Game data is stored in RON files loaded at startup:

```
assets/
+-- config/
|   +-- game_rules.ron        # Points to win, stall speed, etc.
|   +-- difficulty.ron        # AI parameters per difficulty level
|   +-- disc_params.ron       # Disc physical constants
|   +-- throw_catalog.ron     # Throw type defaults
+-- teams/
|   +-- team_001.ron          # Team roster, colors, home venue
|   +-- ...
+-- venues/
|   +-- park_default.ron
|   +-- beach_sunset.ron
|   +-- ...
+-- players/
    +-- player_templates.ron  # Base stat distributions by role
    +-- names.ron             # Name generation pools
```

## Error Handling Strategy

- **Panics**: Only in truly unrecoverable situations (corrupt game state)
- **Result types**: Used for I/O, config loading, network operations
- **ECS queries**: Use `Query::get()` with graceful fallback rather than assuming entity existence
- **Physics**: Clamp values to prevent NaN propagation (e.g., normalize vectors with length check)
