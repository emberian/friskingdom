# 03 — Rules & Gameplay

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom implements the World Flying Disc Federation (WFDF) rules for ultimate frisbee, adapted for real-time gameplay. This document maps official rules to game mechanics, defines offensive/defensive strategies, player roles, and special situations.

Related docs: [01-disc-physics](01-disc-physics.md), [04-player-controls](04-player-controls.md), [09-ai-system](09-ai-system.md), [14-tutorial-onboarding](14-tutorial-onboarding.md)

## Field Dimensions

```
|<------------ 100m ------------->|
|  End Zone  |   Playing Field   |  End Zone  |
|   (18m)    |      (64m)        |   (18m)    |
|            |                    |            |
+------------+--------------------+------------+
|            |                    |            |  ^
|            |                    |            |  |
|            |        X          |            |  37m
|            |    (brick mark)   |            |  |
|            |                    |            |  v
+------------+--------------------+------------+
```

| Measurement | Value |
|-------------|-------|
| Field length (playing) | 64m |
| End zone depth | 18m |
| Total length | 100m |
| Field width | 37m |
| Brick mark | 18m from each end zone line |
| Center line | 50m from each end |

## Core Rules Implementation

### Scoring
- A point is scored when a player catches the disc in the opposing end zone
- Score check: `player.position.z` within end zone bounds AND `player.has_disc == true` AND `disc.caught_this_frame == true`
- Game to 15 points (default), win by 2 (optional), or timed halves
- **Cap rules**: Hard cap at 17 (or time cap + 2)

