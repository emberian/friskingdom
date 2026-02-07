# 15 — Technical Roadmap

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
The FrisKingdom implementation is divided into 5 phases, each building on the previous. Each phase results in a playable milestone. The roadmap prioritizes core gameplay feel first (disc physics + player movement), then adds rules and AI, then polish, then management modes, and finally multiplayer.

Related docs: All documents. This roadmap references every system.

## Phase Summary

| Phase | Name | Duration | Key Milestone |
|-------|------|----------|--------------|
| 1 | Foundation | Weeks 1–4 | Disc flight + throw/catch loop + deterministic rollback vertical slice (2 players) |
| 2 | Core Gameplay | Weeks 5–10 | Full 7v7 game with rules, AI, scoring — playable Quick Match |
| 3 | Polish | Weeks 11–15 | Camera system, audio, VFX, UI — game feels good to play |
| 4 | Modes & Management | Weeks 16–20 | Season mode, career mode, tutorial — full single-player experience |
| 5 | Multiplayer Productization | Weeks 21–25 | Online matchmaking, ranked flow, replay/share, production hardening |

**Post-launch**: GPU physics compute, additional venues, community features.

## Phase 1: Foundation (Weeks 1–4)

### Goal
Build the physics engine, basic rendering, and the core throw→fly→catch loop.

### Week 1: Project Setup & Disc Physics Crate
**Tasks**:
- Initialize Cargo workspace with all crate stubs
- Implement `frisbee-physics` crate:
  - `DiscState`, `DiscParams` structs
  - Aerodynamic force model (lift, drag, pitching moment, gravity)
  - RK4 integrator at 240 Hz
  - `WindField` trait with `NullWind` implementation
  - `ThrowType` enum with default parameters for all 8 throw types
  - `Simulator::simulate_flight()` — run a full disc flight to ground
- Unit tests: verify backhand flight curves right then left, forehand opposite, hammer arcs high and fades
- CLI test tool: print disc trajectory as CSV for plotting

**Deliverable**: `cargo test -p frisbee-physics` passes. Disc flight paths are physically plausible.

**Key files**:
- `crates/frisbee-physics/src/lib.rs`
- `crates/frisbee-physics/src/aerodynamics.rs`
- `crates/frisbee-physics/src/integration.rs`
- `crates/frisbee-physics/src/throws.rs`
- `crates/frisbee-physics/src/disc.rs`

### Week 2: Bevy Scaffolding & Field
**Tasks**:
- Set up Bevy app with `DefaultPlugins`
- Implement `FkCorePlugin`: register components, events, resources, game states
- Create basic field mesh (100m × 37m plane with line markings)
- Spawn disc entity with `DiscPhysicsState` component
- Integrate physics: run `frisbee-physics` simulation each `FixedUpdate` tick (4× substep)
- Visual: render disc as a simple colored cylinder
- Implement disc trail (ribbon mesh following disc path)
- Basic camera: fixed broadcast-angle camera looking at field

**Deliverable**: Launch the game, see a field, press a key to throw a disc, watch it fly across the field with correct physics. Disc trail visible.

**Key files**:
- `crates/fk-core/src/lib.rs` (components, events)
- `crates/fk-game/src/lib.rs` (game plugin)
- `crates/fk-game/src/disc_system.rs`
- `crates/fk-render/src/field.rs`
- `crates/fk-render/src/camera.rs`

### Week 3: Stickman Player & Movement
**Tasks**:
- Implement stickman rig: 12-bone capsule/sphere hierarchy
- Procedural IK locomotion: foot placement, step triggering, speed-adaptive stride
- Player movement system: position, velocity, acceleration, deceleration
- Sprint + stamina system
- Basic input handling: gamepad left stick for movement, button for sprint
- Spawn 14 players (2 teams of 7) with team colors
- Player switching (RB/Tab)
- Controlled player indicator (ring at feet)

**Deliverable**: Run around the field with a stickman that has natural-looking procedural locomotion. Sprint, switch players, see teams.

**Key files**:
- `crates/fk-render/src/stickman.rs`
- `crates/fk-game/src/player_movement.rs`
- `crates/fk-game/src/input.rs`
- `crates/fk-render/src/animation/ik.rs`
- `crates/fk-render/src/animation/locomotion.rs`

