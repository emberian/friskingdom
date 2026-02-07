use glam::DVec3;

/// Trait for spatial/temporal wind fields.
pub trait WindField: Send + Sync {
    /// Sample the wind velocity at a given world position and time.
    fn sample(&self, position: DVec3, time: f64) -> DVec3;
}

/// No wind at all.
pub struct NullWind;

impl WindField for NullWind {
    fn sample(&self, _position: DVec3, _time: f64) -> DVec3 {
        DVec3::ZERO
    }
}

/// Spatially uniform, time-invariant wind.
pub struct ConstantWind {
    pub velocity: DVec3,
}

impl ConstantWind {
    pub fn new(velocity: DVec3) -> Self {
        Self { velocity }
    }
}

impl WindField for ConstantWind {
    fn sample(&self, _position: DVec3, _time: f64) -> DVec3 {
        self.velocity
    }
}
