# 11 — Season & Career Mode

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom features two interconnected management modes: **Season Mode** (manage a team through a league season) and **Career Mode** (create a player and develop them through an RPG-lite progression system). Both modes add longevity beyond single matches and give context to on-field gameplay.

Related docs: [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md), [09-ai-system](09-ai-system.md), [13-ui-ux](13-ui-ux.md)

## Season Mode

### League Structure
```
Division A (8 teams)          Division B (8 teams)
├── Regular Season            ├── Regular Season
│   (14 games: play each      │   (14 games: play each
│    division team 2×)         │    division team 2×)
├── Division Standings         ├── Division Standings
└── Top 4 → Playoffs          └── Top 4 → Playoffs

Playoffs:
  Quarterfinals (8 teams) → Semifinals (4) → Finals (2) → Champion
  Single elimination, higher seed hosts
```

### Season Calendar
| Phase | Duration (games) | Description |
|-------|-----------------|-------------|
| Preseason | 2 exhibition games | Practice, set roster, no standings impact |
| Regular Season | 14 games | Division play, standings determined |
| Playoffs | 3 rounds (up to 3 games) | Single elimination bracket |
| Offseason | Management phase | Trades, draft, training |

### Team Management

#### Roster
- **Active roster**: 20 players maximum
- **Starting 7**: 7 players on field per point
- **O-line / D-line**: Separate lineups for offense and defense (with overlap allowed)
- **Captain**: One player designated as captain (bonus to team swag, special celebrations)

```rust
pub struct TeamRoster {
    pub team_name: String,
    pub team_colors: TeamColors,
    pub home_venue: VenueType,
    pub players: Vec<RosterPlayer>,
    pub o_line: [PlayerId; 7],
    pub d_line: [PlayerId; 7],
    pub captain: PlayerId,
}

pub struct RosterPlayer {
    pub player_id: PlayerId,
    pub name: String,
    pub stats: PlayerStats,
    pub role: PlayerRole,
    pub age: u8,           // 18–35
    pub experience: u32,   // career games played
    pub morale: f32,       // 0–1
    pub fitness: f32,      // 0–1, affects stamina
    pub injury_status: InjuryStatus,
    pub contract: Contract,
}
```

#### Substitution Strategy
Between points, the player can:
1. Quick sub: Swap tired players for fresh legs
2. Custom lines: Set specific O-line and D-line
3. Auto-sub: Let the AI manage substitutions based on stamina/matchups

### Standings & Statistics

#### League Table
| Col | Description |
|-----|------------|
| Team | Team name |
| W | Wins |
| L | Losses |
| PF | Points For (total goals scored) |
| PA | Points Against (total goals conceded) |
| PD | Point Differential (PF - PA) |
| Streak | Current win/loss streak |

Tiebreakers: 1) Head-to-head record, 2) Point differential, 3) Points for

#### Season Stats Leaders
Track and display league leaders in:
- Goals, Assists, Ds, Completions, Completion %, Hucks completed, Layouts, Callahans

### Between Games
Between season games, the player can:
1. Review last game stats and highlights
2. Adjust roster and starting lines
3. Scout upcoming opponent (see their stats, tendencies)
4. Manage training (see Training section)

## Training System

Between games, players can be assigned training focuses:

### Training Activities
| Activity | Primary Stat Boost | Secondary | Duration |
|----------|-------------------|-----------|----------|
| Sprint drills | Speed +0.02 | Acceleration +0.01 | 1 session |
| Distance running | Endurance +0.02 | Speed +0.01 | 1 session |
| Throwing practice | Throwing accuracy +0.02 | Throwing power +0.01 | 1 session |
| Huck practice | Throwing power +0.02 | Throwing accuracy +0.01 | 1 session |
| Catching drills | Catching +0.02 | Agility +0.01 | 1 session |
| Agility course | Agility +0.02 | Speed +0.01 | 1 session |
| Jump training | Jumping +0.02 | — | 1 session |
| Film study | Disc IQ +0.03 | — | 1 session |
| Scrimmage | All stats +0.005 | Fitness +0.02 | 1 session |
| Rest | — | Fitness +0.05, Morale +0.05 | 1 session |

**Training slots**: 3 sessions between games (Mon/Wed/Fri equivalent)
**Fatigue**: Training reduces fitness; overtraining risks injury. Balance training with rest.