### Starting Play — The Pull
1. After a point, the scoring team pulls (throws off) to the other team
2. Both teams line up on their respective end zone lines
3. Pulling team signals readiness -> puller throws
4. **Pull landing rules**:
   - In bounds: Offense picks up disc where it stopped (or catches in air)
   - Out of bounds (sideline): Offense at the point where it crossed the sideline, or brick mark (receiver's choice)
   - Out of bounds (back of end zone): Offense at the front of end zone, or brick mark
   - Dropped catch: Play continues from where the disc lands

### Stall Count
- When a player has the disc, the marker (closest defender) counts to 10
- Count increments every 1.0 seconds (real-time)
- At "Stall 10" (10 seconds), it's a turnover if the disc hasn't been released
- **Fast count**: AI markers have slight timing variations based on difficulty level
- HUD shows stall count prominently (see [13-ui-ux](13-ui-ux.md))

### Turnovers
Change of possession occurs when:
| Event | Trigger | Result |
|-------|---------|--------|
| Drop | Disc hits ground after throw | Play from disc position |
| Out of bounds | Disc or receiver touches OOB | Play from sideline where disc crossed |
| Stall out | Stall count reaches 10 | Play from thrower position |
| Interception | Defender catches disc | Live — play continues |
| Callahan | Interception in the attacking end zone | POINT for intercepting team |
| Hand block | Defender blocks throw at release | Play from block position |
| Foot block | Defender kicks disc during throw | Play from block position |

### Pivoting
- The thrower establishes a pivot foot and cannot move it
- The thrower can pivot 360 degrees around the pivot foot
- In-game: controlled player with disc has a fixed pivot point; player rotates around it
- **Travel violation**: If the pivot moves, turnover (enforced automatically)

### The Mark / Force
- One defender marks the thrower (the "mark")
- The mark establishes a "force" — trying to prevent throws to one side
- **Force home**: Force throws toward home sideline
- **Force away**: Force throws toward away sideline
- **Force straight-up**: No force (allows both sides equally)
- **Mark infractions**: Mark cannot be closer than disc-length (automatic in-game)

### Substitutions
- Substitutions happen after each point (between points only)
- Each team has 7 players on the field (standard)
- Roster can be up to 20 players

## Offensive Strategies

### Vertical Stack ("Vert Stack")
```
         Handler    Handler    Handler
              |
              |
            Cutter 1
              |
            Cutter 2
              |
            Cutter 3
              |
            Cutter 4
```
- Cutters line up vertically downfield, one behind another
- One cutter at a time makes a "cut" (sprint) to get open
- **Open side**: The side the force allows throws to
- **Break side**: The side the force prevents throws to
- Cuts are primarily in/out (toward handler, then away) or deep (long downfield)

**In-game AI behavior**: Cutters queue in the stack; front cutter initiates, others clear space. See [09-ai-system](09-ai-system.md).

### Horizontal Stack ("Horiz Stack")
```
    Handler    Handler    Handler

   Cutter 1   Cutter 2   Cutter 3   Cutter 4
```
- Cutters spread across the field horizontally, creating 1v1 isolation
- Each cutter has a defined lane/area
- Cuts are primarily deep or under (in toward the disc)

**In-game AI behavior**: Cutters maintain spacing; react to disc movement, cut when defender is out of position.

### Side Stack / Split Stack
- Variation where cutters stack on one side of the field
- Creates isolation on the other side for a primary cutter

### Flow Offense
- Less structured; continuous cutting based on disc movement
- "Give and go" — handler throws and immediately cuts
- Used by experienced teams; AI implements at higher difficulty

## Defensive Strategies

### Person-to-Person (Man Defense)
- Each defender assigned to one offensive player
- Mark sets the force; downfield defenders deny their player the disc
- **Positioning**: Stay between your player and the disc (unless fronting deep)

### Force Configurations
| Force | Mark Position | Goal |
|-------|-------------|------|
| Force forehand | Stand on backhand side | Make thrower throw forehand |
| Force backhand | Stand on forehand side | Make thrower throw backhand |
| Force home | Stand between thrower and far sideline | Push offense toward home side |
| Force away | Stand between thrower and near sideline | Push offense toward away side |
| Force middle | Stand straight-up | Prevent hucks, allow short throws |
| No-mark (bait) | Stand off to side | Bait bad throw, recover quickly |

### Zone Defense
```
    Cup (3 players): Wall-Wall-Chase
         [W] [Chase] [W]

    Wings (2 players):
    [Wing]              [Wing]

    Deeps (2 players):
          [Deep]  [Deep]
```
- **Cup**: 3 players form a wall near the disc, preventing short throws
- **Wings**: 2 players cover the sidelines and intermediate space
- **Deeps**: 2 players cover deep downfield threats
- Zone is effective in windy conditions (forces more throws, each affected by wind)

### Clam Defense
- Hybrid: man defense on handlers, zone on cutters
- "Clam" players match up to the nearest cutter in their zone
- Switches frequently; requires communication (AI simulates this)

### Junk Defense
- Mix of man and zone; specific defenders play man on key players
- Other defenders play zones or floater roles
- AI uses this to counter strong individual players

## Player Roles

### Handlers (typically 3)
| Attribute | Priority |
|-----------|----------|
| Throwing skill | Very High |
| Disc IQ | Very High |
| Speed | Medium |
| Endurance | High |
| Catching | High |

Handlers control the disc in the backfield, reset the stall count, and look for break throws. They are the playmakers.

### Cutters (typically 4)
| Attribute | Priority |
|-----------|----------|
| Speed | Very High |
| Agility | Very High |
| Catching | Very High |
| Throwing skill | Medium |
| Endurance | High |

Cutters create separation from defenders through timing, speed, and agility. Their primary job is to get open and catch.

### Hybrid Players
Some players can play both roles effectively. They are versatile and valuable for strategic flexibility.

## Special Situations

### Callahan (Defensive Score)
- A defender intercepts the disc inside the end zone they are attacking
- Immediate point for the defending team
- **Rarity**: Very rare; happens maybe once in 20+ games at competitive level
- In-game: huge "swag" boost for intercepting player, special celebration animation (see [05-stickman-animation](05-stickman-animation.md))

### Greatest (The Greatest Play)
- A player catches the disc while airborne from in-bounds, and throws it back into the field before landing out of bounds
- The throw must be caught by a teammate for a completion
- In-game: requires specific button combination (catch + immediate throw while airborne)
- Contributes heavily to the swag system

### Layout (Diving Bid)
- A player dives horizontally to catch (or block) a disc they otherwise couldn't reach
- **Offensive layout**: Diving catch; if successful, player has the disc but must get up before throwing
- **Defensive layout**: Diving block (D); if defender gets a touch, disc hits ground = turnover
- **Mechanics**: Press layout button within catch range; extends reach by ~1.5m, reduces catch probability
- Player takes 1.5s to get up after a layout

### Sky (Jumping Catch in Traffic)
- Player jumps to catch a high disc above defenders
- Timing-based: jump must be timed with disc arrival
- Height advantage matters (taller players win contested skies more often)

### Huck (Long Throw)
- Full-field throw, usually a deep backhand or forehand
- High risk, high reward
- Affected significantly by wind (see [02-wind-field](02-wind-field.md))
- Requires high power stat and good wind reading

## Game Modes

### Quick Match
- Pick two teams, select venue, play one game
- Adjustable: points to win, time cap, wind, difficulty

### Season Mode
- Full season with league standings, playoffs, finals
- See [11-season-management](11-season-management.md)

### Career Mode
- Create a player, join a team, develop skills
- RPG-lite progression with skill trees
- See [11-season-management](11-season-management.md)

### Practice Mode
- Free practice: throwing, catching, specific plays
- Used by the tutorial system (see [14-tutorial-onboarding](14-tutorial-onboarding.md))

### Online Match
- 1v1 or team vs team online
- Rollback netcode (see [10-netcode](10-netcode.md))

## Point Flow — State Machine

```
PointStart
  |-- TeamSetup (choose O-line / D-line, subs)
  |     \-- LineUp (both teams on end zone lines)
  |           \-- PullReady (puller signals)
  |                 \-- PullFlight (disc in air)
  |                       |-- PullCaught -> LivePlay
  |                       \-- PullLanded -> PickUp -> LivePlay
  |
  \-- LivePlay
        |-- DiscInAir (thrown, in flight)
        |     |-- Caught -> LivePlay (new thrower)
        |     |-- Dropped -> Turnover -> LivePlay (new offense)
        |     |-- OutOfBounds -> Turnover -> LivePlay
        |     |-- Intercepted -> LivePlay (defender becomes offense)
        |     \-- Caught in EndZone -> Score -> PointEnd
        |
        |-- ThrowingPhase (player has disc, stall counting)
        |     |-- Throw -> DiscInAir
        |     |-- StallOut -> Turnover -> LivePlay
        |     \-- Timeout (each team gets 2 per half)
        |
        \-- Stoppage
              |-- Foul call -> discussion -> resume
              |-- Injury timeout
              \-- Spirit timeout
```

## Foul System

Ultimate is self-officiated ("Spirit of the Game"). In FrisKingdom:
- **Fouls are detected automatically** by the game engine (no manual calling needed)
- Contact fouls: defender contacts thrower during throw, defender contacts receiver during catch attempt
- **Contested vs Uncontested**: Resolution is explicit and deterministic (no AI arbitration in online play)
- **Foul frequency**: Adjustable per difficulty level. Higher difficulty = fewer AI fouls.

### Deterministic Foul Resolution Flow
When a foul candidate is detected, the game enters `Stoppage::FoulPending` with the triggering frame ID:
1. Snapshot the pre-contact frame
2. Show a short decision window (e.g., 2 seconds) to each human side: `Accept` or `Contest`
3. Exchange decisions as rollback inputs
4. Resolve via a fixed rules table

| Decisions | Result |
|----------|--------|
| Both accept | Uncontested foul: restore to pre-contact frame, apply foul continuation rules |
| Either contests | Contested foul: restore to pre-contact frame, disc to thrower, stall resumes at `min(previous_stall + 1, 9)` |
| Timeout/no response | Defaults to `Accept` to prevent deadlocks |

```rust
enum FoulResolutionState {
    Live,
    FoulPending { foul_frame: u64, timer_frames: u16 },
    ResolvedUncontested,
    ResolvedContested,
}
```

Online note: foul decisions are part of synchronized input, so both peers resolve the stoppage identically.

## Scoring & Statistics

### In-Game Stats Tracked
| Stat | Description |
|------|------------|
| Goals | Points scored (catches in end zone) |
| Assists | Throw that led to a goal |
| Completions | Successful throws |
| Turnovers | Throws resulting in turnover |
| Ds (Blocks) | Defensive blocks |
| Catches | Successful catches |
| Drops | Failed catch attempts |
| Layouts | Number of diving bids |
| Hucks | Long throws (>30m) |
| Break throws | Throws to the break side |
| Callahans | Defensive end zone interceptions |

### Post-Game Summary
After each game, a detailed stats screen shows individual and team performance (see [13-ui-ux](13-ui-ux.md)).

## Difficulty Scaling

| Difficulty | AI Throwing Accuracy | AI Reaction Time | AI Strategy | Foul Frequency | Stall Speed |
|-----------|---------------------|------------------|------------|----------------|-------------|
| Beginner | 60% | 0.5s delay | Basic vert stack | High | Slow (1.3s/count) |
| Casual | 75% | 0.3s delay | Vert/Horiz stack | Medium | Normal (1.0s/count) |
| Competitive | 88% | 0.1s delay | All formations | Low | Normal |
| Elite | 95% | 0.05s delay | Adaptive | Very Low | Fast (0.9s/count) |
| Spirit | 99% | Near-zero | Reads your playstyle | Minimal | Real-time |
