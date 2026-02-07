use glam::{DMat3, DVec3};

// ---------------------------------------------------------------------------
// Euler angles
// ---------------------------------------------------------------------------

/// Disc orientation as Euler angles in radians.
///
/// Convention: **Y-X-Z intrinsic** rotation order, designed so that the
/// intuitive disc angles map cleanly to the world frame (Y-up):
///
///   R = Ry(yaw) * Rx(pitch) * Rz(roll)
///
/// - **Yaw** (psi): heading angle, rotation about world Y (up). 0 = facing +Z.
/// - **Pitch** (theta): nose angle, rotation about the body X (lateral) axis.
///   Positive pitch = nose up.
/// - **Roll** (phi): hyzer angle, rotation about the body Z (forward) axis.
///   Positive roll = left edge down (for a right-hand backhand thrower).
///
/// Body frame: the disc lies in the body XZ plane, with body +Y as the spin
/// axis normal (pointing "up" from the top face). At identity orientation the
/// disc is flat and horizontal, facing +Z.
#[derive(Debug, Clone, Copy)]
pub struct Euler {
    /// Roll / hyzer (phi) -- rotation about body Z (forward) axis.
    pub roll: f64,
    /// Pitch / nose angle (theta) -- rotation about body X (lateral) axis.
    pub pitch: f64,
    /// Yaw / heading (psi) -- rotation about world Y (up) axis.
    pub yaw: f64,
}

impl Euler {
    pub fn new(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Build the Y-X-Z rotation matrix (body -> world).
    ///
    /// R = Ry(yaw) * Rx(pitch) * Rz(roll)
    pub fn to_rotation_matrix(&self) -> DMat3 {
        let (sr, cr) = libm::sincos(self.roll);
        let (sp, cp) = libm::sincos(self.pitch);
        let (sy, cy) = libm::sincos(self.yaw);

        // R = Ry(yaw) * Rx(pitch) * Rz(roll)
        //
        // Rx(p) = [1,   0,    0  ]    Ry(y) = [cy,  0, sy]    Rz(r) = [cr, -sr, 0]
        //         [0, cos(p), -sin(p)]         [0,   1,  0]            [sr,  cr, 0]
        //         [0, sin(p),  cos(p)]         [-sy, 0, cy]            [0,   0,  1]
        //
        // Let T = Rx(p) * Rz(r):
        //   T = [cr,        -sr,         0    ]
        //       [cp*sr,      cp*cr,      -sp  ]
        //       [sp*sr,      sp*cr,       cp  ]
        //
        // R = Ry(y) * T:
        //   R[0,0] = cy*cr + sy*sp*sr      R[0,1] = -cy*sr + sy*sp*cr     R[0,2] = sy*cp
        //   R[1,0] = cp*sr                 R[1,1] = cp*cr                  R[1,2] = -sp
        //   R[2,0] = -sy*cr + cy*sp*sr     R[2,1] = sy*sr + cy*sp*cr      R[2,2] = cy*cp

        DMat3::from_cols(
            DVec3::new(
                cy * cr + sy * sp * sr,
                cp * sr,
                -sy * cr + cy * sp * sr,
            ),
            DVec3::new(
                -cy * sr + sy * sp * cr,
                cp * cr,
                sy * sr + cy * sp * cr,
            ),
            DVec3::new(sy * cp, -sp, cy * cp),
        )
    }

    /// The disc's spin-axis normal (body +Y axis expressed in world coords).
    /// This is the second column of the rotation matrix.
    ///
    /// At identity orientation (all zeros) this returns (0, 1, 0) = world up.
    pub fn normal(&self) -> DVec3 {
        let (sr, cr) = libm::sincos(self.roll);
        let (sp, cp) = libm::sincos(self.pitch);
        let (sy, cy) = libm::sincos(self.yaw);

        DVec3::new(-cy * sr + sy * sp * cr, cp * cr, sy * sr + cy * sp * cr)
    }
}

// ---------------------------------------------------------------------------
// Disc physical parameters
// ---------------------------------------------------------------------------

/// Physical parameters for a disc model.
#[derive(Debug, Clone, Copy)]
pub struct DiscParams {
    /// Mass (kg).
    pub mass: f64,
    /// Diameter (m).
    pub diameter: f64,
    /// Reference planform area (m^2).
    pub area: f64,
    /// Moment of inertia about spin axis I_z (kg*m^2).
    pub i_spin: f64,
    /// Transverse moment of inertia I_x = I_y (kg*m^2).
    pub i_pitch: f64,
    /// Lift coefficient at zero angle of attack.
    pub cl0: f64,
    /// Lift slope (per radian).
    pub cla: f64,
    /// Parasitic drag coefficient.
    pub cd0: f64,
    /// Induced drag coefficient (per rad^2).
    pub cda: f64,
    /// Pitching moment at zero AoA.
    pub cm0: f64,
    /// Pitch stability derivative (per radian).
    pub cma: f64,
    /// Spin-down torque coefficient.
    pub c_spin: f64,
    /// Magnus roll moment coefficient.
    pub c_roll: f64,
    /// Air density rho (kg/m^3).
    pub air_density: f64,
}

impl DiscParams {
    /// Standard Ultrastar 175 g disc.
    pub fn ultrastar() -> Self {
        Self {
            mass: 0.175,
            diameter: 0.273,
            area: 0.0585,
            i_spin: 0.00122,
            i_pitch: 0.000614,
            cl0: 0.04,
            cla: 1.15,
            cd0: 0.10,
            cda: 2.72,
            cm0: -0.02,
            cma: -0.02,
            c_spin: 0.004,
            c_roll: 0.00005,
            air_density: 1.225,
        }
    }
}

// ---------------------------------------------------------------------------
// State derivatives (used by the integrator)
// ---------------------------------------------------------------------------

/// Derivatives of the disc state, used by the RK4 integrator.
#[derive(Debug, Clone, Copy)]
pub struct DiscDerivatives {
    /// d(position)/dt = velocity.
    pub d_position: DVec3,
    /// d(velocity)/dt = net force / mass.
    pub d_velocity: DVec3,
    /// d(roll)/dt, d(pitch)/dt, d(yaw)/dt -- Euler angle rates.
    pub d_roll: f64,
    pub d_pitch: f64,
    pub d_yaw: f64,
    /// d(angular_velocity)/dt = torque / I (for transverse axes).
    pub d_angular_velocity: DVec3,
    /// d(spin_rate)/dt = spin torque / I_z.
    pub d_spin_rate: f64,
}

impl DiscDerivatives {
    pub fn zero() -> Self {
        Self {
            d_position: DVec3::ZERO,
            d_velocity: DVec3::ZERO,
            d_roll: 0.0,
            d_pitch: 0.0,
            d_yaw: 0.0,
            d_angular_velocity: DVec3::ZERO,
            d_spin_rate: 0.0,
        }
    }

