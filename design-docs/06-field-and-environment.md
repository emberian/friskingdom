# 06 — Fields & Venues

## Overview
FrisKingdom features multiple venue types, each with distinct visual character, weather behavior, and gameplay implications. Venues are procedurally generated from templates, ensuring variety while maintaining consistent field dimensions. The environment system handles field surfaces, surroundings, weather effects, time of day, and crowd rendering.

Related docs: [02-wind-field](02-wind-field.md), [07-rendering](07-rendering.md), [12-audio](12-audio.md)

## Field Specifications

All venues use standard WFDF field dimensions (see [03-ultimate-rules-gameplay](03-ultimate-rules-gameplay.md)):

| Measurement | Value |
|-------------|-------|
| Playing field | 64m × 37m |
| End zones | 18m × 37m each |
| Total | 100m × 37m |
| Brick marks | 18m from each end zone |
| Safety margin | 3m beyond all lines (minimum) |

### Field Surface Properties
The field surface affects player movement and disc behavior:

| Surface | Traction | Sprint Speed Modifier | Disc Skip | Venues |
|---------|----------|----------------------|-----------|--------|
| Natural grass (dry) | 1.0 (baseline) | 1.0 | Medium bounce | Park, Stadium |
| Natural grass (wet) | 0.75 | 0.90 | High skip | Park, Stadium (rain) |
| Artificial turf | 1.1 | 1.05 | Low bounce, fast roll | Stadium (indoor-outdoor) |
| Beach sand | 0.6 | 0.70 | No skip, disc sticks | Beach |
| Indoor court | 1.15 | 1.0 | Sharp skip | Indoor |
| Mud | 0.5 | 0.65 | Disc sticks/plops | Park (heavy rain) |

## Venue Types

### 1. Park (Default Venue)
**Description**: An open public park with grass fields, trees on the perimeter, and a casual atmosphere. The most common ultimate frisbee setting.

```
Layout:
   [Trees]  [Trees]  [Shade Structure]  [Trees]

   ┌──────────────────────────────────────┐
   │  End Zone  │   Playing Field   │  EZ  │
   │            │                   │      │
   │            │                   │      │
   │            │                   │      │
   └──────────────────────────────────────┘

   [Parking Lot]  [Playground]  [Walking Path]
```

| Property | Value |
|----------|-------|
| Surface | Natural grass |
| Wind exposure | Full (open field) |
| Surface roughness (z0) | 0.03 |
| Lighting | Natural (time of day dependent) |
| Crowd capacity | 50–200 (informal, spread out) |
| Wind obstacles | Perimeter trees |
| Special features | Shade trees, park benches, walking path |

**Procedural variation**: Tree placement, park features, background buildings.

### 2. Beach
**Description**: A sandy beach with ocean backdrop. Popular for beach ultimate tournaments.

| Property | Value |
|----------|-------|
| Surface | Sand |
| Wind exposure | Very high (oceanfront) |
| Surface roughness (z0) | 0.0002 |
| Lighting | Bright, high sun, water reflections |
| Crowd capacity | 50–150 (blankets/chairs along sideline) |
| Wind obstacles | Minimal (possible dune, volleyball net) |
| Special features | Ocean waves, tide line, seagulls, sunset mode |

**Gameplay impact**: Sand dramatically reduces player speed. Layouts are easier (softer landing) but running is harder. Wind is typically stronger and more consistent.

### 3. Stadium
**Description**: A purpose-built or multi-sport stadium for major tournaments and league finals.

| Property | Value |
|----------|-------|
| Surface | Artificial turf or premium grass |
| Wind exposure | Moderate (partially shielded by stands) |
| Surface roughness (z0) | 0.1 |
| Lighting | Mixed (natural + stadium lights for evening) |
| Crowd capacity | 2,000–15,000 |
| Wind obstacles | Stadium walls create wind tunnels and sheltering |
| Special features | Jumbotron, PA system, proper scoreboards, fireworks |

**Procedural variation**: Stadium size scales with career/season prestige. Playoff finals get larger venues.

### 4. Indoor Arena
**Description**: An indoor facility — gymnasium or converted warehouse.

| Property | Value |
|----------|-------|
| Surface | Indoor court (wood or sport surface) |
| Wind exposure | None (no wind) |
| Surface roughness (z0) | N/A |
| Lighting | Artificial (overhead fluorescent/LED) |
| Crowd capacity | 200–2,000 |
| Wind obstacles | N/A |
| Special features | Echo acoustics, bright even lighting, roof structure visible |

