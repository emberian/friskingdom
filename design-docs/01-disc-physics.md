# 01 — Disc Aerodynamics & Physics

## Overview
The `frisbee-physics` crate is a standalone Rust library modeling the flight of an ultimate frisbee disc. It has zero dependency on Bevy and can be used independently for simulations, tools, or testing. The crate implements a 6-DOF (six degrees of freedom) rigid body model with aerodynamic forces specific to a flying disc.

Related docs: [02-wind-field](02-wind-field.md), [04-player-controls](04-player-controls.md), [08-bevy-ecs-architecture](08-bevy-ecs-architecture.md), [10-netcode](10-netcode.md)

## Disc Physical Parameters

### Reference Disc (Ultrastar 175g)
| Parameter | Symbol | Value | Unit |
|-----------|--------|-------|------|
| Mass | m | 0.175 | kg |
| Diameter | d | 0.273 | m |
| Radius | r | 0.1365 | m |
| Reference area | A | 0.0585 | m² |
| Moment of inertia (spin axis) | I_z | 0.00122 | kg·m² |
| Moment of inertia (pitch/roll) | I_x = I_y | 0.000614 | kg·m² |
| Thickness | t | 0.032 | m |

## Coordinate System
- **World frame**: Right-handed; X = field width, Y = up, Z = field length
- **Disc body frame**: Origin at disc center of mass; z_body = spin axis (normal to disc face), x_body = forward in disc plane
- Euler angles: φ (roll), θ (pitch), ψ (yaw) — applied in ZYX order
- Spin rate: ω_z (positive = counterclockwise viewed from above for RHBH)

## Aerodynamic Force Model

### State Vector
The full disc state is a 12-element vector:
```
state = [x, y, z, vx, vy, vz, φ, θ, ψ, ωx, ωy, ωz]
```
Position (3) + linear velocity (3) + orientation (3 Euler angles) + angular velocity (3).

### Air-Relative Velocity
```
v_rel = v_disc - v_wind(position)
v_air = |v_rel|
```
The wind field is sampled at the disc's current position via the `WindField` trait (see [02-wind-field](02-wind-field.md)).

### Angle of Attack
The angle of attack α is the angle between the air-relative velocity vector and the disc plane:
```
α = arcsin( (v_rel · n̂) / |v_rel| )
```
where n̂ is the disc's spin-axis normal vector (z_body in world coords).

### Dynamic Pressure
```
q = 0.5 × ρ × v_air² × A
```
where ρ = 1.225 kg/m³ (sea level standard), A = reference area.

### Lift Force
```
C_L(α) = C_L0 + C_Lα × α
C_L0 = 0.15        (lift at zero AoA)
C_Lα = 1.4         (lift slope, per radian)
F_lift = q × C_L(α) × n̂_lift
```
n̂_lift is perpendicular to v_rel in the plane containing v_rel and n̂:
```
n̂_lift = normalize( n̂ - (n̂ · v̂_rel) × v̂_rel )
```

At high angles of attack (|α| > 45°), lift is reduced using a stall model:
```
C_L_stall(α) = C_L(45°) × sin(2α)    for |α| > 45°
```

### Drag Force
```
C_D(α) = C_D0 + C_Dα × α²
C_D0 = 0.08        (parasitic drag)
C_Dα = 2.72        (induced drag coefficient, per radian²)
F_drag = q × C_D(α) × (-v̂_rel)
```

### Pitching Moment
```
C_M(α) = C_M0 + C_Mα × α
C_M0 = -0.02       (pitching moment at zero AoA)
C_Mα = -0.02       (pitch stability derivative, per radian)
M_pitch = q × d × C_M(α)
```
Applied about the pitch axis (perpendicular to both spin axis and velocity).

### Spin Decay (Drag Torque)
```
C_spin = 0.004      (spin-down torque coefficient)
M_spin_drag = -C_spin × q × d × ω_z / |ω_z|    (opposes spin)
```

Spin decay rate:
```
dω_z/dt = M_spin_drag / I_z
```

### Gyroscopic Precession
A spinning disc acts as a gyroscope. Applied pitching moments cause precession rather than direct pitch change:
```
Ω_precession = M_pitch / (I_z × ω_z)
```
This precession rate is applied as yaw change perpendicular to the pitching moment axis, which is what causes the characteristic "turn and fade" of a backhand throw.

### Roll Moment (Magnus Effect)
The disc's spin interacting with airflow creates an asymmetric pressure distribution:
```
C_roll = 0.00005    (roll moment coefficient)
M_roll = C_roll × q × d × (ω_z × v_air)
```
This produces a gradual roll tendency based on spin direction and airspeed.

### Gravity
```
F_gravity = [0, -m × g, 0]     where g = 9.81 m/s²
```

### Total Force & Moment
```
F_total = F_lift + F_drag + F_gravity
M_total = M_pitch_precession + M_roll + M_spin_drag
```

