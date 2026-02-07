# 04 — Input & Controls

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
FrisKingdom supports gamepad (primary) and keyboard+mouse as input methods. Controls are context-sensitive, changing based on whether the player is on offense with the disc, offense without the disc, or defense. The throwing interface is the most nuanced — it uses an analog power/aim system inspired by golf games.

Related docs: [01-disc-physics](01-disc-physics.md), [05-stickman-animation](05-stickman-animation.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md), [13-ui-ux](13-ui-ux.md)

## Input Contexts

The game has three primary input contexts based on the player's state:

| Context | Active When | Primary Actions |
|---------|-------------|-----------------|
| **WithDisc** | Controlled player has the disc | Throw, pivot, fake, call play |
| **OffenseNoCut** | On offense, don't have disc | Move, cut, call for disc, switch player |
| **Defense** | On defense | Move, mark, bid for block, switch player |
| **Pull** | Pulling at start of point | Aim, power, throw pull |
| **Menu** | In menus/pause | Navigate, select |

## Gamepad Layout (Xbox / PlayStation)

### Universal Controls (All Contexts)
| Button | Xbox | PS | Action |
|--------|------|-----|--------|
| Left Stick | LS | LS | Move player |
| Right Stick | RS | RS | Camera look / aim |
| LB / L1 | LB | L1 | Sprint (hold) |
| Start | Menu | Options | Pause menu |
| Select | View | Touch | Scoreboard / stats |

### WithDisc Context (Offense, Holding Disc)
| Button | Xbox | PS | Action |
|--------|------|-----|--------|
| Left Stick | LS | LS | Pivot direction |
| Right Stick | RS | RS | Aim throw direction |
| RT / R2 (hold) | RT | R2 | Charge throw power |
| RT / R2 (release) | RT | R2 | Release throw |
| A / X | A | Cross | Backhand throw (quick) |
| X / Square | X | Square | Forehand throw (quick) |
| Y / Triangle | Y | Triangle | Overhead throw (hammer/scoober) |
| B / Circle | B | Circle | Fake throw / pump fake |
| LT / L2 | LT | L2 | Throw modifier (hold for specialty throws) |
| LT + A | LT+A | L2+Cross | Push pass |
| LT + X | LT+X | L2+Square | Blade |
| LT + Y | LT+Y | L2+Triangle | Thumber / Chicken wing |
| D-Pad Up | D-Up | D-Up | Call play: Vert stack |
| D-Pad Down | D-Down | D-Down | Call play: Horiz stack |
| D-Pad Left | D-Left | D-Left | Call play: Isolation |
| D-Pad Right | D-Right | D-Right | Call timeout |
| RB / R1 | RB | R1 | Cycle throw target (next receiver) |

### OffenseNoCut Context (No Disc, On Offense)
| Button | Xbox | PS | Action |
|--------|------|-----|--------|
| Left Stick | LS | LS | Move/run direction |
| Right Stick | RS | RS | Camera |
| A / X | A | Cross | Cut in (toward disc) |
| X / Square | X | Square | Cut deep (away from disc) |
| Y / Triangle | Y | Triangle | Call for disc (signal) |
| B / Circle | B | Circle | Clear out / reset position |
| RT / R2 | RT | R2 | Layout / dive (when near disc) |
| RB / R1 | RB | R1 | Switch to nearest teammate |
| LB / L1 | LB | L1 | Sprint |

### Defense Context
| Button | Xbox | PS | Action |
|--------|------|-----|--------|
| Left Stick | LS | LS | Move/run direction |
| Right Stick | RS | RS | Camera |
| A / X | A | Cross | Tighten mark / close gap |
| X / Square | X | Square | Hand block (when marking) |
| Y / Triangle | Y | Triangle | Jump / sky attempt |
| B / Circle | B | Circle | Bid (defensive layout) |
| RT / R2 | RT | R2 | Secondary sprint (same as LB/L1, remappable) |
| RB / R1 | RB | R1 | Switch to nearest defender |
| LT / L2 | LT | L2 | Face-guard (mirror opponent) |
| D-Pad Up | D-Up | D-Up | Force forehand |
| D-Pad Down | D-Down | D-Down | Force backhand |
| D-Pad Left | D-Left | D-Left | Call zone defense |
| D-Pad Right | D-Right | D-Right | Call man defense |

