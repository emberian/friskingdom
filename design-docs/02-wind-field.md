# 02 — Wind Simulation

> Metadata: `doc_version=1.1`, `last_updated_utc=2026-02-07`, `last_validated_commit=workspace-local`

## Overview
Wind profoundly affects disc flight in ultimate frisbee. FrisKingdom models wind as a 3D vector field that evolves over time, composed of layered components: base flow, gusts, thermals, and obstacle wake effects. The wind system feeds into the `frisbee-physics` crate via the `WindField` trait (see [01-disc-physics](01-disc-physics.md)) and influences player movement, audio, and visual effects (see [06-field-and-environment](06-field-and-environment.md), [12-audio](12-audio.md)).

## Wind Architecture

```
WindSystem
├── BaseWind          — Steady directional wind
├── GustLayer         — Turbulent gusts (Perlin noise)
├── ThermalLayer      — Vertical updrafts (optional, outdoor venues)
├── ObstacleWake      — Wake from trees/buildings near field
└── GPU Wind Texture  — Baked 3D texture for compute shader sampling
```

### WindField Trait (from frisbee-physics)
```rust
pub trait WindField: Send + Sync {
    fn sample(&self, position: DVec3, time: f64) -> DVec3;
}
```

The game implements `CompositeWindField` which combines all layers:
```rust
pub struct CompositeWindField {
    pub base: BaseWind,
    pub gusts: GustLayer,
    pub thermals: Option<ThermalLayer>,
    pub obstacles: Vec<ObstacleWake>,
}

impl WindField for CompositeWindField {
    fn sample(&self, pos: DVec3, t: f64) -> DVec3 {
        let mut w = self.base.sample(pos, t);
        w += self.gusts.sample(pos, t);
        if let Some(ref thermal) = self.thermals {
            w += thermal.sample(pos, t);
        }
        for obs in &self.obstacles {
            w += obs.sample(pos, t);
        }
        w
    }
}
```

## Base Wind

The base wind is a steady directional flow with height-dependent speed (logarithmic wind profile):

```
v_base(h) = v_ref × ln(h / z0) / ln(h_ref / z0)
```

| Parameter | Symbol | Description | Default |
|-----------|--------|-------------|---------|
| Reference speed | v_ref | Wind speed at reference height | 0–15 m/s |
| Reference height | h_ref | Height of reference measurement | 10.0 m |
| Surface roughness | z0 | Depends on venue type | varies |
| Direction | θ_wind | Horizontal angle (0 = +Z) | 0–2π |

### Surface Roughness by Venue
| Venue | z0 (m) | Notes |
|-------|--------|-------|
| Beach | 0.0002 | Smooth sand, minimal friction |
| Open park | 0.03 | Short grass |
| Stadium | 0.1 | Surrounded by structures |
| Indoor | N/A | No wind (or HVAC draft) |
| Forest clearing | 0.5 | Trees create turbulence |

### Wind Direction Shift
Over a match, the base wind direction can slowly drift:
```
θ(t) = θ_initial + A_drift × sin(2π × t / T_drift)
A_drift: ±5° to ±15° (random per match)
T_drift: 300–600 seconds
```

## Gust Model

Gusts are modeled using 3D Perlin noise with multiple octaves, creating spatially and temporally coherent turbulence:

```rust
pub struct GustLayer {
    pub intensity: f64,           // 0.0–1.0 scale factor
    pub noise_scale_spatial: f64, // meters per noise unit (default: 20.0)
    pub noise_scale_temporal: f64,// seconds per noise unit (default: 5.0)
    pub octaves: u32,             // default: 3
    pub persistence: f64,         // amplitude falloff per octave (default: 0.5)
    pub lacunarity: f64,          // frequency multiplier per octave (default: 2.0)
    pub seed: u64,                // deterministic seed
}

impl GustLayer {
    pub fn sample(&self, pos: DVec3, t: f64) -> DVec3 {
        let p = pos / self.noise_scale_spatial;
        let tt = t / self.noise_scale_temporal;

        // Sample 3 independent noise channels for x, y, z wind components
        let wx = fbm_noise_3d(p.x, p.y, p.z + 0.0, tt, self.octaves, self.persistence, self.lacunarity, self.seed);
        let wy = fbm_noise_3d(p.x, p.y, p.z + 100.0, tt, self.octaves, self.persistence, self.lacunarity, self.seed);
        let wz = fbm_noise_3d(p.x, p.y, p.z + 200.0, tt, self.octaves, self.persistence, self.lacunarity, self.seed);

        DVec3::new(wx, wy * 0.3, wz) * self.intensity * self.base_gust_speed
    }
}
```