**Gameplay impact**: No wind means pure skill-based disc flight. Ceiling height (typically 10–15m) limits high throws — hammers and scoobers may hit the ceiling.

### 5. Forest Clearing
**Description**: A clearing in a wooded area. Atmospheric and unusual.

| Property | Value |
|----------|-------|
| Surface | Natural grass (patchy) |
| Wind exposure | Low at field level (trees block), variable at height |
| Surface roughness (z0) | 0.5 |
| Lighting | Dappled (sun through trees), golden hour feel |
| Crowd capacity | 20–50 |
| Wind obstacles | Dense tree line creates complex wind patterns |
| Special features | Leaf particles, bird sounds, shadows from canopy |

### 6. Rooftop
**Description**: An urban rooftop field high above the city.

| Property | Value |
|----------|-------|
| Surface | Artificial turf |
| Wind exposure | Extreme (elevated, exposed) |
| Surface roughness (z0) | 0.01 (smooth rooftop) |
| Lighting | City lights at night, skyline backdrop |
| Crowd capacity | 50–200 |
| Wind obstacles | Rooftop edges create updrafts, nearby buildings cause turbulence |
| Special features | City skyline, helicopter flyovers, neon signs |

## Time of Day

Each venue supports multiple times of day, affecting lighting and atmosphere:

| Time | Sun Angle | Light Color | Shadow Length | Mood |
|------|-----------|------------|--------------|------|
| Morning (8–10am) | Low east | Warm gold | Long, west | Fresh, crisp |
| Midday (11am–2pm) | High | Neutral white | Short | Bright, intense |
| Afternoon (3–5pm) | Medium west | Warm | Medium, east | Classic |
| Golden Hour (5:30–6:30pm) | Low west | Deep orange | Very long, east | Beautiful, dramatic |
| Evening/Dusk (7–8pm) | Below horizon | Purple/blue | None (diffuse) | Atmospheric |
| Night | N/A | Stadium lights only | From light sources | Dramatic, competitive |

Night mode requires a venue with lights (Stadium, Indoor, Rooftop).

## Weather System

Weather affects visuals, audio, disc flight (via wind), and surface conditions:

### Weather Conditions
| Condition | Wind | Visibility | Surface | Visual Effects |
|-----------|------|-----------|---------|---------------|
| Clear | Calm–Light | Full | Dry | Blue sky, sun glare |
| Partly Cloudy | Light–Moderate | Full | Dry | Moving cloud shadows |
| Overcast | Moderate | Slightly reduced | Dry | Flat lighting, grey sky |
| Light Rain | Moderate–Strong | Reduced | Wet | Rain particles, puddles |
| Heavy Rain | Strong | Low | Wet/Mud | Dense rain, standing water |
| Fog | Light–Calm | Very low | Damp | Distance fog, mist |
| Snow | Light–Moderate | Reduced | Snow cover (slow) | Snowflakes, white field |

### Weather Transitions
Weather can change during a match (rare, but adds drama):
- Cloud cover builds over 2–5 minutes
- Rain starts light, intensifies over 1–3 minutes
- Field conditions degrade gradually (grass → wet → mud takes ~10 game minutes)

## Procedural Environment Generation

Each venue is generated from a template plus random seed:

```rust
pub struct VenueTemplate {
    pub venue_type: VenueType,
    pub field_surface: Surface,
    pub perimeter_objects: Vec<PerimeterObject>,
    pub background_layers: Vec<BackgroundLayer>,
    pub crowd_zones: Vec<CrowdZone>,
    pub light_sources: Vec<LightSource>,
    pub ambient_audio: Vec<AmbientSource>,
}

pub struct VenueGenerator {
    pub template: VenueTemplate,
    pub seed: u64,
}

impl VenueGenerator {
    pub fn generate(&self) -> GeneratedVenue {
        let mut rng = StdRng::seed_from_u64(self.seed);

        // Place perimeter objects with variation
        let objects = self.template.perimeter_objects.iter()
            .map(|obj| obj.randomize(&mut rng))
            .collect();

        // Generate background (skyline, mountains, ocean, etc.)
        let background = self.generate_background(&mut rng);

        // Place crowd
        let crowd = self.generate_crowd(&mut rng);

        // Generate field markings and surface detail
        let field = self.generate_field_surface(&mut rng);

        GeneratedVenue { objects, background, crowd, field, /* ... */ }
    }
}
```