### Week 4: Throwing & Catching
**Tasks**:
- Throwing interface: hold RT for power, right stick for aim, buttons for throw type
- Power gauge HUD element
- Throw prediction arc (faded line showing trajectory)
- Connect throw input to `frisbee-physics`: create `DiscState` from throw parameters
- Catching system: catch detection volume, auto-catch when player is near disc
- Layout catch: press button to dive, extended reach, reduced probability
- Pivoting: player rotates around fixed pivot foot when holding disc
- Throw animations: procedural backhand and forehand wind-up/release
- Catch animations: two-handed catch, one-handed reach

**Deliverable**: Full throw→fly→catch loop. Player picks up disc, aims, charges throw, releases. Disc flies with physics. Other player catches. Feels like playing catch.

**Key files**:
- `crates/fk-game/src/throwing.rs`
- `crates/fk-game/src/catching.rs`
- `crates/fk-ui/src/hud/power_gauge.rs`
- `crates/fk-render/src/animation/throw_anim.rs`
- `crates/fk-render/src/animation/catch_anim.rs`

### Parallel Track (Weeks 3–4): Determinism & Rollback Slice
**Tasks**:
- Build a minimal rollback state (`players + disc + stall + score + rng frame`)
- Implement checksum comparison every 60 frames for the slice
- Run two local sessions with scripted inputs and artificial latency/jitter
- Validate rollback/re-sim produces identical checksums across debug/release
- Add deterministic test harness to CI for this minimal scenario

**Deliverable**: A deterministic 1v1 half-field prototype survives latency injection without persistent desync.

---

## Phase 2: Core Gameplay (Weeks 5–10)

### Goal
Implement full ultimate frisbee rules, AI, and a playable 7v7 game.

### Week 5: Game Rules & State Machine
**Tasks**:
- Implement game state machine: PrePoint → Pull → LivePlay → PointScored → repeat
- Stall count system (visual + audio)
- Turnover detection: drops, out-of-bounds, stall-out
- Scoring detection: catch in end zone = point
- Pull mechanics: dedicated pull interface, pull landing rules
- Scoreboard resource and HUD element
- Substitution system (between points)

**Deliverable**: Play a full game to 15 with correct scoring, turnovers, and pulls. No AI yet — both teams human-controlled or simple waypoint movement.

### Week 6: Wind System
**Tasks**:
- Implement `CompositeWindField` in fk-game
- Base wind with logarithmic height profile
- Gust layer with 3D Perlin noise (fBm)
- Match wind config generation (seeded)
- Wind HUD indicator (direction + speed)
- Wind sock 3D object on field
- Connect wind to disc physics
- Test: disc flight noticeably affected by wind

**Deliverable**: Wind visibly affects disc flight. Wind indicator on HUD. Different wind conditions per match.

### Week 7–8: AI System (Basic)
**Tasks**:
- Implement 3-layer AI architecture (strategic, tactical, reactive)
- Strategic: vert stack formation positioning
- Tactical: basic cutting (in-cut/deep-cut timing), handler throw decisions
- Reactive: movement toward targets, collision avoidance
- Man defense: positioning between player and disc
- Mark AI: setting force, stall counting
- Difficulty scaling: at least 3 levels (Beginner/Casual/Competitive)
- AI-controlled teammates: follow play calls, cut when appropriate

**Deliverable**: Full 7v7 game against AI opponents. AI runs basic vert stack offense and man defense. Feels like a real game of ultimate.

### Week 9: Offensive & Defensive Depth
**Tasks**:
- Horiz stack formation
- Zone defense (cup + wings + deeps)
- Break throws (AI attempts breaks based on personality)
- Handler resets (dump pass when stall is high)
- Play calling system (D-pad commands)
- Clam defense (basic)
- Improved cut timing and spacing

**Deliverable**: Multiple offensive and defensive formations. AI switches strategies. D-pad play calling works.

### Week 10: Special Situations & Completeness
**Tasks**:
- Callahan detection and celebration
- Greatest detection (catch while airborne, throw before landing OOB)
- Layout animations for offense and defense
- Sky/jump catches (contested aerial)
- Foul detection (contact during throw/catch)
- Hand blocks / foot blocks
- Timeout system
- Halftime (switch sides, optional wind shift)
- Game-over state and post-game stats screen

**Deliverable**: Feature-complete Quick Match mode. All special situations handled. Post-game stats displayed. This is a fully playable ultimate frisbee game.

---

## Phase 3: Polish (Weeks 11–15)

### Goal
Make the game look, sound, and feel great.

### Week 11: Camera System
**Tasks**:
- Broadcast camera with dynamic zoom and tracking
- Behind-player camera mode
- Endzone camera for pulls and scoring
- Cinematic camera for replays
- Sky camera (tactical overhead view)
- Smooth transitions between camera modes
- Auto-camera system (game selects best camera)
- Camera shake on impacts