**Gust characteristics:**
- Spatial coherence: ~20m correlation length (gusts affect disc and nearby players similarly)
- Temporal coherence: ~5s (gusts build and fade naturally)
- Vertical component is reduced (×0.3) — horizontal gusts dominate
- Maximum gust magnitude: base_wind_speed × 0.6

### Gust Intensity by Weather
| Weather | Gust Intensity | Octaves | Notes |
|---------|---------------|---------|-------|
| Calm | 0.1 | 2 | Barely noticeable |
| Breezy | 0.3 | 3 | Moderate effect on flight |
| Windy | 0.6 | 3 | Significant flight deviation |
| Stormy | 0.9 | 4 | Extreme, disc fights wind |

## Thermal Model

Thermals are vertical updraft columns that occur outdoors on warm days:

```rust
pub struct ThermalLayer {
    pub thermals: Vec<Thermal>,
}

pub struct Thermal {
    pub center: DVec2,          // XZ position on field
    pub radius: f64,            // meters (3–8m typical)
    pub strength: f64,          // max updraft m/s (1–4 m/s)
    pub height_ceiling: f64,    // max height of effect (20–50m)
    pub drift_velocity: DVec2,  // thermals drift with wind
    pub lifetime: f64,          // seconds before dissipating
    pub birth_time: f64,
}

impl Thermal {
    pub fn sample(&self, pos: DVec3, t: f64) -> DVec3 {
        let age = t - self.birth_time;
        if age < 0.0 || age > self.lifetime { return DVec3::ZERO; }

        let center_now = self.center + self.drift_velocity * age;
        let dist_h = (DVec2::new(pos.x, pos.z) - center_now).length();
        let height_factor = (1.0 - pos.y / self.height_ceiling).max(0.0);

        // Gaussian radial profile
        let radial = (-dist_h.powi(2) / (2.0 * self.radius.powi(2))).exp();

        // Age envelope: ramp up, sustain, fade out
        let age_factor = smooth_envelope(age, self.lifetime, 0.1, 0.2);

        DVec3::new(0.0, self.strength * radial * height_factor * age_factor, 0.0)
    }
}
```

Thermals are primarily a flavor mechanic — they add unpredictability to high throws (hammers, scoobers) on hot outdoor days. They spawn and despawn procedurally during matches based on venue temperature.

## Obstacle Wake

Objects near the field (trees, buildings, bleachers) create wind shadows and turbulent wakes:

```rust
pub struct ObstacleWake {
    pub position: DVec3,        // obstacle center
    pub size: DVec3,            // bounding box
    pub porosity: f64,          // 0=solid wall, 1=open (trees ~0.5)
    pub wake_length: f64,       // how far downstream the wake extends
}

impl ObstacleWake {
    pub fn sample(&self, pos: DVec3, t: f64, base_wind_dir: DVec3) -> DVec3 {
        // Check if pos is in the wake cone downstream of obstacle
        let to_pos = pos - self.position;
        let downwind = to_pos.dot(base_wind_dir.normalize());

        if downwind < 0.0 || downwind > self.wake_length {
            return DVec3::ZERO;
        }

        // Wake expands with distance
        let wake_width = self.size.x * (1.0 + downwind / self.wake_length);
        let crosswind = (to_pos - base_wind_dir.normalize() * downwind).length();

        if crosswind > wake_width {
            return DVec3::ZERO;
        }

        // Reduce wind speed and add turbulence in wake
        let reduction = (1.0 - self.porosity) * (1.0 - downwind / self.wake_length);
        let turbulence = perlin_turbulence(pos, t) * reduction * 0.3;

        -base_wind_dir * reduction + turbulence
    }
}
```

Wake effects are pre-computed per venue as part of the venue definition (see [06-field-and-environment](06-field-and-environment.md)).

## GPU 3D Wind Texture

For GPU compute physics (see [01-disc-physics](01-disc-physics.md)), the wind field is baked into a 3D texture updated each frame:

### Texture Specification
| Property | Value |
|----------|-------|
| Resolution | 64 × 16 × 128 (width × height × depth) |
| Format | `Rgba16Float` (4 channels: XYZ wind + magnitude) |
| Coverage | Full field + 20m margin (130m × 30m × 60m) |
| Update rate | Every frame (60 Hz) |
| Filtering | Trilinear interpolation |