### Pull Context
| Button | Xbox | PS | Action |
|--------|------|-----|--------|
| Left Stick | LS | LS | Aim pull direction |
| Right Stick (vertical) | RS | RS | Adjust pull angle (high/low) |
| RT / R2 (hold) | RT | R2 | Charge pull power |
| A / X | A | Cross | Throw pull |

## Keyboard + Mouse Layout

### Universal
| Key | Action |
|-----|--------|
| WASD | Move player |
| Mouse movement | Aim / camera |
| Shift (hold) | Sprint |
| Escape | Pause menu |
| Tab | Scoreboard |
| Space | Context action (catch/jump/layout) |

### WithDisc (Offense, Holding)
| Key | Action |
|-----|--------|
| WASD | Pivot direction |
| Mouse aim | Throw direction |
| Left Click (hold + release) | Charge + release throw |
| Right Click | Fake / pump fake |
| 1 | Backhand |
| 2 | Forehand |
| 3 | Hammer |
| 4 | Scoober |
| 5 | Blade |
| Q | Push pass |
| E | Thumber |
| Mouse wheel | Cycle throw target |
| F1–F4 | Call plays |

### OffenseNoCut
| Key | Action |
|-----|--------|
| WASD | Move |
| Space | Layout / dive |
| E | Call for disc |
| R | Cut in |
| F | Cut deep |
| C | Clear out |
| Tab (quick) | Switch player |

### Defense
| Key | Action |
|-----|--------|
| WASD | Move |
| Space | Jump / sky |
| E | Hand block (when marking) |
| R | Bid / defensive layout |
| F | Tighten mark |
| Q | Face-guard toggle |
| Tab (quick) | Switch player |
| 1–4 | Force calls |

## Throwing Interface — Detailed

The throwing system is the core skill mechanic. It's designed to be easy to learn but hard to master.

### Power Gauge
```
Hold RT ──────────────────────> Release RT
  |                                |
  0%    25%    50%    75%   100%  OVER
  [░░░░░░░░░░░░░░░░░░░░░░░░░░░]
           SWEET SPOT (70-85%)
```

- **Hold RT/R2** to charge throw power
- **Release** to throw at current power level
- Power fills over ~1.2 seconds from 0% to 100%
- **Sweet spot** (70–85%): Bonus accuracy, disc releases cleanly
- **Over-power** (>95%): Release becomes less accurate, wobble added
- **Under-power** (<30%): Short, floaty throw that hangs (easy D)

### Aim System
- **Right stick (gamepad)** / **Mouse** controls throw aim
- Horizontal: Left-right direction of the throw relative to facing
- Vertical: Nose angle — up = nose up (more loft), down = nose down (more speed)
- A guide arrow shows predicted throw direction (adjustable visibility in settings)

### Release Angle (Hyzer/Anhyzer)
- **Left stick horizontal** while charging adjusts the disc's tilt at release:
  - Left = hyzer (disc tilted inward, curves left for RHBH)
  - Right = anhyzer (disc tilted outward, curves right for RHBH)
  - Neutral = flat release

### IO/OI Modification
- Combining aim direction with release angle:
  - **Inside-out (IO)**: Aiming one way while releasing anhyzer — disc holds an S-curve
  - **Outside-in (OI)**: Aiming one way while releasing hyzer — sharp curve

### Quick Throw vs Charged Throw
- **Quick throw** (tap A/X/Y without charging): Releases at ~50% power immediately with the selected throw type. Fast but less controllable.
- **Charged throw**: Hold RT then release. More power range and precision, but slower.

### Throw Type Selection
1. Default throw changes based on last used or control scheme preference
2. Buttons select throw type immediately if pressed during charge
3. Hold LT modifier for specialty throws (blade, push pass, thumber, chicken wing)

### Predicted Flight Path
When charging a throw, a faded arc shows the approximate disc trajectory (affected by wind):
- Updates in real-time as aim/power/angle change
- Fidelity based on difficulty level (easier = more visible, harder = less)
- Can be turned off in settings for realism