### Stat Caps & Decay
- Stats cap at 1.0 (maximum human ability)
- Stats decay by 0.005 per game if not trained (natural skill regression)
- Older players (30+) decay faster
- Younger players (18–22) gain from training faster

## Career Mode (RPG-Lite)

### Create-a-Player
```rust
pub struct CareerPlayer {
    pub name: String,
    pub appearance: PlayerAppearance,
    pub position: PlayerRole,
    pub age: u8,                 // starts at 18
    pub stats: PlayerStats,      // starts low
    pub skill_points: u32,       // earned through play
    pub reputation: f32,         // 0–1, affects team offers
    pub career_stats: CareerStats,
    pub skill_tree: SkillTree,
    pub trait_slots: Vec<PlayerTrait>,
}
```

### Starting Stats
New career players start with low stats, varying by chosen position:

| Stat | Handler Start | Cutter Start | Hybrid Start |
|------|-------------|-------------|-------------|
| Speed | 0.40 | 0.55 | 0.48 |
| Acceleration | 0.40 | 0.50 | 0.45 |
| Throwing Power | 0.50 | 0.30 | 0.40 |
| Throwing Accuracy | 0.45 | 0.25 | 0.35 |
| Catching | 0.45 | 0.50 | 0.48 |
| Agility | 0.40 | 0.50 | 0.45 |
| Jumping | 0.35 | 0.45 | 0.40 |
| Endurance | 0.50 | 0.50 | 0.50 |
| Disc IQ | 0.40 | 0.30 | 0.35 |

### Skill Tree
The skill tree offers specialization paths:

```
                    ┌─── Huck Master ─── Sky Huck
                    │
Handler Path ───────┼─── Break King ─── Around Back
                    │
                    └─── Reset God ─── Endurance Throws

                    ┌─── Speed Demon ─── Afterburner
                    │
Cutter Path ────────┼─── Sky High ─── Posterizer
                    │
                    └─── Layout Artist ─── No Fear

                    ┌─── Iron Man ─── Never Tired
                    │
Universal Path ─────┼─── High IQ ─── Play Reader
                    │
                    └─── Swag Lord ─── Crowd Favorite
```

### Skill Tree Nodes (Examples)
| Node | Path | Cost | Effect |
|------|------|------|--------|
| Huck Master | Handler | 10 SP | +0.1 throwing power on hucks, +5% huck accuracy |
| Sky Huck | Handler | 20 SP | Unlock huck fake animation; hucks gain height |
| Break King | Handler | 10 SP | +0.1 throwing accuracy on break throws |
| Around Back | Handler | 20 SP | Unlock chicken wing throw with reduced accuracy penalty |
| Reset God | Handler | 10 SP | +0.05 speed when pivoting with disc |
| Speed Demon | Cutter | 10 SP | +0.1 top sprint speed |
| Afterburner | Cutter | 20 SP | Sprint speed doesn't decrease when stamina is low |
| Sky High | Cutter | 10 SP | +0.15 jump height |
| Posterizer | Cutter | 20 SP | Win contested catches 80% of the time (from 50%) |
| Layout Artist | Cutter | 10 SP | +0.3m layout range, +10% layout catch rate |
| No Fear | Cutter | 20 SP | Layouts don't reduce stamina |
| Iron Man | Universal | 10 SP | +20% stamina pool |
| Never Tired | Universal | 20 SP | Stamina regenerates 2× faster |
| High IQ | Universal | 10 SP | Teammate AI plays smarter when you're on field |
| Play Reader | Universal | 20 SP | See opponent's intended cuts briefly (visual indicator) |
| Swag Lord | Universal | 10 SP | Swag meter starts higher and decays slower |
| Crowd Favorite | Universal | 20 SP | Crowd cheers boost stats slightly; celebrations are flashier |

### Earning Skill Points
| Action | SP Earned |
|--------|----------|
| Play a game | 3 |
| Win a game | +2 bonus |
| Score a goal | +1 |
| Get an assist | +1 |
| Record a D (block) | +1 |
| Layout catch | +1 |
| Callahan | +5 |
| Greatest | +5 |
| MVP of the game | +3 |
| Complete a season | +10 |
| Win championship | +20 |

### Player Traits
Traits are passive bonuses earned through gameplay milestones:

