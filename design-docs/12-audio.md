# 12 — Audio Design

## Overview
FrisKingdom's audio is primarily procedural — sound effects are synthesized in real-time rather than playing pre-recorded samples. This approach reduces asset size, allows infinite variation, and enables dynamic responses to game state. The audio system covers disc sounds, player sounds, crowd reactions, abstract commentary, ambient environments, and procedural music.

Related docs: [06-field-and-environment](06-field-and-environment.md), [07-rendering](07-rendering.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)

## Audio Architecture

```
Audio Engine (bevy_kira_audio)
├── Procedural SFX Generator
│   ├── Disc Sounds
│   ├── Player Sounds
│   └── Impact Sounds
├── Crowd System
│   ├── Ambient Crowd
│   └── Reactive Crowd
├── Commentary System
│   └── Abstract Vocal Cues
├── Ambient Layer
│   └── Per-Venue Ambience
├── Music System
│   └── Procedural Dynamic Music
└── Spatial Audio
    └── 3D Positioned Sources
```

## Procedural Sound Effects

### Disc Sounds

#### Disc Whoosh (In-Flight)
The disc produces a tonal whoosh that varies with speed and spin:

```rust
pub struct DiscWhooshParams {
    pub speed: f32,          // disc air speed in m/s
    pub spin_rate: f32,      // rad/s
    pub altitude: f32,       // for distance attenuation
}

// Synthesis approach:
// - Base: filtered noise (bandpass)
// - Center frequency: 200 Hz + speed * 30 Hz (faster = higher pitch)
// - Bandwidth: wider at higher speeds (more turbulent)
// - Amplitude: proportional to speed² (aerodynamic sound scales with v²)
// - Modulation: spin rate creates periodic flutter (AM at spin frequency)
// - Additional: harmonic at 2× frequency for "singing disc" effect at high spin
```

**Whoosh characteristics by flight phase**:
| Phase | Pitch | Volume | Character |
|-------|-------|--------|-----------|
| Launch (0–0.2s) | Rising quickly | High | Sharp, punchy |
| Cruise | Steady, based on speed | Medium | Sustained hum |
| Fading (speed decreasing) | Dropping | Decreasing | Winding down |
| Near listener | Doppler shift applied | Louder | Pass-by effect |

#### Disc Catch Sound
A short, satisfying "snap" or "clap" when a disc is caught:

```rust
pub fn generate_catch_sound(catch_type: CatchType, disc_speed: f32) -> AudioBuffer {
    match catch_type {
        CatchType::TwoHanded => {
            // Clap-like: short burst of filtered noise
            // Attack: 2ms, Decay: 50ms
            // Higher pitch/louder at higher disc speeds
            // Add subtle "smack" transient
        }
        CatchType::OneHanded => {
            // Snappier, more tonal
            // Short sine burst at 800–1200 Hz
            // Very quick decay (30ms)
        }
        CatchType::Layout => {
            // Catch snap + body impact
            // Layered: catch sound + thud (low-frequency burst)
            // Longer tail from ground slide
        }
    }
}
```

#### Disc Hit Ground
When the disc hits the ground (turnover):
- **Grass**: Soft thud — low-frequency noise burst, 100ms decay, muted character
- **Turf**: Sharper impact — more mid-frequency content, shorter decay
- **Sand**: Dull plop — very low frequency, almost no bounce sound
- **Indoor court**: Sharp click/clatter — high frequency transient, possible rattle
- **Skip**: Initial impact + bouncing sequence with diminishing volume

#### Disc Spike (Celebration)
Aggressive downward throw into ground:
- Loud impact transient (much louder than normal ground hit)
- Add "crack" component (short high-frequency burst)
- Brief reverb tail if in stadium/indoor

### Player Sounds

#### Footsteps
Procedurally generated footsteps synced to the animation system's step triggers:

```rust
pub struct FootstepParams {
    pub surface: SurfaceType,
    pub speed: f32,           // determines step rate and intensity
    pub weight: f32,          // player build
}

// Synthesis per surface:
// Grass: Soft thud + subtle rustle (noise burst 60Hz + filtered noise 2kHz)
// Sand: Crunchy (noise burst with resonant filter sweep, 300–600Hz)
// Turf: Firm step (sharper transient than grass)
// Indoor: Clear tap (higher pitch, more defined transient)
// Mud: Squelch (low noise burst + pitched down resonant sweep)
```

**Step rate**: Tied to animation (see [05-stickman-animation](05-stickman-animation.md))
- Walk: ~1.8 Hz
- Jog: ~2.5 Hz
- Run: ~3.2 Hz
- Sprint: ~3.8 Hz

Volume scales with speed; sprinting footsteps are louder and have more high-frequency content.

#### Breathing / Exertion
When players are low on stamina:
- Heavy breathing sound (filtered noise with rhythmic amplitude modulation)
- Increases in volume and frequency as stamina drops
- Only plays for the controlled player (to avoid cacophony)

#### Layout Impact
When a player dives:
- Body thud: Low-frequency (80Hz) noise burst, 200ms decay
- Slide component: Filtered noise fading over 0.5s
- Surface-dependent: louder on hard surfaces, muffled on sand
- Optional "oof" vocal (short pitched noise)

#### Jump / Land
- Jump: Quick upward pitch sweep (subtle "hup")
- Land: Thud proportional to jump height

### Impact Sounds

#### Hand Block (D)
Quick slap transient + disc deflection sound:
- Slap: Very short (10ms) noise burst at 1–3 kHz
- Followed by disc wobble sound (short, unstable whoosh)

#### Player Collision (Incidental Contact)
Soft thud when players bump:
- Low intensity, brief
- Shouldn't sound violent (Spirit of the Game)

## Crowd System

### Ambient Crowd
A constant background of crowd noise that varies by venue:

| Venue | Crowd Size | Base Volume | Character |
|-------|-----------|-------------|-----------|
| Park | 50–200 | Low | Casual chatter, scattered claps |
| Beach | 50–150 | Low | Relaxed, distant conversations |
| Stadium | 2K–15K | High | Roar, organized cheers |
| Indoor | 200–2K | Medium | Echoey, contained |
| Forest | 20–50 | Very Low | Quiet murmurs |
| Rooftop | 50–200 | Low | Urban backdrop mixed in |

**Synthesis**: Crowd ambience is a mix of:
- Pink noise filtered to 200–4000 Hz range (general "murmur")
- Random sparse claps (Poisson process, rate based on excitement)
- Occasional individual shouts (short tonal bursts at random pitches)

### Reactive Crowd
The crowd reacts to game events with intensity based on the `CrowdSystem.excitement` level:

```rust
pub struct CrowdAudio {
    pub base_volume: f32,
    pub excitement: f32,        // 0–1
    pub home_bias: f32,         // affects cheer/groan balance
}

pub fn crowd_reaction(event: &GameEvent, crowd: &CrowdAudio) -> CrowdReaction {
    match event {
        GameEvent::PointScored { team, .. } => {
            if is_home_team(team) {
                CrowdReaction::Cheer { intensity: 1.0, duration: 3.0 }
            } else {
                CrowdReaction::Groan { intensity: 0.5, duration: 1.5 }
            }
        }
        GameEvent::LayoutCatch => {
            CrowdReaction::OohThenCheer { ooh_duration: 0.5, cheer_intensity: 0.8 }
        }
        GameEvent::Block => {
            CrowdReaction::Cheer { intensity: 0.6, duration: 1.0 }
        }
        GameEvent::Turnover { team, .. } => {
            if is_home_team(team) {
                CrowdReaction::Groan { intensity: 0.4, duration: 1.0 }
            } else {
                CrowdReaction::Cheer { intensity: 0.4, duration: 1.0 }
            }
        }
        GameEvent::Callahan => {
            CrowdReaction::Roar { intensity: 1.0, duration: 5.0 }
        }
        GameEvent::CloseGame => {
            // Sustained excitement — raise ambient level
            CrowdReaction::SustainedExcitement { level: 0.8 }
        }
    }
}
```

