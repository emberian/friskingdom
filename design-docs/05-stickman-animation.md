# 05 — Stickman Animation & Style

## Overview
FrisKingdom uses a stylized stickman aesthetic — minimalist characters with expressive procedural animation. The animation system is driven entirely by inverse kinematics (IK) and procedural motion rather than traditional keyframe animation. A unique "swag" confidence system modifies how players move and animate, giving each player personality.

Related docs: [04-player-controls](04-player-controls.md), [07-rendering](07-rendering.md), [11-season-management](11-season-management.md)

## Stickman Rig

### Bone Structure (~12 bones)
```
          Head (sphere)
            |
          Neck
            |
     ┌──── Spine ────┐
     |      |         |
  L_Shoulder  R_Shoulder
     |                |
  L_Upper_Arm   R_Upper_Arm
     |                |
  L_Lower_Arm   R_Lower_Arm
     |                |
  L_Hand        R_Hand
            |
          Hips
       ┌───┴───┐
  L_Upper_Leg  R_Upper_Leg
       |              |
  L_Lower_Leg  R_Lower_Leg
       |              |
  L_Foot        R_Foot
```

### Bone Properties
| Bone | Length (arbitrary units) | Radius | Notes |
|------|------------------------|--------|-------|
| Head | — | 0.12 | Sphere; slightly oversized for readability |
| Neck | 0.08 | 0.02 | Short connector |
| Spine | 0.30 | 0.03 | Single segment (torso) |
| Shoulder | 0.15 | — | Offset from spine top |
| Upper Arm | 0.20 | 0.025 | |
| Lower Arm | 0.18 | 0.02 | |
| Hand | 0.06 | 0.015 | Small for subtle gestures |
| Hips | 0.12 | — | Offset from spine bottom |
| Upper Leg | 0.28 | 0.03 | |
| Lower Leg | 0.26 | 0.025 | |
| Foot | 0.10 | 0.02 | |

### Rendering Style
- Bones rendered as capsules (rounded cylinders) with consistent thickness
- Joints rendered as small spheres at connection points
- Head is a sphere; can be customized (hats, headbands, etc.)
- Color palette per team (jersey = torso color, shorts = hip/upper leg color)
- Optional accessories: headband, wristband, cleats (visual-only elements)

## Procedural IK Locomotion

All locomotion is generated procedurally using inverse kinematics. No pre-baked walk/run cycles.

### Ground Contact System (Foot Placement)
The foot placement system uses a step-trigger approach:

```rust
pub struct FootPlacement {
    pub left_foot_target: Vec3,    // Current ground target for left foot
    pub right_foot_target: Vec3,   // Current ground target for right foot
    pub left_foot_phase: f32,      // 0 = grounded, 1 = mid-step
    pub right_foot_phase: f32,
    pub stride_length: f32,        // Adapts to speed
    pub step_height: f32,          // How high feet lift
    pub step_speed: f32,           // Step frequency
}
```

**Step Logic**:
1. Feet have target positions on the ground, offset from the hips
2. When the hip moves far enough from a foot's target, that foot initiates a step
3. Steps alternate (left, right, left, right)
4. Step trajectory follows an arc (sine curve for height)
5. The other foot stays planted during the step

```
Step Arc:
         /‾‾\
        /    \
  ─────/      \─────
  [old pos]    [new pos]
```

### Speed-Adaptive Motion
| Movement | Stride Length | Step Frequency | Step Height | Arm Swing |
|----------|-------------|----------------|-------------|-----------|
| Idle | 0 | 0 | 0 | Subtle sway |
| Walk | 0.4m | 1.8 Hz | 0.03m | Light |
| Jog | 0.7m | 2.5 Hz | 0.06m | Moderate |
| Run | 1.0m | 3.2 Hz | 0.10m | Full |
| Sprint | 1.3m | 3.8 Hz | 0.14m | Pumping |

### Lean & Tilt
- Forward lean proportional to acceleration (0° standing → 15° full sprint)
- Lateral lean during turns (proportional to angular velocity)
- Deceleration: character leans back slightly, shorter stride

### Arm Swing
Arms swing opposite to legs (natural counter-rotation):
- Amplitude increases with speed
- When carrying disc: one arm holds disc at side/chest, other arm swings
- During cuts: arms pump asymmetrically to sell the direction change

## Throw Animations (Procedural)

Each throw type has a procedural animation sequence defined by keypose targets and IK constraints:

### Backhand Throw Sequence
```
1. Wind-up (0.15s):
   - Torso rotates ~90° away from throw direction
   - Throwing arm extends back across body
   - Off-hand arm balances
   - Weight shifts to back foot

2. Release (0.1s):
   - Torso snaps forward
   - Throwing arm sweeps across body
   - Wrist flicks at release point
   - Weight transfers to front foot

3. Follow-through (0.2s):
   - Arm continues past release point
   - Torso continues rotation slightly
   - Return to neutral over 0.3s
```

