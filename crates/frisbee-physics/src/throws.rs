use glam::DVec3;

use crate::disc::{DiscState, Euler};

/// Catalogued throw types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Direction of disc spin when viewed from above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinDirection {
    /// Clockwise when viewed from above (negative omega_z).
    Clockwise,
    /// Counter-clockwise when viewed from above (positive omega_z).
    CounterClockwise,
}

/// Raw throw parameters before modification.
#[derive(Debug, Clone, Copy)]
pub struct ThrowParams {
    /// Release speed (m/s).
    pub speed: f64,
    /// Spin rate magnitude (rad/s). Always positive; sign comes from `spin_direction`.
    pub spin_rate: f64,
    /// Launch elevation angle (radians). Positive = upward.
    pub launch_angle: f64,
    /// Hyzer angle (radians). Positive = left edge down (for RHBH), negative = anhyzer.
    pub hyzer_angle: f64,
    /// Nose angle (radians). Positive = nose up, negative = nose down.
    pub nose_angle: f64,
    /// Spin direction.
    pub spin_direction: SpinDirection,
}

/// Adjustments applied on top of default throw parameters.
#[derive(Debug, Clone, Copy)]
pub struct ThrowModifications {
    /// Power factor in [0, 1]. 0.5 is baseline.
    pub power: f64,
    /// Horizontal aim offset (radians).
    pub aim_angle: f64,
    /// Additional hyzer/anhyzer (radians).
    pub hyzer_adjust: f64,
    /// Additional nose angle (radians).
    pub nose_adjust: f64,
}

impl Default for ThrowModifications {
    fn default() -> Self {
        Self {
            power: 0.5,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        }
    }
}

/// Helper: degrees to radians.
fn deg(d: f64) -> f64 {
    d * core::f64::consts::PI / 180.0
}

impl ThrowType {
    /// Return the default (unmodified) throw parameters for this throw type.
    pub fn default_params(&self) -> ThrowParams {
        match self {
            ThrowType::Backhand => ThrowParams {
                speed: 22.0,
                spin_rate: 62.8,
                launch_angle: deg(8.0),
                hyzer_angle: deg(15.0),
                nose_angle: deg(-3.0),
                spin_direction: SpinDirection::CounterClockwise,
            },
            ThrowType::Forehand => ThrowParams {
                speed: 20.0,
                spin_rate: 83.8,
                launch_angle: deg(6.0),
                hyzer_angle: deg(-10.0),
                nose_angle: deg(-2.0),
                spin_direction: SpinDirection::Clockwise,
            },
            ThrowType::Hammer => ThrowParams {
                speed: 18.0,
                spin_rate: 73.3,
                launch_angle: deg(60.0),
                hyzer_angle: deg(0.0),
                nose_angle: deg(-45.0),
                spin_direction: SpinDirection::Clockwise,
            },
            ThrowType::Scoober => ThrowParams {
                speed: 14.0,
                spin_rate: 52.4,
                launch_angle: deg(50.0),
                hyzer_angle: deg(0.0),
                nose_angle: deg(-40.0),
                spin_direction: SpinDirection::Clockwise,
            },
            ThrowType::Thumber => ThrowParams {
                speed: 17.0,
                spin_rate: 68.1,
                launch_angle: deg(55.0),
                hyzer_angle: deg(0.0),
                nose_angle: deg(45.0),
                spin_direction: SpinDirection::CounterClockwise,
            },
            ThrowType::Blade => ThrowParams {
                speed: 19.0,
                spin_rate: 78.5,
                launch_angle: deg(5.0),
                hyzer_angle: deg(-80.0),
                nose_angle: deg(0.0),
                spin_direction: SpinDirection::Clockwise,
            },
            ThrowType::PushPass => ThrowParams {
                speed: 10.0,
                spin_rate: 31.4,
                launch_angle: deg(3.0),
                hyzer_angle: deg(5.0),
                nose_angle: deg(2.0),
                spin_direction: SpinDirection::CounterClockwise,
            },
            ThrowType::ChickenWing => ThrowParams {
                speed: 12.0,
                spin_rate: 41.9,
                launch_angle: deg(10.0),
                hyzer_angle: deg(-20.0),
                nose_angle: deg(0.0),
                spin_direction: SpinDirection::Clockwise,
            },
        }
    }
}

/// Create an initial `DiscState` from a throw type, modifications, release
/// position, and the facing angle of the thrower.
///
/// `facing_angle` is in radians: 0 = throwing along +Z, PI/2 = along +X, etc.
/// The throw direction is rotated in the XZ horizontal plane.
///
/// Power scales:
/// - speed: +/- 30 % around baseline  (power 0 -> 0.7x, power 1 -> 1.3x)
/// - spin:  +/- 20 % around baseline  (power 0 -> 0.8x, power 1 -> 1.2x)
pub fn create_throw_state(
    throw_type: ThrowType,
    mods: &ThrowModifications,
    position: DVec3,
    facing_angle: f64,
) -> DiscState {
    let base = throw_type.default_params();

    // Power scaling: power in [0,1], 0.5 is neutral.
    // speed_factor = 1.0 + 0.6 * (power - 0.5) = 0.7 at 0, 1.0 at 0.5, 1.3 at 1.
    let speed_factor = 1.0 + 0.6 * (mods.power - 0.5);
    let spin_factor = 1.0 + 0.4 * (mods.power - 0.5);

    let speed = base.speed * speed_factor;
    let spin_mag = base.spin_rate * spin_factor;

    let launch_angle = base.launch_angle;
    let hyzer = base.hyzer_angle + mods.hyzer_adjust;
    let nose = base.nose_angle + mods.nose_adjust;

    // Signed spin rate (positive = CCW from above).
    let spin_rate = match base.spin_direction {
        SpinDirection::CounterClockwise => spin_mag,
        SpinDirection::Clockwise => -spin_mag,
    };

    // Total horizontal direction: facing + aim offset.
    let dir_angle = facing_angle + mods.aim_angle;

    // Velocity vector.
    // In our coordinate system Y is up.
    // Horizontal components in XZ plane:
    //   vx = speed * cos(launch_angle) * sin(dir_angle)
    //   vz = speed * cos(launch_angle) * cos(dir_angle)
    //   vy = speed * sin(launch_angle)
    let cos_launch = libm::cos(launch_angle);
    let sin_launch = libm::sin(launch_angle);
    let (sin_dir, cos_dir) = libm::sincos(dir_angle);

    let velocity = DVec3::new(
        speed * cos_launch * sin_dir,
        speed * sin_launch,
        speed * cos_launch * cos_dir,
    );

    // Orientation as Euler angles.
    //
    // Yaw = facing direction angle (rotation about Y / world up).
    // The disc faces the throw direction, so yaw = dir_angle.
    //
    // Pitch = nose angle. Positive pitch rotates the nose up.
    // Roll = hyzer angle. Positive roll tilts the left edge down.
    let orientation = Euler::new(hyzer, nose, dir_angle);

    // Compute the initial AoA from the velocity and disc orientation.
    // This is stored in angular_velocity.z for the decoupled AoA model.
    let normal = orientation.normal();
    let v_mag = velocity.length();
    let initial_alpha = if v_mag > 1e-9 {
        let sin_a = (velocity.dot(normal) / v_mag).clamp(-1.0, 1.0);
        libm::asin(sin_a)
    } else {
        0.0
    };

    DiscState {
        position,
        velocity,
        orientation,
        angular_velocity: DVec3::new(0.0, 0.0, initial_alpha),
        spin_rate,
        time: 0.0,
    }
}