| Trait | Unlock Condition | Effect |
|-------|-----------------|--------|
| Clutch | Score 3 game-winning goals | +10% stats in final point |
| Hot Streak | 5 goals in one game | After 2 consecutive goals, +15% speed |
| Lockdown | 10 games with 3+ Ds | Opponents throw 10% less accurately when you're marking |
| Captain | Reach reputation 0.8 | Teammates get +5% morale when you're on field |
| Veteran | Play 100 career games | Reduce all stat decay by 50% |
| Underdog | Win 3 games as 10+ point underdog | Start each game with full swag |

### Career Progression

```
Year 1: Join a low-tier team as a bench player
  → Play 3–5 games, earn spot in rotation
  → Season ends, attract offers from mid-tier teams

Year 2–3: Starting player on a competitive team
  → Develop stats, earn skill points
  → Aim for league stats leaders

Year 4+: Star player, possible captain
  → Lead team to playoffs/championship
  → Reputation unlocks better team offers

Year 8+: Veteran
  → Stats start declining
  → Mentor role: boost young teammates
  → Retirement decision
```

### Reputation System
Reputation (0.0–1.0) determines which teams make offers and how much playing time you get:

| Reputation | Level | Effect |
|-----------|-------|--------|
| 0.0–0.2 | Unknown | Only low-tier teams; bench role |
| 0.2–0.4 | Rising | Mid-tier offers; rotation player |
| 0.4–0.6 | Established | Starter on mid-tier; bench on top-tier |
| 0.6–0.8 | Star | Starter on top-tier teams; fan favorite |
| 0.8–1.0 | Legend | Any team wants you; captain offers; legacy |

Reputation increases from: winning, personal stats, big plays (Callahan, Greatest), team success.
Reputation decreases from: poor play, turnovers, losing, sitting out (injury).

## Progression & Unlocks

### Unlockable Content
| Category | Examples | Unlock Method |
|----------|---------|---------------|
| Venues | Rooftop, Forest Clearing | Win games at each venue type |
| Celebrations | Moonwalk, The Worm, team celebrations | Reach swag milestones in career |
| Accessories | Special headbands, wristbands, sunglasses | Career achievements |
| Team Colors | Alternate color schemes | Season wins |
| Camera Angles | Sky cam, player-locked | Career progression |
| Commentary Lines | Additional commentary variations | Play X total games |
| Historical Teams | Famous real-world-inspired teams | Complete seasons |

### Achievement System
Achievements track career milestones:
- "First Flight" — Complete your first game
- "Century Club" — Score 100 career goals
- "Disc Wizard" — Complete 1000 career throws
- "Sky's the Limit" — Win 10 contested sky catches in one season
- "Layout Legend" — 50 career layout catches
- "Spirit of the Game" — Complete a season with 0 fouls committed
- "Dynasty" — Win 3 consecutive championships
- "GOAT" — Max out all stats in career mode

## Save System

### Save Data Structure
```rust
pub struct SaveData {
    pub schema_version: u32,      // increment on any breaking save format change
    pub build_version: String,    // game build that wrote this save
    pub saved_at_unix: i64,
    pub career: Option<CareerSaveData>,
    pub season: Option<SeasonSaveData>,
    pub unlocks: UnlockData,
    pub settings: GameSettings,
    pub stats: GlobalStats,
}

pub struct CareerSaveData {
    pub player: CareerPlayer,
    pub current_team: TeamId,
    pub current_season: u32,
    pub career_history: Vec<SeasonRecord>,
}

pub struct SeasonSaveData {
    pub league: LeagueState,
    pub schedule: Vec<ScheduledGame>,
    pub current_week: u32,
    pub roster: TeamRoster,
    pub training_log: Vec<TrainingSession>,
}
```

Save files are serialized with RON format and stored locally. Career mode auto-saves after each game.

### Save Compatibility & Migration Policy
- Save loading is version-gated by `schema_version` (not app version string)
- Backward compatibility target: latest version supports loading at least previous 3 schema versions
- Migrations run sequentially (`vN -> vN+1`) with deterministic transforms and tests
- Before writing migrated data, keep a backup (`.bak`) of the original save
- On migration failure: keep original file untouched, surface recoverable error, and continue to main menu

```rust
pub fn load_save(path: &Path) -> Result<SaveData, SaveError> {
    let raw = std::fs::read_to_string(path)?;
    let mut save: SaveData = ron::from_str(&raw)?;
    while save.schema_version < CURRENT_SCHEMA_VERSION {
        save = migrate_one_version(save)?;
    }
    Ok(save)
}
```
