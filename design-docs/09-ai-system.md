# 09 — AI System

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom uses a 3-layer AI architecture that mirrors human decision-making in ultimate frisbee: strategic (team-level play calling), tactical (individual positioning and timing), and reactive (frame-by-frame adjustments). The AI controls all non-human players — both teammates and opponents — with behavior scaled by difficulty level and modified by individual player "personality" traits.

Related docs: [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md), [11-season-management](11-season-management.md)

## Architecture

```
┌─────────────────────────────────────────────┐
│              Strategic Layer                 │
│  (Team-level, updates every 2–5 seconds)    │
│  - Formation selection                      │
│  - Play calling                             │
│  - Substitution decisions                   │
│  - Defensive scheme selection               │
└─────────────┬───────────────────────────────┘
              │ orders/formation
              ▼
┌─────────────────────────────────────────────┐
│              Tactical Layer                  │
│  (Per-player, updates every 0.5–1 second)   │
│  - Cut timing and direction                 │
│  - Handler throw decision                   │
│  - Defensive positioning                    │
│  - Mark force execution                     │
└─────────────┬───────────────────────────────┘
              │ targets/intentions
              ▼
┌─────────────────────────────────────────────┐
│              Reactive Layer                  │
│  (Per-player, updates every frame at 60Hz)  │
│  - Movement toward target                   │
│  - Collision avoidance                      │
│  - Catch/block attempts                     │
│  - Sprint/stamina management                │
└─────────────────────────────────────────────┘
```

### Deterministic Randomness in AI
All AI randomness must route through the shared deterministic RNG contract (see [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)):
- No per-system `thread_rng()` usage
- No mutable RNG consumption whose result depends on query iteration order
- Random values must be keyed by `(match_seed, frame, stream, player/entity id, salt)`

## Strategic Layer

The strategic layer makes team-level decisions. It evaluates the game state and selects formations and plays.

### Offensive Strategy Selection
```rust
pub struct OffensiveStrategy {
    pub formation: Formation,
    pub play_style: PlayStyle,
    pub tempo: Tempo,
}

#[derive(Clone, Copy)]
pub enum Formation {
    VertStack,
    HorizStack,
    SideStack,
    SplitStack,
}

#[derive(Clone, Copy)]
pub enum PlayStyle {
    Methodical,     // Short passes, patient, work up the field
    Aggressive,     // Look for hucks and fast breaks
    Balanced,       // Mix of both
    IsolationPlay,  // Clear space for a star player
}

#[derive(Clone, Copy)]
pub enum Tempo {
    Fast,           // Quick disc movement, attack early in stall
    Medium,         // Normal pace
    Slow,           // Hold disc, wait for perfect opportunity
}
```

**Selection factors**:
- Score differential (losing team may go aggressive)
- Wind conditions (windy → methodical, short passes)
- Opponent's defensive scheme (zone → horiz stack to spread defense)
- Stall count pressure (high stall → switch to faster tempo)
- Player matchups (speed advantage → deep looks; throw advantage → methodical)
- Game situation (game point → conservative; down big → aggressive)

### Defensive Strategy Selection
```rust
pub struct DefensiveStrategy {
    pub scheme: DefenseScheme,
    pub force_direction: ForceDirection,
    pub pressure_level: f32,  // 0 = passive, 1 = maximum pressure
}

#[derive(Clone, Copy)]
pub enum DefenseScheme {
    ManToMan,
    Zone3_3_1,      // 3-person cup, 3 wings, 1 deep
    Zone2_3_2,      // 2 cup, 3 mid, 2 deep
    Clam,
    Junk,
}

#[derive(Clone, Copy)]
pub enum ForceDirection {
    ForceForehand,
    ForceBackhand,
    ForceHome,
    ForceAway,
    ForceMiddle,
}
```

### Play Calling
The strategic layer can call specific plays that assign roles:

```rust
pub struct PlayCall {
    pub name: &'static str,
    pub assignments: Vec<PlayerAssignment>,
    pub trigger: PlayTrigger,
}

pub struct PlayerAssignment {
    pub role: PlayRole,
    pub target_zone: Rect,        // area on field to operate in
    pub cut_pattern: Option<CutPattern>,
    pub priority: f32,            // higher = gets disc first
}

pub enum CutPattern {
    InCut,          // run toward disc
    DeepCut,        // run away from disc toward end zone
    UnderCut,       // short in-cut, then clear
    DiagonalCut,    // diagonal across field
    ComebackCut,    // fake deep, come back
    ClearOut,       // move away to create space
}
```