### Perimeter Objects
| Object | Park | Beach | Stadium | Indoor | Forest | Rooftop |
|--------|------|-------|---------|--------|--------|---------|
| Trees | Many | Palm trees | Few | None | Dense | Potted |
| Benches | Some | Chairs | Stadium seats | Bleachers | Logs | None |
| Fencing | Low | None | High | Walls | None | Railing |
| Scoreboards | Manual | None | Electronic | Small | None | LED |
| Tents/Canopy | Pop-ups | Umbrellas | VIP boxes | None | None | Awnings |

## Crowd System

### Crowd Rendering
Crowds are rendered as billboard sprites (2D cards facing the camera) with procedural variation:
- Each spectator is a small stickman sprite (matching the game's art style)
- Variations: standing, sitting, waving, cheering, holding signs
- Color variation based on team allegiance (split crowd or home advantage)
- LOD: Close spectators get individual animation; distant ones are static/batch-rendered

### Crowd Behavior
```rust
pub struct CrowdSystem {
    pub excitement: f32,     // 0–1, affects cheer volume and animation
    pub home_bias: f32,      // 0–1, how much crowd favors home team
}
```

| Game Event | Crowd Reaction |
|-----------|---------------|
| Score (home) | Big cheer, arms raise, excitement spike |
| Score (away) | Groan, mixed reactions |
| Layout catch | Collective "ooh!" then applause |
| D / block | Cheers |
| Turnover (home) | Disappointed murmur |
| Callahan | Crowd goes wild |
| Close game | Sustained high excitement |

Crowd audio integrates with the audio system (see [12-audio](12-audio.md)).

## Field Rendering Details

### Field Lines
- White lines, 5cm wide (real spec)
- Rendered as textured quads on the field surface
- Cone markers at corners and brick marks (small 3D objects)

### Grass Rendering (Outdoor Venues)
- Procedural grass blade shader
- Blades respond to wind (sampled from wind texture, see [02-wind-field](02-wind-field.md))
- Player footprint deformation (subtle, temporary)
- Mowing pattern (alternating lighter/darker stripes)

```wgsl
// grass.wgsl — vertex shader for grass blade instancing
@vertex
fn vs_main(@builtin(instance_index) inst: u32, @location(0) vert: vec3<f32>) -> VertexOutput {
    let blade_pos = grass_positions[inst];
    let wind = textureSampleLevel(wind_texture, wind_sampler, blade_pos.xz / field_size, 0.0).xz;

    // Bend blade based on wind and height along blade
    let bend = wind * vert.y * vert.y * 0.3;  // quadratic bend (more at tip)
    let world_pos = blade_pos + vert + vec3(bend.x, 0.0, bend.y);

    // ... transform to clip space
}
```

### Sand Rendering (Beach)
- Particle-based sand surface with normal mapping
- Footprint decals that fade over time
- Disc impact marks in sand
- Windblown sand particles along field surface

### Sky
- Procedural sky shader based on time of day
- Sun/moon position calculated from time
- Cloud layers (scrolling textures with noise)
- Weather overlay (rain, snow, fog) as post-process

## Venue Configuration File Format

Venues are defined in RON (Rusty Object Notation) configuration files:

```ron
Venue(
    name: "Sunset Park",
    venue_type: Park,
    surface: NaturalGrass,
    wind_range: (min: 0.0, max: 8.0),
    z0: 0.03,
    time_of_day: Afternoon,
    weather: PartlyCloudy,
    perimeter: [
        Tree(position: (-25, 0, -5), height_range: (4.0, 8.0), count: 8),
        Tree(position: (25, 0, -5), height_range: (4.0, 8.0), count: 6),
        Bench(position: (-20, 0, 20), count: 4),
        ScoreBoard(position: (0, 0, -22), style: Manual),
    ],
    crowd_zones: [
        CrowdZone(side: Near, density: 0.3, length: 80.0),
        CrowdZone(side: Far, density: 0.1, length: 40.0),
    ],
    background: [
        Layer(type: Skyline, distance: 500.0, parallax: 0.02),
        Layer(type: Hills, distance: 200.0, parallax: 0.05),
    ],
)
```

## Memory & Performance Considerations

| System | Budget | Notes |
|--------|--------|-------|
| Field mesh | 1 draw call | Single textured quad with detail normal map |
| Grass | ~50K blades | GPU instanced, LOD by distance |
| Crowd | ~500 sprites | Billboarded, batch rendered |
| Trees/Objects | ~20–50 objects | Instanced where possible |
| Sky | 1 fullscreen pass | Procedural shader |
| Weather particles | ~2000 particles | GPU particle system |
| Wind texture | 64×16×128 | Updated per frame (see [02-wind-field](02-wind-field.md)) |

Total venue rendering budget target: <2ms on mid-range GPU.