**Cheer synthesis**: Layered noise with pitch envelope (rising then sustaining), random individual "woo" sounds scattered in.

**"Ooh" sound**: Short collective pitch rise (200Hz → 400Hz sweep over 0.3s) — triggered by spectacular plays.

## Abstract Commentary

Rather than full voice commentary, FrisKingdom uses abstract vocal cues — short, non-verbal sounds that convey excitement and context without language.

### Commentary Cues
| Trigger | Sound | Description |
|---------|-------|------------|
| Score | Rising "dah-DAH!" | Two-tone celebration cue |
| Turnover | Descending "wah-wah" | Trombone-like failure cue |
| Great throw | Quick "ooh!" | Single bright exclamation |
| Near miss | Sharp intake | Gasp-like transient |
| Stall rising (7+) | Rhythmic ticking | Tension builder |
| Callahan | Extended fanfare | Multi-note celebration |
| Game point | Sustained drone | Tension/anticipation |
| End of game | Full musical sting | Victory/defeat melody |

Commentary cues are synthesized tonal sounds (sine/triangle waves with envelopes), not recorded voices. This keeps the game language-agnostic and fits the abstract aesthetic.

## Ambient Sound

### Per-Venue Ambient Layers
| Venue | Ambient Sources |
|-------|----------------|
| Park | Birds, distant traffic, wind in trees, dog barking (rare) |
| Beach | Ocean waves (continuous), seagulls, distant music |
| Stadium | HVAC hum (subtle), PA system echo, distant city |
| Indoor | Room tone (low hum), echo/reverb on all sounds, shoe squeaks |
| Forest | Birds, rustling leaves, creek (if present), insects |
| Rooftop | City traffic below, wind gusts, helicopter (rare) |

### Wind Audio
Wind produces sound based on the current wind speed:
```rust
pub fn wind_audio_params(wind_speed: f32) -> WindAudioParams {
    WindAudioParams {
        // Volume scales with wind speed
        volume: (wind_speed / 15.0).clamp(0.0, 1.0) * 0.6,
        // Pitch rises with speed (more turbulent)
        center_freq: 100.0 + wind_speed * 40.0,
        // Bandwidth increases with speed
        bandwidth: 50.0 + wind_speed * 30.0,
        // Gusts cause volume modulation
        gust_modulation: wind_speed * 0.1,
    }
}
```

Synthesis: Filtered noise (bandpass) with slow amplitude modulation for gust effect. Stereo panning follows wind direction.

## Procedural Dynamic Music

### Music System Design
Music is procedurally generated and responds to game state. It's not pre-recorded tracks but layered procedural patterns.

### Musical Layers
```
Layer 1: Rhythm (always present)
  - Kick drum pattern (sine burst at 60Hz)
  - Hi-hat pattern (noise bursts)
  - Tempo: 100–140 BPM based on game intensity

Layer 2: Bass (during active play)
  - Simple bass line following a pentatonic scale
  - Root note changes based on score/momentum

Layer 3: Melody (during exciting moments)
  - Short melodic phrases triggered by big plays
  - Pentatonic scale, 2–4 bar phrases
  - Instrument: square wave with filter envelope (chiptune-like)

Layer 4: Pads (atmosphere)
  - Sustained chords (saw wave with low-pass filter)
  - Chord progression shifts with game tension
  - Major key when winning, minor when losing
```

### Dynamic Music Responses
| Game State | Tempo | Layers Active | Key/Mode |
|-----------|-------|---------------|----------|
| Pre-point setup | 80 BPM | Pads only | Neutral |
| Pull | 100 BPM | Rhythm + Pads | Building tension |
| Open play (normal) | 110 BPM | Rhythm + Bass + Pads | Based on score |
| Exciting play | 130 BPM | All layers | Major (if positive for player) |
| Stall count high | 120 BPM + accelerating | Rhythm (ticking) + Pads | Tense |
| Score! | 140 BPM | All layers + fanfare | Major, bright |
| Turnover | Tempo drops | Rhythm + Pads | Minor shift |
| Game point | 120 BPM | Pads (sustained) | Tense, anticipatory |
| Victory | 140 BPM | All + extra celebration | Major, triumphant |
| Defeat | 80 BPM | Pads only | Minor, subdued |