**Deliverable**: Dynamic camera that feels like watching a sports broadcast. Multiple selectable modes.

### Week 12: Audio System
**Tasks**:
- Procedural disc whoosh (speed-dependent pitch and volume)
- Disc catch snap and ground hit sounds
- Procedural footsteps (surface-dependent)
- Crowd ambience (per-venue)
- Reactive crowd (cheers on score, groans on turnover)
- Abstract commentary cues (non-verbal)
- Wind audio
- Spatial audio (3D positioned sources)
- Per-venue reverb

**Deliverable**: Full audio experience. Game sounds alive and responsive.

### Week 13: Visual Effects & Rendering
**Tasks**:
- Disc spin blur shader
- Disc shadow (height-readable)
- Stickman toon shader with rim lighting
- Post-processing: bloom, DOF (subtle), motion blur (per-object)
- Particle effects: dust puffs, grass spray, celebration confetti
- Grass rendering with wind response
- Time-of-day lighting
- Color grading per venue

**Deliverable**: Game looks polished and stylish. Stickman aesthetic fully realized.

### Week 14: Swag System & Celebrations
**Tasks**:
- Implement swag meter per player
- Swag events: score → boost, turnover → decrease, etc.
- Swag animation modifiers: posture, step bounce, arm swing, head angle
- Celebration system: solo celebrations gated by swag level
- Team celebrations triggered by context
- Procedural appearance generation (height, build, accessories)
- Jersey numbers on stickmen

**Deliverable**: Characters have personality and flair. Swag system makes the game feel alive.

### Week 15: UI Polish
**Tasks**:
- Complete HUD: scoreboard, stall count, wind, stamina, throw power
- Context-sensitive HUD visibility
- Main menu with animated background
- Team select screen
- Venue select screen
- Settings menu (all categories)
- Post-game stats screen (detailed)
- Notifications/toasts for game events
- Loading screens
- Accessibility: colorblind modes, scalable text, input remapping

**Deliverable**: Complete, polished UI. All menus functional. Accessibility options available.

---

## Phase 4: Modes & Management (Weeks 16–20)

### Goal
Add season mode, career mode, and tutorial to create a full single-player experience.

### Week 16: Tutorial System
**Tasks**:
- Tutorial state machine and lesson framework
- Chapter 1: Movement basics (4 lessons)
- Chapter 2: Throwing (6 lessons)
- Chapter 3: Catching (4 lessons)
- Chapter 4: Disc flight & advanced throws (5 lessons)
- First-time player flow

**Deliverable**: New player can go through tutorial and learn to play.

### Week 17: Tutorial (Continued) & Practice Mode
**Tasks**:
- Chapter 5: Rules of ultimate (6 lessons)
- Chapter 6: Offense (5 lessons)
- Chapter 7: Defense (5 lessons)
- Chapter 8: Advanced strategy (5 lessons, optional)
- Practice mode: throwing range, catch machine, 1v1 drills, sandbox
- Contextual hint system

**Deliverable**: Complete tutorial with all chapters. Practice mode available.

### Week 18: Season Mode
**Tasks**:
- League structure: 2 divisions of 8 teams
- Schedule generation
- Season hub UI
- Standings and statistics tracking
- Playoff bracket
- Between-game flow (review stats, manage roster, scout opponent)
- Training system (stat boosts between games)
- Season progression and champion declaration

**Deliverable**: Play through a full season from start to championship.

### Week 19: Career Mode
**Tasks**:
- Create-a-player flow (appearance, position, name)
- Starting stat generation by role
- Skill point earning system
- Skill tree implementation (17 nodes across 3 paths)
- Player traits (6 traits with unlock conditions)
- Reputation system
- Team offers and transfers
- Career dashboard UI

**Deliverable**: Create a player, join a team, play games, earn skill points, unlock abilities.

### Week 20: Venues & Content
**Tasks**:
- Implement all 6 venue types (Park, Beach, Stadium, Indoor, Forest, Rooftop)
- Venue configuration loading (RON files)
- Procedural venue generation
- Per-venue ambient audio
- Weather system (clear, cloudy, rain, fog, snow)
- Weather transitions during games
- Unlock system for venues and cosmetics
- Achievement system
- Procedural music system (dynamic layers)

**Deliverable**: Multiple varied venues. Weather affects gameplay. Unlockable content provides progression incentive.

---

## Phase 5: Multiplayer Productization (Weeks 21–25)

### Goal
Add online multiplayer with rollback netcode.