## Numerical Integration — RK4

The crate uses 4th-order Runge-Kutta integration with a fixed timestep:

```rust
pub struct RK4Integrator {
    pub dt: f64,  // Default: 1/240 s (240 Hz physics)
}

impl RK4Integrator {
    pub fn step(
        &self,
        state: &DiscState,
        params: &DiscParams,
        wind: &dyn WindField,
        t: f64,
    ) -> DiscState {
        let k1 = self.derivatives(state, params, wind, t);
        let k2 = self.derivatives(&state.add_scaled(&k1, 0.5 * self.dt), params, wind, t + 0.5 * self.dt);
        let k3 = self.derivatives(&state.add_scaled(&k2, 0.5 * self.dt), params, wind, t + 0.5 * self.dt);
        let k4 = self.derivatives(&state.add_scaled(&k3, self.dt), params, wind, t + self.dt);

        state.add_scaled(&k1, self.dt / 6.0)
             .add_scaled(&k2, self.dt / 3.0)
             .add_scaled(&k3, self.dt / 3.0)
             .add_scaled(&k4, self.dt / 6.0)
    }
}
```

**Physics timestep**: 1/240s (240 Hz) for accurate disc flight. The game's fixed timestep runs at 60 Hz; the physics crate sub-steps 4× per game tick.

## Throw Catalog

Each throw type defines initial conditions applied to the disc state:

### Throw Parameters
```rust
pub struct ThrowParams {
    pub name: &'static str,
    pub speed: f32,          // m/s — release speed
    pub spin_rate: f32,      // rad/s — initial spin
    pub launch_angle: f32,   // rad — elevation angle
    pub hyzer_angle: f32,    // rad — disc tilt (negative = anhyzer)
    pub nose_angle: f32,     // rad — nose up/down
    pub spin_direction: SpinDirection, // CW or CCW
}
```

### Throw Types — Default Values (Power = 1.0, right-handed)

| Throw | Speed (m/s) | Spin (rad/s) | Launch (°) | Hyzer (°) | Nose (°) | Spin Dir |
|-------|------------|-------------|-----------|----------|---------|---------|
| Backhand | 22.0 | 62.8 (600rpm) | 8 | 15 | -3 | CCW |
| Forehand | 20.0 | 83.8 (800rpm) | 6 | -10 | -2 | CW |
| Hammer | 18.0 | 73.3 (700rpm) | 60 | 0 | -45 | CW |
| Scoober | 14.0 | 52.4 (500rpm) | 50 | 0 | -40 | CW |
| Thumber | 17.0 | 68.1 (650rpm) | 55 | 0 | 45 | CCW |
| Blade | 19.0 | 78.5 (750rpm) | 5 | -80 | 0 | CW |
| Push Pass | 10.0 | 31.4 (300rpm) | 3 | 5 | 2 | CCW |
| Chicken Wing | 12.0 | 41.9 (400rpm) | 10 | -20 | 0 | CW |

**Power scaling**: Player power stat (0.0–1.0) scales speed ±30% and spin ±20%. Accuracy stat adds random noise to angles.

### Throw Modification by Input
The player's input modifies throw parameters (see [04-player-controls](04-player-controls.md)):
- **Power** (0–1): Scales speed and spin
- **Aim** (angle): Rotates the throw direction in the horizontal plane
- **Release angle** (hyzer/anhyzer): Adjusts hyzer_angle
- **IO (inside-out / outside-in)**: Further adjusts hyzer_angle and release direction
- **Nose angle**: Adjusted by stick vertical position

## GPU Compute Path

For advanced scenarios (training AI, replay analysis, batch simulations), the physics can be offloaded to GPU compute shaders:

```wgsl
// disc_physics.wgsl — compute shader for parallel disc simulation
@group(0) @binding(0) var<storage, read_write> disc_states: array<DiscStateGPU>;
@group(0) @binding(1) var<uniform> params: DiscParamsGPU;
@group(0) @binding(2) var wind_texture: texture_3d<f32>;
@group(0) @binding(3) var wind_sampler: sampler;

struct DiscStateGPU {
    pos: vec3<f32>,
    vel: vec3<f32>,
    orientation: vec4<f32>,  // quaternion
    angular_vel: vec3<f32>,
    spin_rate: f32,
}

@compute @workgroup_size(64)
fn rk4_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if idx >= arrayLength(&disc_states) { return; }

    var state = disc_states[idx];
    let wind = textureSampleLevel(wind_texture, wind_sampler, state.pos / world_scale, 0.0).xyz;

    // RK4 sub-steps (same math as CPU path)
    let k1 = compute_derivatives(state, wind);
    let s2 = advance_state(state, k1, 0.5 * DT);
    let k2 = compute_derivatives(s2, wind);
    let s3 = advance_state(state, k2, 0.5 * DT);
    let k3 = compute_derivatives(s3, wind);
    let s4 = advance_state(state, k3, DT);
    let k4 = compute_derivatives(s4, wind);

    disc_states[idx] = rk4_combine(state, k1, k2, k3, k4, DT);
}
```