### Musical Scale
All procedural music uses the **pentatonic scale** to ensure everything sounds harmonious regardless of randomization:
- Major pentatonic: C, D, E, G, A (for positive/exciting moments)
- Minor pentatonic: C, Eb, F, G, Bb (for tense/negative moments)

## Spatial Audio

### 3D Sound Positioning
All gameplay sounds are positioned in 3D space:
- Disc whoosh follows disc position
- Footsteps positioned at each player's feet
- Catch/block sounds at the action location
- Crowd sounds positioned around the field perimeter

### Listener Position
The audio listener follows the camera position:
```rust
pub fn update_audio_listener(
    camera: Query<&Transform, With<MainCamera>>,
    mut listener: ResMut<AudioListener>,
) {
    if let Ok(cam_transform) = camera.get_single() {
        listener.position = cam_transform.translation;
        listener.forward = cam_transform.forward();
        listener.up = cam_transform.up();
    }
}
```

### Distance Attenuation
Sounds attenuate with distance from the listener:
```
volume = base_volume / (1 + distance / reference_distance)²
```
- Reference distance: 10m (within this distance, full volume)
- Max distance: 100m (beyond this, sound is inaudible)
- Crowd sounds have a larger reference distance (30m) since they're ambient

### Doppler Effect
Applied to fast-moving sound sources (primarily the disc):
```
f_perceived = f_source × (v_sound / (v_sound - v_source_toward_listener))
```
- Speed of sound: 343 m/s
- Disc at 20 m/s creates a noticeable but subtle Doppler shift
- More pronounced when disc passes close to the camera

## Reverb

### Per-Venue Reverb Settings
| Venue | Reverb Type | Decay (s) | Mix | Notes |
|-------|------------|-----------|-----|-------|
| Park | None/minimal | 0 | 0% | Open air, no reflections |
| Beach | None | 0 | 0% | Open air |
| Stadium | Large hall | 1.2 | 15% | Concrete reflections |
| Indoor | Medium room | 0.8 | 25% | Strong reflections |
| Forest | Diffuse | 0.5 | 10% | Scattered tree reflections |
| Rooftop | City | 0.3 | 5% | Distant building reflections |

### Implementation
```rust
pub struct VenueReverbConfig {
    pub decay_time: f32,
    pub wet_mix: f32,
    pub pre_delay: f32,
    pub room_size: f32,
    pub diffusion: f32,
}
```

Reverb is applied as a global send effect — all sounds route a portion to the reverb bus based on the venue's wet_mix setting.

## Audio Mixing

### Mix Bus Structure
```
Master Bus (final output)
├── SFX Bus (gameplay sounds) ── Volume: 80%
│   ├── Disc Bus ── Volume: 100%
│   ├── Player Bus ── Volume: 90%
│   └── Impact Bus ── Volume: 100%
├── Crowd Bus ── Volume: 70%
├── Commentary Bus ── Volume: 85%
├── Ambience Bus ── Volume: 60%
├── Music Bus ── Volume: 50%
└── UI Bus (menu sounds) ── Volume: 90%
```

All bus volumes are user-adjustable in settings (see [13-ui-ux](13-ui-ux.md)).

### Prioritization
When many sounds play simultaneously, prioritize:
1. Disc sounds (always audible — gameplay-critical)
2. Catch/block sounds (immediate feedback)
3. Controlled player footsteps
4. Commentary cues
5. Other player footsteps
6. Crowd reactions
7. Ambient sounds
8. Music

Maximum simultaneous voices: 32 (excess lower-priority sounds are culled).