## Tactical Layer

The tactical layer makes per-player decisions every 0.5–1.0 seconds.

### Offensive Cutting AI
When a player doesn't have the disc, the tactical AI decides when and where to cut:

```rust
pub fn evaluate_cut_opportunity(
    player: &Player,
    defender: &Player,
    disc_pos: Vec3,
    formation: &Formation,
    stall_count: f32,
) -> CutDecision {
    let separation = (player.position - defender.position).length();
    let space_available = check_open_space(player.position, formation);
    let stall_urgency = stall_count / 10.0;  // 0–1

    // Score different cut options
    let in_cut_score = score_in_cut(player, defender, disc_pos, space_available);
    let deep_cut_score = score_deep_cut(player, defender, disc_pos, space_available);
    let clear_score = score_clear(player, formation);

    // Add urgency bias (more cuts as stall rises)
    let urgency_bonus = stall_urgency * 0.3;

    let best = max_of(in_cut_score + urgency_bonus, deep_cut_score, clear_score);

    match best {
        _ if best == in_cut_score + urgency_bonus => CutDecision::InCut(target_position),
        _ if best == deep_cut_score => CutDecision::DeepCut(target_position),
        _ => CutDecision::Clear(clear_position),
    }
}
```

**Cut timing principles**:
1. Only one cutter should be active at a time (in vert stack)
2. Cuts should create space for subsequent cutters
3. If a cut isn't working (defender stays with you), clear out
4. Respond to the disc: if the disc moves, react accordingly

### Handler Throw Decision
When the AI handler has the disc:

```rust
pub fn evaluate_throw_options(
    handler: &Player,
    teammates: &[Player],
    defenders: &[Player],
    wind: &WindField,
    stall_count: f32,
    strategy: &OffensiveStrategy,
) -> ThrowDecision {
    let mut options: Vec<ThrowOption> = Vec::new();

    for teammate in teammates {
        // Evaluate each possible throw to each teammate
        let throws = [ThrowType::Backhand, ThrowType::Forehand, ThrowType::Hammer];
        for throw_type in throws {
            let option = evaluate_single_throw(
                handler, teammate, throw_type, defenders, wind, stall_count
            );
            options.push(option);
        }
    }

    // Score options based on:
    // - Completion probability (most important)
    // - Yardage gained
    // - Strategic value (moving disc to break side, end zone looks)
    // - Risk tolerance (based on score, game situation, personality)

    options.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Stall pressure: lower threshold for acceptable options as stall rises
    let threshold = lerp(0.7, 0.3, stall_count / 10.0);

    if options[0].score > threshold {
        ThrowDecision::Throw(options[0].clone())
    } else if stall_count < 6.0 {
        ThrowDecision::Hold  // wait for better option
    } else {
        ThrowDecision::Dump  // safe reset throw to a handler
    }
}
```

### Defensive Positioning
Each defender maintains position based on their assignment:

**Man defense positioning**:
```
Optimal position = Between your player and the disc
Bias toward preventing the most dangerous cut:
  - If guarding a cutter: shade toward the open side
  - If guarding a handler: stay close, deny easy reset
  - If marking the thrower: maintain force, prevent break throws
```

**Zone defense positioning**:
```
Cup players: Triangle formation around the disc, 3–4m away
Wings: Split the distance between sideline and cup edge
Deeps: Cover the deep space, shade toward the open side
Shift as a unit when the disc moves
```

### Mark AI (The Stall Count)
The defender marking the thrower:
```rust
pub fn mark_behavior(
    marker: &Player,
    thrower: &Player,
    force_dir: ForceDirection,
    difficulty: Difficulty,
    rng: &DeterministicRng,
) -> MarkerAction {
    // Position: stay on the force side
    let ideal_pos = compute_force_position(thrower, force_dir);

    // React to fakes: difficulty determines reaction time
    let fake_reaction_delay = match difficulty {
        Difficulty::Beginner => 0.5,    // slow to react
        Difficulty::Casual => 0.3,
        Difficulty::Competitive => 0.15,
        Difficulty::Elite => 0.08,
        Difficulty::Spirit => 0.04,     // barely fooled
    };

    // Stall count: slight timing variation to feel natural, but deterministic.
    let r = sample_unit(rng, RngStream::AiTactical, Some(marker.id.into()), 0x5A11);
    let stall_variation = (r * 2.0 - 1.0) * 0.05;

    MarkerAction {
        target_position: ideal_pos,
        stall_timing: 1.0 + stall_variation,  // seconds per count
        reaction_delay: fake_reaction_delay,
    }
}
```