## Player Switching

### Auto-Switch
By default, the game auto-switches to the most relevant player:
- On offense: Switches to the receiver nearest the thrown disc during flight
- On defense: Switches to the defender nearest the disc or the most relevant threat
- Player can override auto-switch with RB/R1 or Tab

### Manual Switch
- **Tap RB/R1**: Cycle to the next nearest player
- **Hold RB/R1 + Left Stick**: Direct-switch to a specific player (icon selector appears)
- Setting: Auto-switch can be set to "full auto", "semi-auto" (only on turnover/catch), or "manual only"

## Sprint & Stamina

- **Sprint** (hold LB/Shift): Player runs at ~1.5x normal speed
- **Stamina bar**: Each player has a stamina gauge that depletes while sprinting
  - Regenerates while walking or standing
  - Regeneration is faster when the player isn't involved in the play
  - Exhaustion: At 0 stamina, player speed drops to 0.7x and takes 3s to begin recovering
- Stamina ties into the season management system — fitter players have larger stamina pools (see [11-season-management](11-season-management.md))

## Pivoting

When the controlled player has the disc:
- Left stick rotates the player around the pivot foot
- The pivot foot position is locked on catch
- 360 degree rotation available
- Animation: Procedural IK keeps pivot foot planted (see [05-stickman-animation](05-stickman-animation.md))
- If the player moves beyond pivot range -> travel violation -> turnover

## Catching

Catching is mostly automatic but with skill-based elements:
- If the controlled player is near a catchable disc, pressing nothing = auto-catch attempt
- **Layout catch** (RT/Space near a disc at edge of range): Player dives for the disc
  - Extends reach by ~1.5m
  - Catch probability reduced (see [01-disc-physics](01-disc-physics.md))
  - Triggers layout animation (see [05-stickman-animation](05-stickman-animation.md))
- **Sky catch** (Y/Space when disc is high): Player jumps to contest
  - Height stat + timing determines success
  - Contested catches use the catch probability formula

## Pulling

The pull (kickoff) has its own dedicated interface:
1. Player can move left/right on the end zone line
2. Aim direction with LS
3. Adjust loft angle with RS vertical
4. Hold RT for power gauge (same system as throwing)
5. Press A to release the pull
6. Backhand pull is default; press X for forehand pull
7. Goal: Land the disc deep in the opponent's end zone or at a specific target

## Accessibility

- All controls are remappable
- Auto-aim assist levels (off / light / strong)
- Throw power assist (auto sweet-spot mode)
- One-handed controller layouts available
- Colorblind modes for team colors and UI (see [13-ui-ux](13-ui-ux.md))
- Adjustable stall count speed for casual play

## Input in Replay Mode

During replay review:
| Control | Action |
|---------|--------|
| LS / WASD | Pan camera |
| RS / Mouse | Orbit camera |
| RT / Shift | Fast forward |
| LT / Ctrl | Rewind |
| A / Space | Play/pause |
| D-Pad L/R | Frame step |
| Y / R | Toggle disc flight path overlay |
| B / Esc | Exit replay |

## Implementation Notes

### Input Buffering
- All inputs are buffered for 3 frames (~50ms) to prevent dropped inputs
- Throw release timing is recorded at the exact frame for determinism (see [10-netcode](10-netcode.md))
- In rollback netcode, only the button states are transmitted; analog values are quantized to 8-bit resolution

### Bevy Input Resource
```rust
#[derive(Resource)]
pub struct PlayerInput {
    pub move_dir: Vec2,          // left stick, normalized
    pub aim_dir: Vec2,           // right stick, normalized
    pub throw_power: f32,        // 0–1, from RT hold duration
    pub throw_type: Option<ThrowType>,
    pub sprint: bool,
    pub layout_bid: bool,
    pub switch_player: bool,
    pub call_play: Option<PlayCall>,
    pub context: InputContext,
}

#[derive(Clone, Copy)]
pub enum InputContext {
    WithDisc,
    OffenseNoCut,
    Defense,
    Pull,
    Menu,
    Replay,
}
```