The GPU path runs identically to the CPU path for determinism. It is NOT used during online multiplayer (CPU path ensures bit-identical results across machines).

## Crate Public API

```rust
// frisbee-physics/src/lib.rs

pub struct DiscState {
    pub position: DVec3,
    pub velocity: DVec3,
    pub orientation: DEuler,       // φ, θ, ψ
    pub angular_velocity: DVec3,
    pub spin_rate: f64,            // ω_z
    pub time: f64,
}

pub struct DiscParams {
    pub mass: f64,
    pub diameter: f64,
    pub area: f64,
    pub i_spin: f64,               // I_z
    pub i_pitch: f64,              // I_x = I_y
    pub cl0: f64,                  // C_L0
    pub cla: f64,                  // C_Lα
    pub cd0: f64,                  // C_D0
    pub cda: f64,                  // C_Dα
    pub cm0: f64,                  // C_M0
    pub cma: f64,                  // C_Mα
    pub c_spin: f64,               // spin drag
    pub c_roll: f64,               // Magnus roll
    pub air_density: f64,
}

impl DiscParams {
    pub fn ultrastar() -> Self { /* standard 175g disc */ }
}

pub trait WindField: Send + Sync {
    fn sample(&self, position: DVec3, time: f64) -> DVec3;
}

pub struct NullWind;
impl WindField for NullWind {
    fn sample(&self, _: DVec3, _: f64) -> DVec3 { DVec3::ZERO }
}

pub struct Simulator {
    pub integrator: RK4Integrator,
    pub params: DiscParams,
}

impl Simulator {
    pub fn new(params: DiscParams) -> Self;
    pub fn step(&self, state: &DiscState, wind: &dyn WindField) -> DiscState;
    pub fn simulate_flight(
        &self,
        initial: DiscState,
        wind: &dyn WindField,
        max_time: f64,
        ground_y: f64,
    ) -> FlightPath;
}

pub struct FlightPath {
    pub states: Vec<DiscState>,
    pub flight_time: f64,
    pub distance: f64,
    pub max_height: f64,
    pub landing_position: DVec3,
}

pub fn throw(throw_type: ThrowType, power: f64, modifications: ThrowModifications) -> DiscState;

pub enum ThrowType {
    Backhand,
    Forehand,
    Hammer,
    Scoober,
    Thumber,
    Blade,
    PushPass,
    ChickenWing,
}
```

## Flight Behavior Summary

| Throw | Typical Flight Path | Key Physics |
|-------|-------------------|-------------|
| Backhand (RHBH) | Turns right, then fades left | Precession from pitching moment; CCW spin → right turn → left fade |
| Forehand (RHFH) | Turns left, then fades right | CW spin → left turn → right fade |
| Hammer | Arcs high, flattens, falls left | High launch angle, inverted; precession flips disc |
| Scoober | Floaty arc, flattens | Similar to hammer but lower speed, lower arc |
| Blade | Cuts vertically through air | Near-90° hyzer; minimal lift, knife-edge flight |
| IO Backhand | Initial right, holds angle | Anhyzer release fights natural fade |
| OI Forehand | Sharp left break | Hyzer release amplifies natural turn |

## Collision & Catch Detection

### Ground Collision
```
if disc.position.y <= ground_height(disc.position.xz) + disc.thickness/2 {
    // Ground hit — check angle and speed for skip vs stop
    if v_vertical > SKIP_THRESHOLD && angle_to_ground < SKIP_ANGLE {
        apply_skip_bounce();  // disc skips off ground
    } else {
        disc.grounded = true;  // turnover
    }
}
```

### Catch Volume
Catch detection uses a cylindrical volume around each player's hands:
- **Catch radius**: 0.5m (centered on hand position from animation)
- **Catch height**: 0.4m
- **Catch difficulty**: Based on disc speed, player skill, and disc position relative to body
- **Layout extension**: Diving adds 1.5m to effective reach, with lower catch probability

### Catch Probability
```
p_catch = base_skill × position_factor × speed_factor × contested_factor
```
- `base_skill`: Player's catching stat (0.5–1.0)
- `position_factor`: 1.0 at chest, decreasing toward extremes
- `speed_factor`: 1.0 below 15 m/s, decreasing above
- `contested_factor`: 0.6 if defender within 1m

## Determinism Requirements

For rollback netcode (see [10-netcode](10-netcode.md)):
- All floating-point operations use `f64` in the physics crate
- No platform-dependent math (no `fast_math`, no auto-vectorization that changes results)
- Fixed iteration counts in all loops
- Wind field sampling must be deterministic given the same seed
- The `DiscState` struct implements a custom hash for state verification