### Forehand Throw Sequence
```
1. Wind-up (0.12s):
   - Throwing arm cocks back at side (elbow bent ~90°)
   - Slight torso rotation to throwing side
   - Off-hand out for balance
   - Weight on back foot

2. Release (0.08s):
   - Arm extends forward and slightly out
   - Wrist snap (most critical motion)
   - Elbow leads, then forearm whips through

3. Follow-through (0.15s):
   - Arm extends fully
   - Index/middle finger point at target
   - Body returns to neutral
```

### Hammer Throw Sequence
```
1. Wind-up (0.2s):
   - Arm raises overhead
   - Torso tilts slightly to non-throwing side
   - Disc above and behind head

2. Release (0.1s):
   - Arm whips forward and over
   - Release point above head
   - Disc releases inverted

3. Follow-through (0.2s):
   - Arm continues downward in front
   - Body straightens
```

### Other Throws
Similar procedural sequences for scoober (upside-down forehand release overhead), thumber (overhead, inverted grip), blade (vertical forehand), push pass (short pushing motion), chicken wing (behind-the-back release).

## Catch Animations

### Standard Catch (Two-Handed)
- Arms extend toward disc arrival point (IK targets the disc)
- Hands clap together at catch moment
- Arms pull disc in toward chest/body
- Body absorbs slightly (small crouch)

### One-Handed Catch
- Triggered when disc is far from body center
- Reaching arm extends fully via IK
- Hand snaps shut on disc
- "Cool factor" — boosts swag meter

### Layout Catch (Diving)
```
1. Launch (0.1s):
   - Both feet leave ground
   - Body extends horizontally toward disc
   - Arms stretch forward

2. Flight (0.2–0.4s):
   - Body is horizontal, fully extended
   - Arms reaching for disc
   - IK tracks disc position

3. Catch/Miss + Landing (0.3s):
   - If catch: arms clutch disc to chest mid-air
   - Body impacts ground (chest slide)
   - Gradual deceleration on ground

4. Recovery (1.0–1.5s):
   - Roll to side, push up
   - Stand up with disc (if caught)
   - Penalty: 1.5s before can throw
```

### Sky (Jump Catch)
- Timed jump with arms extending overhead
- At apex, arms reach for disc
- Contested: multiple players jump, highest reach wins
- Landing: absorb with knee bend

## The "Swag" System

The swag system is FrisKingdom's signature animation modifier. It represents a player's current confidence level, which affects how they move, throw, and celebrate.

### Swag Meter
```rust
pub struct SwagMeter {
    pub current: f32,      // 0.0 (dejected) to 1.0 (peak confidence)
    pub base: f32,         // Player's baseline swag (personality stat)
    pub momentum: f32,     // Rate of change
    pub decay_rate: f32,   // How fast swag fades without reinforcement
}
```

### Swag Events (What Changes It)
| Event | Swag Change | Duration |
|-------|-------------|----------|
| Score a goal | +0.25 | Sustained |
| Assist | +0.15 | Sustained |
| Layout catch (success) | +0.20 | 30s |
| Layout catch (fail/drop) | -0.10 | 20s |
| Callahan | +0.40 | Rest of game |
| Greatest | +0.35 | Rest of game |
| Beautiful throw (break, huck completion) | +0.10 | 20s |
| Turnover (own throw) | -0.15 | 30s |
| Drop catch | -0.12 | 20s |
| Get scored on (defender) | -0.08 | 15s |
| Get D'd (thrower) | -0.10 | 20s |
| Consecutive completions (3+) | +0.05/each | 15s |
| Teammate scores | +0.05 | 10s |

### Swag Animation Effects

| Swag Level | Movement Style | Throw Style | Idle Behavior |
|-----------|---------------|-------------|---------------|
| 0.0–0.2 (Low) | Hunched, slower transitions | Hesitant wind-up, shorter follow-through | Hands on knees, looking down |
| 0.2–0.4 (Below Average) | Slightly reserved | Normal but cautious | Shifts weight, fidgets |
| 0.4–0.6 (Neutral) | Standard animations | Clean, efficient | Relaxed standing |
| 0.6–0.8 (Confident) | Upright, fluid transitions | Exaggerated follow-through, smooth | Bouncing on toes, looking around |
| 0.8–1.0 (Peak Swag) | Swagger walk, head high | Flashy wind-ups, flourishes | Clapping, pointing, hyping teammates |