### Compute Shader — Wind Bake
```wgsl
// wind_bake.wgsl
@group(0) @binding(0) var<uniform> wind_params: WindParams;
@group(0) @binding(1) var output: texture_storage_3d<rgba16float, write>;

struct WindParams {
    base_direction: vec3<f32>,
    base_speed: f32,
    gust_intensity: f32,
    time: f32,
    noise_seed: u32,
    z0: f32,
    h_ref: f32,
    grid_origin: vec3<f32>,
    grid_scale: vec3<f32>,
}

@compute @workgroup_size(8, 4, 8)
fn bake_wind(@builtin(global_invocation_id) id: vec3<u32>) {
    let tex_size = textureDimensions(output);
    if any(id >= tex_size) { return; }

    let world_pos = wind_params.grid_origin + vec3<f32>(id) * wind_params.grid_scale;
    let height = world_pos.y;

    // Base wind with log profile
    let log_factor = log(max(height, 0.1) / wind_params.z0) / log(wind_params.h_ref / wind_params.z0);
    var wind = wind_params.base_direction * wind_params.base_speed * log_factor;

    // Gust noise (3-octave fBm)
    let noise_pos = world_pos / 20.0;
    let noise_t = wind_params.time / 5.0;
    wind.x += fbm3(noise_pos + vec3(0.0), noise_t, 3u, wind_params.noise_seed) * wind_params.gust_intensity;
    wind.y += fbm3(noise_pos + vec3(100.0), noise_t, 3u, wind_params.noise_seed) * wind_params.gust_intensity * 0.3;
    wind.z += fbm3(noise_pos + vec3(200.0), noise_t, 3u, wind_params.noise_seed) * wind_params.gust_intensity;

    let mag = length(wind);
    textureStore(output, id, vec4<f32>(wind, mag));
}
```

### CPU/GPU Consistency
The GPU texture is used ONLY for:
- Visual effects (grass sway, particle drift, flag motion)
- Audio (wind volume/pitch)
- Non-authoritative disc trail prediction

The authoritative disc physics always uses CPU computation for determinism. The GPU wind texture is an approximation used for rendering purposes.

## Wind Indicators

### In-Game UI
- **Wind arrow**: HUD element showing current wind direction and speed (see [13-ui-ux](13-ui-ux.md))
- **Wind sock**: 3D object at field edges that visually indicates wind direction/speed
- **Grass sway**: Procedural grass animation driven by wind texture
- **Particle drift**: Dust/leaves/rain particles moved by wind

### Wind Speed Categories (for UI display)
| Category | Speed (m/s) | Speed (mph) | Disc Effect |
|----------|------------|-------------|-------------|
| Calm | 0–2 | 0–4 | Negligible |
| Light | 2–5 | 4–11 | Slight drift |
| Moderate | 5–8 | 11–18 | Noticeable; adjustments needed |
| Strong | 8–12 | 18–27 | Major flight changes |
| Extreme | 12+ | 27+ | Disc may stall or flip |

## Match Wind Generation

At the start of a match, wind conditions are generated based on venue and weather settings:

```rust
pub struct MatchWindConfig {
    pub base_speed: f64,          // m/s
    pub base_direction: f64,      // radians (0 = +Z)
    pub gust_intensity: f64,      // 0–1
    pub thermals_enabled: bool,
    pub thermal_frequency: f64,   // spawns per minute
    pub drift_amplitude: f64,     // direction drift in radians
    pub drift_period: f64,        // seconds
    pub seed: u64,                // for determinism
}

impl MatchWindConfig {
    pub fn generate(venue: &Venue, weather: &WeatherCondition, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let (min_wind, max_wind) = venue.wind_range();
        let base_speed = rng.gen_range(min_wind..=max_wind);
        // ... etc
    }
}
```

### Half-Time Wind Shift
Between halves, the wind may shift direction by 0–30° (randomized), simulating natural weather evolution. Teams switch ends at half, so the wind advantage shifts.

## Determinism

For rollback netcode (see [10-netcode](10-netcode.md)):
- All noise functions use a seeded PRNG, not `rand::thread_rng()`
- The same `MatchWindConfig` seed produces identical wind on all clients
- Perlin noise implementation is custom (not library-dependent) to ensure cross-platform determinism
- Thermal spawn times and positions are pre-computed from the seed at match start
- Wind sampling at a given `(position, time)` is a pure function of the seed
- Runtime gameplay systems never "pull next random value" for wind; all stochastic values are derived from stable keys
