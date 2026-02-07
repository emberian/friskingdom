# 07 — Rendering Pipeline

## Overview
FrisKingdom's rendering pipeline is built on Bevy's wgpu-based renderer with custom shaders for stickman characters, disc visualization, grass, and post-processing. The dynamic camera system provides multiple viewpoints inspired by sports broadcasting. The visual goal is a clean, readable aesthetic with stylized effects.

Related docs: [05-stickman-animation](05-stickman-animation.md), [06-field-and-environment](06-field-and-environment.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md)

## Camera System

### Camera Modes

#### 1. Broadcast Camera (Default)
The primary gameplay camera, positioned at a high angle on the sideline:
```
Side View:
                    Camera <-- 25m up, 30m back
                   /
                  / 40 deg down angle
                 /
    +-------------------------------+
    |         Field                 |
    +-------------------------------+
```

| Parameter | Value |
|-----------|-------|
| Height | 20-30m (dynamic based on play) |
| Lateral offset | 25-35m from field center |
| Look-at target | Midpoint between disc and active players |
| FOV | 45 deg |
| Follow speed | Smooth lerp at 3.0/s |

**Dynamic behavior**:
- Zooms in during tight plays near end zone
- Zooms out during long hucks to show full field
- Slight pan lead in the direction of play
- Rotates around the play to maintain optimal angle

#### 2. Behind-Player Camera
Close follow camera behind the controlled player:
```
Top View:
    Player --> Direction of movement
        \
         Camera (5m behind, 3m up)
```

| Parameter | Value |
|-----------|-------|
| Distance | 4-6m behind player |
| Height | 2.5-3.5m above ground |
| Look-at | Point 5m ahead of player |
| FOV | 60 deg |
| Follow speed | 5.0/s (snappy) |

Best for immersive play experience. Automatically switches to broadcast on disc throw for visibility.

#### 3. Endzone Camera
Fixed camera behind the attacking end zone, showing the field length:
```
    Camera
      |
      v
    +--------------------------------------+
    | EZ |        Field          |   EZ   |
    +--------------------------------------+
```

| Parameter | Value |
|-----------|-------|
| Position | 5m behind end zone, 8m up |
| FOV | 70 deg (wide to see full width) |
| Follow | Tracks disc horizontally |

Used during pull and when offense is near scoring.

#### 4. Cinematic Camera (Replays/Highlights)
Free-moving camera for replays and big plays:
- Slow-motion (0.25x-0.5x speed)
- Orbital motion around the action
- Dolly zoom on key moments (catch, layout, Callahan)
- Depth of field focused on the key player/disc