## Reactive Layer

The reactive layer runs every frame (60 Hz) and handles low-level execution:

### Movement Execution
```rust
pub fn execute_movement(
    player: &mut Player,
    target: Vec3,
    context: &MovementContext,
) -> Vec3 {
    let to_target = target - player.position;
    let distance = to_target.length();

    // Steering: smooth direction changes based on agility
    let desired_dir = to_target.normalize();
    let current_dir = player.velocity.normalize_or_zero();
    let turn_rate = player.stats.agility * 8.0; // radians/sec
    let new_dir = rotate_toward(current_dir, desired_dir, turn_rate * DT);

    // Speed: accelerate toward target speed
    let target_speed = if context.sprinting && player.stamina.current > 0.0 {
        player.stats.speed * 1.5 * context.surface_modifier
    } else {
        player.stats.speed * context.surface_modifier
    };

    let acceleration = player.stats.acceleration * 15.0; // m/s²
    let new_speed = move_toward(player.speed, target_speed, acceleration * DT);

    new_dir * new_speed
}
```

### Catch/Block Attempts
```rust
pub fn attempt_catch_or_block(
    player: &Player,
    disc: &DiscState,
    is_offense: bool,
) -> Option<InterceptAction> {
    let to_disc = disc.position - player.position;
    let disc_distance = to_disc.length();

    let reach = player.reach(); // based on height + arm length
    let layout_reach = reach + 1.5;

    if disc_distance <= reach {
        // Standard catch/block attempt
        Some(InterceptAction::Standard)
    } else if disc_distance <= layout_reach && should_layout(player, disc, is_offense) {
        // Layout attempt
        Some(InterceptAction::Layout(to_disc.normalize()))
    } else if disc.position.y > 2.0 && disc_distance < reach + 0.5 {
        // Sky attempt
        Some(InterceptAction::Jump)
    } else {
        None
    }
}
```

### Collision Avoidance
Players avoid running into each other:
```rust
pub fn avoid_collisions(
    player_pos: Vec3,
    player_vel: Vec3,
    nearby_players: &[Vec3],
    avoidance_radius: f32,
) -> Vec3 {
    let mut steering = Vec3::ZERO;
    for other in nearby_players {
        let diff = player_pos - *other;
        let dist = diff.length();
        if dist < avoidance_radius && dist > 0.01 {
            steering += diff.normalize() / dist; // stronger push when closer
        }
    }
    steering.normalize_or_zero() * 2.0 // avoidance force magnitude
}
```

## AI Personality System

Each AI player has personality traits that modify their decision-making:

```rust
pub struct AiPersonality {
    pub aggression: f32,       // 0 = conservative, 1 = always attacks
    pub creativity: f32,       // 0 = textbook, 1 = tries unusual throws
    pub composure: f32,        // 0 = panics under pressure, 1 = ice cold
    pub team_play: f32,        // 0 = selfish, 1 = always finds the open player
    pub hustle: f32,           // 0 = jogs, 1 = layouts for everything
    pub disc_hog: f32,         // 0 = distributes, 1 = holds too long
}
```

**Personality effects**:
- **aggression**: Increases deep cut frequency, lowers completion threshold for hucks
- **creativity**: Adds hammer/scoober/blade to throw evaluation; unusual cut patterns
- **composure**: Maintains throw quality under high stall pressure; fewer turnovers in tight games
- **team_play**: Weights throw decisions toward the best option rather than own preference
- **hustle**: Increases sprint frequency, layout willingness, chasing down errant discs
- **disc_hog**: Increases hold time before throwing; more pump fakes

