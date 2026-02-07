# 13 — UI/UX Design

## Overview
FrisKingdom's UI is split between an in-game HUD (minimal, informative, non-intrusive) and menu systems (clean, easy to navigate with gamepad or keyboard). The HUD provides essential game information — score, stall count, wind, stamina — while staying out of the way of gameplay. Menu UI handles team selection, season management, settings, and post-game screens.

Related docs: [04-player-controls](04-player-controls.md), [11-season-management](11-season-management.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)

## Design Principles
1. **Readability first**: All HUD elements must be readable at a glance during fast gameplay
2. **Minimal during play**: Only essential info visible; details on demand
3. **Controller-native**: All menus fully navigable with gamepad (no mouse required)
4. **Consistent visual language**: Shared color/font/spacing across all screens
5. **Accessibility**: Colorblind modes, scalable text, high contrast option

## Typography & Color
- **Heading font**: Bold geometric sans-serif (e.g., Bebas Neue or custom)
- **Body font**: Clean sans-serif (e.g., Inter or custom)
- **Mono font**: For stats/numbers (e.g., JetBrains Mono)
- **Primary color**: Team-dependent (home/away colors)
- **Accent**: Bright orange (#FF6B00) for highlights, selections
- **Background**: Dark charcoal (#1A1A2E) for menus
- **Text**: White (#FFFFFF) primary, light gray (#B0B0B0) secondary

## In-Game HUD

### HUD Layout
```
┌─────────────────────────────────────────────────────────────┐
│  [Home] 8 ─ 7 [Away]           ⏱ 12:34        🌬 NW 8mph  │  ← Top Bar
│                                                              │
│                                                              │
│                                                              │
│                                                              │
│                   [GAME AREA]                                │
│                                                              │
│                                                              │
│                                                              │
│                                                              │
│  [Stall: ███████░░░ 7]    [Power: ████░░░]    [Stamina ▮▮▮] │  ← Bottom Bar
│  ← Player Name    #22 →                                     │
└─────────────────────────────────────────────────────────────┘
```

### Top Bar Elements

#### Scoreboard
- Team names (abbreviated) with team color indicators
- Current score in large, bold numbers
- Point indicator (shows who's on offense)
- Halftime indicator

#### Game Clock
- Match time (count up from 0:00)
- If time cap is active: countdown timer
- Flashes when time cap is approaching

#### Wind Indicator
- Arrow showing wind direction (rotates)
- Speed in mph or m/s (user setting)
- Arrow size/color intensity scales with wind speed
- Compact: just arrow + number

### Bottom Bar Elements

#### Stall Count (visible when player has disc)
- Horizontal bar filling from left to right
- Number overlay (1–10)
- Color: green (1–5), yellow (6–7), orange (8–9), red (10)
- Pulsing animation at 8+
- Disappears when player throws or doesn't have disc

#### Throw Power Gauge (visible when charging throw)
- Appears when holding throw button
- Fills up as power charges
- Sweet spot zone highlighted (70–85%)
- Color: blue (low) → green (sweet spot) → red (over-power)

#### Stamina Bar
- Small segmented bar near the player indicator
- Shows controlled player's remaining stamina
- Color: green (>50%) → yellow (20–50%) → red (<20%)
- Flashes when exhausted

#### Player Indicator
- Controlled player's name and jersey number
- Small position icon (H for handler, C for cutter)
- Current swag level (subtle glow intensity)

### Contextual HUD Elements

#### Throw Prediction Arc
- Faded line showing estimated disc trajectory
- Updates in real-time while aiming
- Visibility based on difficulty setting (always/sometimes/never)

#### Receiver Indicators
- Small icons above potential receivers when charging throw
- Brightness indicates throw quality to that receiver (good pass = bright, risky = dim)
- Currently targeted receiver has a ring

#### Play Call Display
- When a play is called, briefly shows formation diagram (1.5s)
- Fades quickly to not obstruct gameplay
- Corner of screen, semi-transparent

#### Timeout Display
- Remaining timeouts per team (small icons in top bar)
- Timeout screen: full overlay with timer

## Menu System

### Main Menu
```
┌─────────────────────────────────────┐
│                                     │
│         F R I S K I N G D O M       │
│         ─────────────────────       │
│                                     │
│         ▸ Quick Match               │
│           Season Mode               │
│           Career Mode               │
│           Online Play               │
│           Practice                  │
│           Settings                  │
│           Quit                      │
│                                     │
│    [A] Select   [B] Back            │
└─────────────────────────────────────┘
```

- Animated stickman characters in background (doing tricks, catching discs)
- Menu items highlight on selection with slide animation
- Smooth transitions between screens

### Team Select Screen
```
┌─────────────────────────────────────────────────────┐
│  TEAM SELECT                                         │
│                                                      │
│  ┌──────────────┐          ┌──────────────┐          │
│  │  [Team Logo]  │   VS    │  [Team Logo]  │         │
│  │  THUNDERBOLTS │         │  WAVE RIDERS  │         │
│  │  ████████████ │         │  ████████████ │         │
│  │              │          │              │          │
│  │  Record: 8-6 │          │  Record: 10-4│          │
│  │  Style: Vert  │         │  Style: Horiz│          │
│  └──────────────┘          └──────────────┘          │
│                                                      │
│  ◄ Prev Team    Next Team ►                          │
│                                                      │
│  [A] Confirm   [Y] View Roster   [B] Back            │
└─────────────────────────────────────────────────────┘
```

### Venue Select Screen
- Visual preview of the selected venue (rotating camera shot)
- Weather and time-of-day selectors
- Wind setting (off / light / medium / heavy / random)

### Settings Menu
```
Settings
├── Gameplay
│   ├── Difficulty: [Beginner | Casual | Competitive | Elite | Spirit]
│   ├── Auto-switch: [Full Auto | Semi-Auto | Manual]
│   ├── Throw prediction: [Always | Only When Charging | Never]
│   ├── Stall count speed: [Slow | Normal | Fast]
│   └── Points to win: [11 | 13 | 15 | Custom]
├── Display
│   ├── Resolution
│   ├── Fullscreen / Windowed
│   ├── Quality Preset: [Low | Medium | High | Ultra]
│   ├── VSync
│   └── HUD Scale: [Small | Medium | Large]
├── Audio
│   ├── Master Volume
│   ├── SFX Volume
│   ├── Music Volume
│   ├── Crowd Volume
│   ├── Commentary Volume
│   └── Wind Units: [mph | m/s | km/h]
├── Controls
│   ├── Button Remapping
│   ├── Stick Sensitivity
│   ├── Vibration: [On | Off]
│   └── Controller Layout Display
├── Accessibility
│   ├── Colorblind Mode: [Off | Protanopia | Deuteranopia | Tritanopia]
│   ├── High Contrast HUD: [On | Off]
│   ├── Text Size: [Normal | Large | Extra Large]
│   ├── Screen Reader: [On | Off]
│   ├── Auto-aim Assist: [Off | Light | Strong]
│   └── Throw Power Assist: [Off | On]
└── Online
    ├── Display Name
    ├── Region
    └── Network Stats Display: [On | Off]
```

## Post-Game Screen

### Score Summary
```
┌─────────────────────────────────────────────────────┐
│  GAME OVER                                           │
│                                                      │
│  THUNDERBOLTS  15 ── 12  WAVE RIDERS                │
│                                                      │
│  ┌─ Point Timeline ─────────────────────────────┐   │
│  │ T W T T W W T T T W T W W T T T W T T T T T │   │
│  │ (each letter = who scored that point)         │   │
│  └───────────────────────────────────────────────┘   │
│                                                      │
│  MVP: Player Name (#7) — 4 Goals, 3 Assists, 2 Ds  │
│                                                      │
│  [A] View Stats   [Y] Replay   [X] Rematch   [B] Exit│
└─────────────────────────────────────────────────────┘
```

### Detailed Stats Tab
```
┌────────────────────────────────────────────────────────────┐
│  GAME STATS                                                 │
│                                                             │
│  THUNDERBOLTS                    WAVE RIDERS                │
│  ──────────                      ──────────                 │
│  # Name        G  A  C  TO  D   # Name        G  A  C TO D │
│  7 J.Smith     4  3  12  1  2   3 M.Chen      3  2  10  2 1│
│  22 K.Jones    3  1   8  0  1  14 A.Patel     2  3   9  1 0│
│  11 R.Davis    2  2  10  1  0   8 L.Kim       2  1   7  2 2│
│  ...                             ...                        │
│                                                             │
│  Team Totals:                                               │
│  Completions: 45/52 (87%)        Completions: 38/49 (78%)  │
│  Hucks: 5/7                      Hucks: 3/6                 │
│  Break throws: 8/12              Break throws: 5/10         │
│  Layouts: 6                      Layouts: 4                 │
│                                                             │
│  [LB] Team Stats  [RB] Individual Stats  [B] Back          │
└────────────────────────────────────────────────────────────┘
```

## Season Management UI

### Season Hub
```
┌─────────────────────────────────────────────────────┐
│  SEASON HUB — Week 8 of 14                          │
│                                                      │
│  ┌─ Your Team ──────────┐  ┌─ Next Game ──────────┐ │
│  │ THUNDERBOLTS (6-1)   │  │ vs WAVE RIDERS       │ │
│  │ 1st in Division A    │  │ Saturday, Week 8     │ │
│  │ Roster: 18/20        │  │ @ Sunset Park        │ │
│  └──────────────────────┘  │ Weather: Partly Cloudy│ │
│                             └──────────────────────┘ │
│                                                      │
│  ▸ Play Next Game                                    │
│    Manage Roster                                     │
│    Training                                          │
│    Scout Opponent                                    │
│    League Standings                                  │
│    Season Schedule                                   │
│    Stats Leaders                                     │
│                                                      │
│  [A] Select   [B] Back to Main Menu                  │
└─────────────────────────────────────────────────────┘
```

### Roster Management
```
┌─────────────────────────────────────────────────────┐
│  ROSTER MANAGEMENT                                   │
│                                                      │
│  O-LINE (7/7)          D-LINE (7/7)                  │
│  ─────────             ─────────                     │
│  #7  J.Smith  H  89    #3  R.Chen   C  85           │
│  #22 K.Jones  C  84    #14 A.Patel  H  82           │
│  #11 R.Davis  H  81    #8  L.Kim    C  88           │
│  #5  T.Brown  C  78    #19 S.Lopez  H  79           │
│  #33 M.Lee    C  82    #2  D.Clark  C  83           │
│  #15 J.White  H  77    #10 K.Hall   C  80           │
│  #9  P.Green  C  75    #21 N.Taylor C  76           │
│                                                      │
│  BENCH                                               │
│  #4  A.Adams  H  72    #17 B.Hill   C  71           │
│  #30 C.King   C  68    #12 E.Scott  H  66           │
│                                                      │
│  [A] Move Player  [X] View Details  [Y] Auto-Set     │
│  [LB/RB] Switch O/D Line  [B] Back                   │
└─────────────────────────────────────────────────────┘
```

### Training Screen
```
┌─────────────────────────────────────────────────────┐
│  TRAINING — 3 Sessions Available                     │
│                                                      │
│  Session 1: [ Sprint Drills      ▼ ]                 │
│  Session 2: [ Throwing Practice  ▼ ]                 │
│  Session 3: [ Rest               ▼ ]                 │
│                                                      │
│  ┌─ Team Impact Preview ────────────────────┐        │
│  │  Speed:    ████████████░░ +0.02           │        │
│  │  Accuracy: ██████████████░ +0.02          │        │
│  │  Fitness:  █████████░░░░░ -0.01           │        │
│  │  Morale:   ██████████████ +0.05           │        │
│  └───────────────────────────────────────────┘       │
│                                                      │
│  ⚠ Player #33 M.Lee has low fitness (45%)            │
│    Consider assigning rest                           │
│                                                      │
│  [A] Confirm Training  [B] Back                      │
└─────────────────────────────────────────────────────┘
```

## Career Mode UI

### Career Dashboard
```
┌─────────────────────────────────────────────────────┐
│  CAREER — Year 2, Week 12                            │
│                                                      │
│  ┌─ Your Player ────────┐  ┌─ Season Stats ────────┐│
│  │ "Flash" Thompson #7  │  │ Games: 10            ││
│  │ Handler | Age 19     │  │ Goals: 8  Assists: 12 ││
│  │ Team: THUNDERBOLTS   │  │ Ds: 5   Layouts: 3   ││
│  │ Rep: ████░░░ (0.45)  │  │ Completion: 84%      ││
│  │ SP Available: 12     │  │ Turnovers: 7         ││
│  └──────────────────────┘  └──────────────────────┘ │
│                                                      │
│  ▸ Play Next Game                                    │
│    Skill Tree                                        │
│    Player Stats                                      │
│    Team Info                                         │
│    League Standings                                  │
│                                                      │
│  [A] Select   [B] Back                               │
└─────────────────────────────────────────────────────┘
```

### Skill Tree UI
- Visual tree layout showing unlocked and available nodes
- Locked nodes are grayed out with SP cost displayed
- Unlocked nodes glow with team color
- Current path highlighted
- Preview pane shows node details on hover

## Replay UI

### Replay Controls
```
┌─────────────────────────────────────────────────────┐
│  REPLAY                                              │
│                                                      │
│              [Game View with Free Camera]             │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │  ◄◄  ◄  ▶/❚❚  ►  ►►  │ 0.25x 0.5x [1x] 2x │   │
│  │  ═══════════■═══════════════════════════════  │   │
│  │  2:34 / 15:42                                 │   │
│  └──────────────────────────────────────────────┘    │
│                                                      │
│  [LS] Pan  [RS] Orbit  [Y] Disc Trail  [B] Exit     │
└─────────────────────────────────────────────────────┘
```

## Notifications & Toasts

In-game notifications for events that don't need full-screen attention:

| Event | Notification | Position | Duration |
|-------|-------------|----------|----------|
| Score | "GOAL! #7 J.Smith" + team color flash | Center | 2.5s |
| Turnover | "TURNOVER — Drop" | Top center | 1.5s |
| Callahan | "CALLAHAN! #3 R.Chen" + special animation | Full screen flash | 3.0s |
| Greatest | "THE GREATEST! #22 K.Jones" | Full screen flash | 3.0s |
| Half time | "HALFTIME — 8-7" | Center | 3.0s |
| Timeout | "TIMEOUT — [Team]" | Center | 2.0s |
| Substitution | "[Player] in for [Player]" | Bottom corner | 1.5s |

## Accessibility Features

### Colorblind Modes
- **Protanopia** (red-blind): Replace red with blue; team differentiation via pattern/brightness
- **Deuteranopia** (green-blind): Adjust green channel; use shape indicators for team
- **Tritanopia** (blue-yellow blind): Remap blue/yellow; use contrast-based indicators
- All modes add shape-based team indicators (triangles vs circles) in addition to color

### Scalable UI
- Three text size options: Normal (16px base), Large (20px), Extra Large (24px)
- HUD scale: Small/Medium/Large (affects all HUD elements proportionally)
- Menu text and buttons scale with text size setting

### Screen Reader Support
- All menu items have text labels
- Navigational audio cues (different tones for different menu levels)
- Score and game state announced audibly on state changes

### Motor Accessibility
- Fully remappable controls
- Auto-aim assist (light and strong options)
- Throw power assist (auto-triggers at sweet spot)
- One-handed gamepad layout option
- Adjustable input buffer window (3–10 frames)

## Implementation Notes

### UI Tech Stack
- **In-game HUD**: Custom Bevy UI nodes (for performance and style control)
- **Menus**: Custom Bevy UI with focus management for gamepad navigation
- **Debug/Dev UI**: `bevy_egui` for runtime tweaking (not shipped in release)

### Performance
- UI rendering budget: <0.3ms per frame
- Minimal texture usage (most UI is vector/procedural)
- Font rendering: SDF-based for crisp text at all sizes
- Animations: driven by Bevy UI style transitions