#### 5. Sky Camera
High overhead (bird's-eye) view:

| Parameter | Value |
|-----------|-------|
| Height | 60-80m |
| Look-at | Field center |
| FOV | 35 deg |
| Use | Tactical view, showing formations |

#### 6. Player-Locked Camera (First Person-ish)
Over-the-shoulder of the controlled player:
- Tight FOV (50 deg)
- Very close to player's head
- Best for immersive throwing experience but limited field awareness
- Optional; unlocked in settings

### Camera Transitions
- Mode switches use a 0.5s smooth interpolation (cubic ease-in-out) for position, rotation, and FOV
- Auto-camera mode (default): Game selects optimal camera based on game state:
  - Pull -> Endzone camera
  - Open play -> Broadcast camera
  - Near end zone -> Endzone or tighter broadcast
  - Replay -> Cinematic camera
  - Player request -> Behind-player or sky

### Camera Shake
Subtle camera shake on impact events:
| Event | Intensity | Duration |
|-------|----------|----------|
| Hard layout landing | 0.3 | 0.4s |
| Disc spike (celebration) | 0.15 | 0.2s |
| Big catch | 0.1 | 0.15s |
| Score | 0.2 | 0.3s |

## Disc Rendering

### Disc Model
- Simple disc mesh (low-poly cylinder with beveled rim)
- Team color decal on top face
- Rim highlight for visibility at distance

### Spin Blur Effect
When the disc is spinning fast, a motion blur effect communicates the spin:
```wgsl
// disc_spin.wgsl — fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let spin_rate = uniforms.spin_rate; // rad/s
    let spin_factor = clamp(abs(spin_rate) / 100.0, 0.0, 1.0);

    // Radial blur: sample texture at multiple rotated UVs
    var color = vec4(0.0);
    let samples = 8u;
    for (var i = 0u; i < samples; i++) {
        let angle = (f32(i) / f32(samples) - 0.5) * spin_factor * 0.3;
        let rotated_uv = rotate_uv(in.uv, angle, vec2(0.5));
        color += textureSample(disc_texture, disc_sampler, rotated_uv);
    }
    color /= f32(samples);

    // Edge glow based on spin
    let edge_dist = distance(in.uv, vec2(0.5)) * 2.0;
    let glow = smoothstep(0.8, 1.0, edge_dist) * spin_factor * 0.5;
    color += vec4(uniforms.team_color.rgb * glow, glow);

    return color;
}
```

### Disc Trail
A fading trail behind the disc during flight:
- Trail is a ribbon mesh that follows disc position
- Fades from disc color to transparent over ~0.5 seconds
- Width based on disc speed
- Becomes more visible at higher speeds

### Disc Shadow
- Blob shadow projected on the ground beneath the disc
- Shadow size/sharpness indicates height (smaller/sharper = lower)
- Critical for gameplay readability — players need to judge disc height

## Stickman Rendering

### Character Shader
Stickman characters use a custom shader:
```wgsl
// stickman.wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Base color from team palette
    var color = in.color;

    // Rim lighting for depth
    let rim = pow(1.0 - max(dot(in.normal, in.view_dir), 0.0), 3.0);
    color = mix(color, vec4(1.0), rim * 0.2);

    // Toon shading: 2-3 light bands
    let ndotl = dot(in.normal, light_dir);
    let band = floor(ndotl * 3.0 + 1.0) / 3.0;
    color *= vec4(vec3(band * 0.6 + 0.4), 1.0);

    return color;
}
```

- Capsule geometry for bones (rounded cylinders)
- Sphere for joints and head
- 2-band toon shading for readable depth
- Rim lighting to separate characters from background
- Team-colored with jersey number as a texture decal

### Player Indicator
- Controlled player has a subtle ring/halo at their feet
- Disc carrier has a bright indicator
- Potential receivers highlighted with dim indicators during throw charge

### Jersey Numbers
- Numbers rendered as SDF (Signed Distance Field) text on the torso capsule
- Visible from both front and back
- Scale with character size

## Post-Processing Pipeline

### Effect Chain
```
Scene Render -> SSAO -> Bloom -> DOF -> Motion Blur -> Color Grading -> Tonemapping -> UI Overlay
```

### 1. Screen Space Ambient Occlusion (SSAO)
- Light SSAO pass for grounding characters on the field
- Low sample count (8 samples) for performance
- Radius: 0.5m (mostly affects foot/ground contact)

### 2. Bloom
- Threshold: 1.2 (only bright highlights bloom)
- Applied to: sun glare, stadium lights, disc glow, score flash
- 3-pass downsample + upsample blur
- Intensity: 0.3 (subtle)

### 3. Depth of Field
- **During gameplay**: Very subtle, keeping the full field in focus
- **During replays/cinematics**: Strong DOF with disc/player as focal point
  - Focal distance: distance to disc/target player
  - Aperture: adjustable for cinematic effect
  - Bokeh: circular (simple) in gameplay, hexagonal in cinematic mode

### 4. Motion Blur
- Per-object motion blur on fast-moving elements (disc, sprinting players)
- Not full-screen motion blur (causes nausea)
- Samples: 4 velocity samples per pixel
- Strength: scales with object velocity

### 5. Color Grading
Venue-specific color grading LUTs:
| Venue | Grading | Tone |
|-------|---------|------|
| Park (day) | Warm, saturated | Cheerful |
| Beach | Bright, teal-orange | Summery |
| Stadium (night) | Cool, contrasty | Dramatic |
| Indoor | Neutral, slightly flat | Clinical |
| Forest | Green-tinted, warm | Earthy |
| Rooftop (night) | High contrast, neon tint | Urban |

### 6. Tonemapping
ACES (Academy Color Encoding System) filmic tonemapping for natural-looking highlights.

## Particle Effects

### Particle System Overview
GPU-driven particle system for environmental and gameplay effects:

| Effect | Max Particles | Lifetime | Trigger |
|--------|--------------|----------|---------|
| Dust puff | 20 | 0.5s | Player cuts/stops |
| Grass spray | 30 | 0.3s | Layout on grass |
| Sand spray | 40 | 0.5s | Layout on sand, footsteps on beach |
| Rain drops | 2000 | 1.5s | Rain weather |
| Snow flakes | 1000 | 3.0s | Snow weather |
| Wind dust | 500 | 2.0s | Windy conditions |
| Celebration confetti | 200 | 2.0s | Score |
| Sweat drops | 5 | 0.3s | Low stamina sprint |
| Disc catch spark | 15 | 0.2s | Clean catch |

### Particle Interaction with Wind
All outdoor particles sample the wind texture to drift realistically:
```rust
fn update_particle(particle: &mut Particle, wind: &WindTexture, dt: f32) {
    let wind_sample = wind.sample(particle.position);
    particle.velocity += wind_sample * particle.wind_influence * dt;
    particle.position += particle.velocity * dt;
    particle.lifetime -= dt;
}
```

## Lighting

### Outdoor Lighting
- Single directional light (sun/moon) with cascaded shadow maps (3 cascades)
- Shadow cascade distances: 15m, 50m, 150m
- Ambient light: hemisphere (sky color above, ground bounce below)
- Environment map for reflections (subtle, mostly on wet surfaces)

### Stadium Lighting (Night)
- 4-8 point/spot lights representing stadium light towers
- Shadow maps per light (low-res for distant lights)
- Volumetric light shafts through dust/fog particles
- Player shadows cast from multiple light sources

### Indoor Lighting
- Array of overhead area lights
- Soft shadows (PCSS or similar)
- No volumetric effects (clean air)

## Rendering Performance Budget

Target: 60 FPS at 1080p on mid-range GPU (RTX 3060 / RX 6600 equivalent)

| Pass | Budget (ms) | Notes |
|------|------------|-------|
| Shadow maps | 1.5 | 3 cascades |
| Scene geometry | 1.5 | 14 stickmen + disc + field |
| Grass | 1.5 | 50K instanced blades |
| Particles | 0.5 | GPU compute + billboard render |
| SSAO | 0.5 | Half-res |
| Bloom | 0.3 | 3-pass |
| DOF | 0.3 | Half-res (gameplay mode) |
| Motion blur | 0.3 | Per-object only |
| Color grading + tonemap | 0.1 | Single fullscreen pass |
| UI overlay | 0.3 | Composed on top |
| **Total** | **~6.5ms** | Well within 16.6ms budget |

## Quality Presets

| Setting | Low | Medium | High | Ultra |
|---------|-----|--------|------|-------|
| Shadow resolution | 512 | 1024 | 2048 | 4096 |
| Shadow cascades | 1 | 2 | 3 | 4 |
| Grass blades | 10K | 25K | 50K | 100K |
| SSAO | Off | 4 samples | 8 samples | 16 samples |
| Bloom | Off | On | On | On |
| DOF | Off | Gameplay only | Full | Full + bokeh |
| Motion blur | Off | Off | On | On |
| Particles | 50% | 75% | 100% | 150% |
| Anti-aliasing | FXAA | TAA | TAA | TAA + FXAA |
| Crowd detail | Low (static) | Medium | High (animated) | Ultra (individual) |