### Week 21: Full-Game Determinism Audit & Prep
**Tasks**:
- Extend the early rollback slice rules to full-game state
- Audit all simulation code for determinism issues
- Replace any non-deterministic operations (HashMap iteration, platform-specific math)
- Implement `GameSnapshot` capture/restore
- Implement checksum computation for desync detection
- Verify: same inputs produce same outputs across debug/release builds
- Verify: same inputs produce same outputs across platforms (CI test on Linux + macOS + Windows)

**Deliverable**: Determinism test suite passes. Snapshot/restore works correctly.

### Week 22: GGRS Integration
**Tasks**:
- Integrate `bevy_ggrs` plugin
- Move simulation systems to `GgrsSchedule`
- Implement `NetworkInput` serialization (8 bytes per player per frame)
- Register all rollback components and resources
- Local testing: two GGRS sessions in same process (simulated network)
- Verify rollback works: inject artificial latency, confirm game stays in sync

**Deliverable**: Two simulated players can play a full game through GGRS with rollback. No desync.

### Week 23: Networking & Matchmaking
**Tasks**:
- Integrate `bevy_matchbox` for WebRTC signaling
- Quick match matchmaking flow
- Private room system (room codes)
- Lobby UI: player list, ready-up, settings
- Pre-game sync: exchange match settings, initial seed
- Connection quality display (ping, packet loss)
- Graceful handling of disconnects

**Deliverable**: Two players on different machines can find each other and play a game online.

### Week 24: Online Polish
**Tasks**:
- Ranked match with ELO system
- Desync detection and recovery
- Spectator mode (delayed input feed)
- Network performance tuning (adaptive input delay based on RTT)
- Anti-cheat basics (input validation, speed hack detection)
- Local multiplayer: shared screen and split screen
- Post-game: rematch option

**Deliverable**: Polished online experience. Ranked play. Spectator mode.

### Week 25: Replay System & Final Polish
**Tasks**:
- Replay recording (save all inputs per frame)
- Replay playback with free camera
- Replay UI: play/pause, slow-mo, fast-forward, frame step
- Disc flight path overlay in replay
- Save replay files, share replay codes
- Final performance optimization pass
- Final bug bash
- Platform-specific testing (Windows, macOS, Linux)

**Deliverable**: Complete game with online multiplayer, replay system, and platform-ready builds.

---

## Post-Launch Roadmap

### GPU Physics Compute (Phase 6)
- Implement WGSL compute shaders for disc physics
- Batch simulation for AI training
- Replay analysis tool (simulate alternative throws)
- Performance benchmark: 1000 simultaneous disc simulations

### Content Updates
- Additional venues (desert, mountain, urban street)
- Additional weather (thunderstorm, hail)
- Community-requested throw types
- More celebrations and accessories
- Tournament mode (multi-round brackets)

### Platform Expansion
- Console ports (PlayStation, Xbox, Nintendo Switch)
- Touch controls for mobile
- Cross-platform multiplayer

### Community Features
- Custom team creation and sharing
- Replay sharing and leaderboards
- Modding support (custom venues, team rosters)
- Stat tracking and career history website

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Disc physics don't feel right | High | Medium | Invest in Phase 1 tuning; real player testing; tunable coefficients |
| Determinism bugs cause desync | High | Medium | Determinism test slice in Weeks 3–4, expanded full-suite in Week 21, CI on multiple platforms |
| AI doesn't play realistic ultimate | High | Medium | Iterative AI tuning in Phase 2; consult ultimate players; watch game film |
| Performance issues with stickman rendering | Medium | Low | 12-bone rig is simple; capsule rendering is cheap |
| Bevy breaking changes | Medium | Medium | Pin Bevy version; update in dedicated sprint |
| Rollback netcode jitter | Medium | Medium | Conservative prediction frames; input delay tuning |
| Scope creep | High | High | Strict phase gates; playable milestone each phase; cut features not fun |

## Success Criteria

| Phase | Criterion |
|-------|-----------|
| Phase 1 | Physics regression suite passes; rollback slice has 0 persistent desyncs across 1,000 scripted possessions |
| Phase 2 | Quick Match full game completes with no blocker bugs in 20 consecutive internal playtests |
| Phase 3 | 60 FPS at 1080p on target mid-range GPU; frame time p95 <= 16.6ms in match scenarios |
| Phase 4 | Season/Career save-load compatibility maintained across 3 schema versions with passing migration tests |
| Phase 5 | Online completion rate >= 95% for matches under 150ms RTT; desync incidence < 1 per 100 matches |