### Personality Generation
Personalities are generated per player based on their role and stats:
```rust
fn generate_personality(stats: &PlayerStats, role: &PlayerRole, rng: &mut StdRng) -> AiPersonality {
    let base = match role {
        PlayerRole::Handler => AiPersonality {
            aggression: 0.4, creativity: 0.5, composure: 0.7,
            team_play: 0.8, hustle: 0.5, disc_hog: 0.3,
        },
        PlayerRole::Cutter => AiPersonality {
            aggression: 0.6, creativity: 0.3, composure: 0.5,
            team_play: 0.6, hustle: 0.7, disc_hog: 0.1,
        },
        PlayerRole::Hybrid => AiPersonality {
            aggression: 0.5, creativity: 0.4, composure: 0.6,
            team_play: 0.7, hustle: 0.6, disc_hog: 0.2,
        },
    };
    // Add deterministic variation ±0.15 from seeded career/match generation context.
    base.randomize(rng, 0.15)
}
```

## Difficulty Scaling

AI difficulty is implemented by modifying parameters across all three layers:

| Parameter | Beginner | Casual | Competitive | Elite | Spirit |
|-----------|----------|--------|-------------|-------|--------|
| Strategic update rate | 5s | 3s | 2s | 1.5s | 1s |
| Tactical update rate | 1.0s | 0.7s | 0.5s | 0.3s | 0.2s |
| Throw accuracy noise | ±15° | ±8° | ±4° | ±2° | ±0.5° |
| Throw power noise | ±20% | ±12% | ±6% | ±3% | ±1% |
| Reaction time (catches) | 0.4s | 0.25s | 0.15s | 0.08s | 0.03s |
| Reaction time (blocks) | 0.5s | 0.35s | 0.2s | 0.1s | 0.05s |
| Cut timing quality | Poor | Okay | Good | Great | Perfect |
| Reads opponent's play | Never | Rarely | Sometimes | Often | Always |
| Adapts mid-game | No | No | Basic | Advanced | Full |
| Intentional mistakes | Frequent | Occasional | Rare | Very rare | Never |

### Adaptive Difficulty (Optional)
An optional "dynamic difficulty" mode adjusts AI parameters during the game:
- If the human player is losing badly (>5 points down), AI slightly degrades
- If the human player is winning easily (>5 points up), AI slightly improves
- Adjustments are subtle and gradual to avoid feeling artificial
- Can be disabled in settings

## Teammate AI vs Opponent AI

Teammate AI (players on the human's team) has additional constraints:
1. **Follows play calls**: When the human calls a play (D-pad), teammates execute assigned roles
2. **Defers to human**: Won't cut into spaces the human is moving toward
3. **Communicates intent**: Briefly shows cut direction before cutting (visual indicator)
4. **Avoids stupid mistakes**: Teammate AI at lower difficulties makes fewer errors than opponent AI at the same level — the human should feel their teammates are competent

## Bevy ECS Integration

```rust
#[derive(Component)]
pub struct AiController {
    pub strategic_state: StrategicState,
    pub tactical_state: TacticalState,
    pub personality: AiPersonality,
    pub decision_timer: Timer,
}

// Systems
pub fn ai_strategic_system(
    time: Res<Time>,
    game_state: Res<GameClock>,
    scoreboard: Res<Scoreboard>,
    wind: Res<MatchWindField>,
    mut teams: Query<&mut TeamAiState>,
    players: Query<(&Player, &Position, &TeamMember)>,
) { /* update team strategy every N seconds */ }

pub fn ai_tactical_system(
    time: Res<Time>,
    strategy: Query<&TeamAiState>,
    mut ai_players: Query<(&mut AiController, &Player, &Position, &PlayerStats), With<AiControlled>>,
    all_players: Query<(&Player, &Position, &TeamMember)>,
    disc: Query<&DiscPhysicsState, With<Disc>>,
    stall: Res<StallCount>,
) { /* update individual decisions */ }

pub fn ai_reactive_system(
    mut ai_players: Query<(&AiController, &mut Velocity, &Position, &mut Stamina), With<AiControlled>>,
    disc: Query<(&Position, &DiscPhysicsState), With<Disc>>,
    all_positions: Query<&Position, With<Player>>,
) { /* frame-by-frame movement execution */ }
```

All AI systems run in the `FixedUpdate` schedule within the `GameSet::AiDecision` set for determinism (see [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)).