    /// Element-wise addition of two derivative sets.
    pub fn add(&self, other: &DiscDerivatives) -> DiscDerivatives {
        DiscDerivatives {
            d_position: self.d_position + other.d_position,
            d_velocity: self.d_velocity + other.d_velocity,
            d_roll: self.d_roll + other.d_roll,
            d_pitch: self.d_pitch + other.d_pitch,
            d_yaw: self.d_yaw + other.d_yaw,
            d_angular_velocity: self.d_angular_velocity + other.d_angular_velocity,
            d_spin_rate: self.d_spin_rate + other.d_spin_rate,
        }
    }

    /// Scalar multiply all derivative components.
    pub fn scale(&self, s: f64) -> DiscDerivatives {
        DiscDerivatives {
            d_position: self.d_position * s,
            d_velocity: self.d_velocity * s,
            d_roll: self.d_roll * s,
            d_pitch: self.d_pitch * s,
            d_yaw: self.d_yaw * s,
            d_angular_velocity: self.d_angular_velocity * s,
            d_spin_rate: self.d_spin_rate * s,
        }
    }
}

// ---------------------------------------------------------------------------
// Full disc state
// ---------------------------------------------------------------------------

/// Full 12-DOF disc state.
#[derive(Debug, Clone, Copy)]
pub struct DiscState {
    /// World position (m). Y is up.
    pub position: DVec3,
    /// World velocity (m/s).
    pub velocity: DVec3,
    /// Orientation as Euler angles.
    pub orientation: Euler,
    /// Angular velocity of the body frame (rad/s) -- transverse components.
    pub angular_velocity: DVec3,
    /// Spin rate about disc normal (rad/s). Positive = CCW viewed from above.
    pub spin_rate: f64,
    /// Simulation time (s).
    pub time: f64,
}

impl DiscState {
    /// Produce a new state by adding `derivatives * dt` to the current state.
    /// Used inside RK4 to build intermediate states.
    pub fn add_scaled(&self, derivatives: &DiscDerivatives, dt: f64) -> DiscState {
        DiscState {
            position: self.position + derivatives.d_position * dt,
            velocity: self.velocity + derivatives.d_velocity * dt,
            orientation: Euler {
                roll: self.orientation.roll + derivatives.d_roll * dt,
                pitch: self.orientation.pitch + derivatives.d_pitch * dt,
                yaw: self.orientation.yaw + derivatives.d_yaw * dt,
            },
            angular_velocity: self.angular_velocity + derivatives.d_angular_velocity * dt,
            spin_rate: self.spin_rate + derivatives.d_spin_rate * dt,
            time: self.time + dt,
        }
    }
}