### Procedural Swag Modifications
The swag value modifies animation parameters:
```rust
fn apply_swag_modifiers(animation: &mut AnimationState, swag: f32) {
    // Posture: 0 = hunched, 1 = upright/leaned-back
    animation.spine_straightness = lerp(0.85, 1.05, swag);

    // Head angle: low swag = looking down, high = chin up
    animation.head_pitch = lerp(-5.0_deg, 8.0_deg, swag);

    // Movement smoothness: higher swag = smoother transitions
    animation.blend_speed = lerp(6.0, 12.0, swag);

    // Step bounce: high swag = bouncier step
    animation.step_height_multiplier = lerp(0.8, 1.3, swag);

    // Arm relaxation: low swag = tight/tense, high = loose/flowing
    animation.arm_swing_amplitude = lerp(0.7, 1.2, swag);

    // Idle fidget rate: low swag = more fidgeting
    animation.fidget_frequency = lerp(2.0, 0.5, swag);
}
```

## Celebration Animations

After scoring, the scoring player and nearby teammates trigger celebration animations:

### Solo Celebrations
| Name | Description | Swag Requirement |
|------|-------------|-----------------|
| Fist pump | Simple arm raise | 0.0+ (default) |
| Disc spike | Slam disc into ground | 0.3+ |
| Airplane | Run with arms out | 0.5+ |
| Finger guns | Point at crowd | 0.6+ |
| Moonwalk | Backwards slide | 0.7+ |
| Robot | Stiff robotic movements | 0.8+ |
| The Worm | Ground body wave | 0.9+ |

### Team Celebrations (2+ players)
| Name | Description | Trigger |
|------|-------------|---------|
| Chest bump | Two players jump and collide | Scorer + assister near each other |
| High-five line | Run through teammate hand-slaps | Any score with 3+ teammates nearby |
| Synchronized jump | All nearby players jump together | Big point (break point, game point) |
| Pile-on | Teammates run and dogpile | Callahan or game-winning point |

Celebrations are procedural — constructed from IK targets and timing curves, not keyframed.

## Procedural Appearance Generation

Each player has a procedurally generated appearance:

```rust
pub struct PlayerAppearance {
    pub height: f32,              // 1.65–1.95m, affects all bone lengths
    pub build: f32,               // 0 = slim, 1 = stocky (bone radius scaling)
    pub skin_tone: Color,         // from a curated palette
    pub hair_style: HairStyle,    // None, Short, Medium, Long, Bun, Mohawk, Braids
    pub hair_color: Color,
    pub accessory: Option<Accessory>, // Headband, Wristband, Sunglasses, Cap
    pub jersey_number: u8,        // 0–99, displayed on torso
    pub team_colors: TeamColors,  // Primary, secondary, accent
    pub handedness: Handedness,   // Left or Right (affects throw animations)
}
```

### Height-Performance Relationship
- Taller players: Better reach for sky catches, but slightly slower acceleration
- Shorter players: Better agility and quicker cuts
- Height is cosmetic in casual mode, affects gameplay in competitive/career mode

### Hair Physics
Longer hair styles have simple procedural physics:
- Responds to head movement with delay (spring system)
- Swings during sprinting and direction changes
- Flattens in wind direction

## Technical Implementation

### IK Solver
Two-bone IK with pole vector for arms and legs:
```rust
pub fn solve_two_bone_ik(
    root: Vec3,        // shoulder/hip
    target: Vec3,      // hand/foot target
    pole: Vec3,        // elbow/knee direction hint
    bone_a_len: f32,   // upper arm/leg length
    bone_b_len: f32,   // lower arm/leg length
) -> (Quat, Quat) {   // returns joint rotations
    let total_len = bone_a_len + bone_b_len;
    let dist = (target - root).length().min(total_len * 0.999);

    // Law of cosines for elbow/knee angle
    let cos_angle_b = (bone_a_len.powi(2) + bone_b_len.powi(2) - dist.powi(2))
                     / (2.0 * bone_a_len * bone_b_len);
    let angle_b = cos_angle_b.acos();

    // ... compute rotations with pole vector constraint
    (rotation_a, rotation_b)
}
```

### Animation Blending
- Procedural animations are layered and blended using weighted masks
- Upper body and lower body blend independently (e.g., throwing while running)
- Transition blending: 4-frame (~67ms) crossfade between states
- Priority system: throw animation overrides arm swing; foot IK always active

### Update Order (Per Frame)
1. Update player state (position, velocity from physics)
2. Compute foot targets (ground IK)
3. Compute hand targets (disc interaction / arm swing)
4. Solve leg IK chains
5. Solve arm IK chains
6. Apply spine/head procedural motion
7. Apply swag modifiers
8. Compute hair/accessory physics
9. Build bone transforms for rendering

### Bevy Components
```rust
#[derive(Component)]
pub struct StickmanRig {
    pub bones: [BoneTransform; 12],
    pub ik_targets: IKTargets,
}

#[derive(Component)]
pub struct ProceduralAnimation {
    pub locomotion: LocomotionState,
    pub upper_body: UpperBodyState,
    pub swag: SwagMeter,
    pub celebration: Option<CelebrationState>,
}

#[derive(Component)]
pub struct PlayerAppearance {
    // ... as defined above
}
```

All animation updates run in the `AnimationSet` system set, after physics and before rendering (see [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)).
