use glam::DVec3;

use crate::disc::{DiscDerivatives, DiscParams, DiscState};
use crate::wind::WindField;

/// Gravitational acceleration (m/s^2).
const G: f64 = 9.81;

/// Small epsilon to avoid division by zero.
const EPSILON: f64 = 1e-9;

/// 45 degrees in radians -- stall threshold.
const STALL_ALPHA: f64 = core::f64::consts::FRAC_PI_4;

/// Compute all aerodynamic forces, moments, and resulting state derivatives.
///
/// Coordinate system: Y is up, XZ is the ground plane.
///
/// The angle of attack is stored as `state.angular_velocity.z` and evolved
/// independently from the velocity direction. This prevents the positive-
/// feedback loop between lift, vertical velocity, and AoA that causes
/// instantaneous stall in coupled models.
///
/// The disc orientation is reconstructed each step from:
///   - roll: hyzer angle (from Magnus effect)
///   - pitch: gamma + alpha (slaved to flight path + AoA)
///   - yaw: heading (from precession)
pub fn compute_derivatives(
    state: &DiscState,
    params: &DiscParams,
    wind: &dyn WindField,
) -> DiscDerivatives {
    // -----------------------------------------------------------------------
    // 1. Air-relative velocity
    // -----------------------------------------------------------------------
    let wind_vel = wind.sample(state.position, state.time);
    let v_rel = state.velocity - wind_vel;
    let v_rel_mag = v_rel.length();

    // If the disc is effectively stationary in the air, only gravity acts.
    if v_rel_mag < EPSILON {
        let gravity_accel = DVec3::new(0.0, -G, 0.0);
        return DiscDerivatives {
            d_position: state.velocity,
            d_velocity: gravity_accel,
            d_roll: 0.0,
            d_pitch: 0.0,
            d_yaw: 0.0,
            d_angular_velocity: DVec3::ZERO,
            d_spin_rate: 0.0,
        };
    }

    let v_rel_hat = v_rel / v_rel_mag;

    // -----------------------------------------------------------------------
    // 2. Angle of attack (from state, not from velocity/orientation)
    // -----------------------------------------------------------------------
    // The AoA is stored in angular_velocity.z and evolved by the pitching
    // moment. This decouples the AoA from the flight path angle changes.
    let alpha = state.angular_velocity.z;

    // Reconstruct the disc normal from the AoA and velocity direction.
    // The normal is in the plane of (v_rel, up), tilted by alpha from
    // perpendicular-to-velocity toward the up direction.
    //
    // First, find the "up" direction perpendicular to velocity:
    let world_up = DVec3::Y;
    let side = v_rel_hat.cross(world_up);
    let side_len = side.length();
    let (perp_up, _side_hat) = if side_len > EPSILON {
        let s = side / side_len;
        // perp_up is the component of world_up perpendicular to v_rel
        let pu = s.cross(v_rel_hat);
        (pu, s)
    } else {
        // Velocity is vertical -- use an arbitrary perpendicular
        (DVec3::Z, DVec3::X)
    };

    // Disc normal without roll, tilted by alpha from perp_up toward v_rel_hat.
    // alpha=0 -> normal = perp_up (perpendicular to velocity, upward).
    // alpha>0 -> positive AoA, air hits bottom, normal tilts into velocity.
    let (sa, ca) = libm::sincos(alpha);
    let normal_base = perp_up * ca + v_rel_hat * sa;

    // Apply roll (hyzer) by rotating the normal about the velocity axis
    let roll = state.orientation.roll;
    let (sr, cr) = libm::sincos(roll);
    // Rotate normal_base about v_rel_hat by angle roll
    // Using Rodrigues' formula: v_rot = v*cos(r) + (k x v)*sin(r) + k*(k.v)*(1-cos(r))
    let normal = normal_base * cr + v_rel_hat.cross(normal_base) * sr
        + v_rel_hat * v_rel_hat.dot(normal_base) * (1.0 - cr);

    // -----------------------------------------------------------------------
    // 3. Dynamic pressure times area
    // -----------------------------------------------------------------------
    let q = 0.5 * params.air_density * v_rel_mag * v_rel_mag * params.area;

    // -----------------------------------------------------------------------
    // 4. Lift
    // -----------------------------------------------------------------------
    let cl = {
        let cl_linear = params.cl0 + params.cla * alpha;
        if libm::fabs(alpha) > STALL_ALPHA {
            let cl_at_stall = params.cl0 + params.cla * STALL_ALPHA;
            cl_at_stall * libm::sin(2.0 * alpha)
        } else {
            cl_linear
        }
    };

    // Lift direction: perpendicular to v_rel, in the v_rel-normal plane.
    let n_proj = normal - v_rel_hat * normal.dot(v_rel_hat);
    let n_proj_len = n_proj.length();
    let lift_dir = if n_proj_len > EPSILON {
        n_proj / n_proj_len
    } else {
        perp_up
    };

    let lift_force = lift_dir * (cl * q);

    // -----------------------------------------------------------------------
    // 5. Drag
    // -----------------------------------------------------------------------
    let cd = params.cd0 + params.cda * alpha * alpha;
    let drag_force = -v_rel_hat * (cd * q);

    // -----------------------------------------------------------------------
    // 6. Gravity
    // -----------------------------------------------------------------------
    let gravity_force = DVec3::new(0.0, -params.mass * G, 0.0);

    // -----------------------------------------------------------------------
    // Total translational acceleration
    // -----------------------------------------------------------------------
    let total_force = lift_force + drag_force + gravity_force;
    let accel = total_force / params.mass;

    // -----------------------------------------------------------------------
    // 7. Angular dynamics
    // -----------------------------------------------------------------------
    let d = params.diameter;
    let m_pitch = q * d * (params.cm0 + params.cma * alpha);

    // --- AoA evolution (stored in d_angular_velocity.z) ---
    // The pitching moment changes the AoA, but the gyroscopic stiffness of
    // the spinning disc resists reorientation. The effective inertia is:
    //
    //   I_eff = I_pitch + L^2 / I_pitch,  where L = I_spin * omega_z
    //
    // At high spin (omega=60 rad/s), I_eff ~ 8.7 kg*m^2, so the AoA
    // changes by only ~0.6 deg/s. As spin decays, the disc becomes less
    // stable and AoA changes faster.
    let angular_momentum = params.i_spin * state.spin_rate;
    let gyro_stiffness = if params.i_pitch > EPSILON {
        angular_momentum * angular_momentum / params.i_pitch
    } else {
        0.0
    };
    let i_eff = params.i_pitch + gyro_stiffness;
    let d_alpha = if i_eff > EPSILON {
        m_pitch / i_eff
    } else {
        0.0
    };

    // --- Yaw: gyroscopic precession ---
    let d_yaw_precession = if libm::fabs(state.spin_rate) > EPSILON {
        m_pitch / (params.i_spin * state.spin_rate)
    } else {
        0.0
    };

    // --- Roll: Magnus effect ---
    let advance_ratio = state.spin_rate * d / (2.0 * v_rel_mag);
    let m_roll = params.c_roll * q * d * advance_ratio;
    let d_roll_magnus = if params.i_pitch > EPSILON {
        m_roll / params.i_pitch
    } else {
        0.0
    };

    // --- Pitch: slaved to gamma + alpha ---
    // The Euler pitch angle is set so the disc's geometric orientation
    // matches the velocity direction plus the current AoA. We compute
    // d_pitch as d(gamma)/dt + d(alpha)/dt.
    let v_horiz_sq = v_rel.x * v_rel.x + v_rel.z * v_rel.z;
    let v_horiz = libm::sqrt(v_horiz_sq);
    let gamma = if v_horiz > EPSILON {
        libm::atan2(v_rel.y, v_horiz)
    } else if v_rel.y > 0.0 {
        core::f64::consts::FRAC_PI_2
    } else if v_rel.y < 0.0 {
        -core::f64::consts::FRAC_PI_2
    } else {
        0.0
    };

    // Compute d(gamma)/dt from the acceleration
    // gamma = atan(v_y / v_h), so d_gamma = (a_y * v_h - v_y * a_h) / (v_h^2 + v_y^2)
    // where a_h is the horizontal acceleration component along v_h direction
    let v_h_dir = if v_horiz > EPSILON {
        DVec3::new(v_rel.x, 0.0, v_rel.z) / v_horiz
    } else {
        DVec3::Z
    };
    let a_h = accel.dot(v_h_dir);
    let d_gamma = if (v_horiz_sq + v_rel.y * v_rel.y) > EPSILON {
        (accel.y * v_horiz - v_rel.y * a_h) / (v_horiz_sq + v_rel.y * v_rel.y)
    } else {
        0.0
    };

    // Also correct for accumulated drift between the Euler pitch and
    // the desired pitch (gamma + alpha - accounting for roll).
    let cos_roll = libm::cos(state.orientation.roll);
    let target_pitch = if libm::fabs(cos_roll) > EPSILON {
        let sin_a = libm::sin(alpha);
        let arg = (sin_a / cos_roll).clamp(-1.0, 1.0);
        libm::asin(arg) - gamma
    } else {
        -gamma
    };
    let pitch_correction = (target_pitch - state.orientation.pitch) * 20.0;

    let d_pitch_total = d_gamma + d_alpha + pitch_correction;

    // --- Spin decay ---
    let spin_sign = if state.spin_rate > EPSILON {
        1.0
    } else if state.spin_rate < -EPSILON {
        -1.0
    } else {
        0.0
    };
    let m_spin = -params.c_spin * q * d * spin_sign;
    let d_spin_rate = if params.i_spin > EPSILON {
        m_spin / params.i_spin
    } else {
        0.0
    };

    // -----------------------------------------------------------------------
    // Assemble
    // -----------------------------------------------------------------------
    DiscDerivatives {
        d_position: state.velocity,
        d_velocity: accel,
        d_roll: d_roll_magnus,
        d_pitch: d_pitch_total,
        d_yaw: d_yaw_precession,
        // d_angular_velocity.z = d(alpha), the rest unused
        d_angular_velocity: DVec3::new(0.0, 0.0, d_alpha),
        d_spin_rate,
    }
}
