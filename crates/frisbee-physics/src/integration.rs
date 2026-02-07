use crate::aerodynamics::compute_derivatives;
use crate::disc::{DiscParams, DiscState};
use crate::wind::WindField;

/// Classic fourth-order Runge-Kutta integrator.
pub struct RK4Integrator {
    /// Integration timestep (seconds). Default: 1/240.
    pub dt: f64,
}

impl RK4Integrator {
    /// Create an integrator with the default timestep of 1/240 s.
    pub fn new() -> Self {
        Self { dt: 1.0 / 240.0 }
    }

    /// Create an integrator with a custom timestep.
    pub fn with_dt(dt: f64) -> Self {
        Self { dt }
    }

    /// Advance the disc state by one timestep using RK4.
    ///
    /// k1 = f(t, y)
    /// k2 = f(t + dt/2, y + dt/2 * k1)
    /// k3 = f(t + dt/2, y + dt/2 * k2)
    /// k4 = f(t + dt,   y + dt   * k3)
    /// y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
    pub fn step(
        &self,
        state: &DiscState,
        params: &DiscParams,
        wind: &dyn WindField,
    ) -> DiscState {
        let dt = self.dt;
        let half_dt = dt * 0.5;

        // k1 = f(state)
        let k1 = compute_derivatives(state, params, wind);

        // k2 = f(state + 0.5 * dt * k1)
        let state2 = state.add_scaled(&k1, half_dt);
        let k2 = compute_derivatives(&state2, params, wind);

        // k3 = f(state + 0.5 * dt * k2)
        let state3 = state.add_scaled(&k2, half_dt);
        let k3 = compute_derivatives(&state3, params, wind);

        // k4 = f(state + dt * k3)
        let state4 = state.add_scaled(&k3, dt);
        let k4 = compute_derivatives(&state4, params, wind);

        // Combine: y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let combined = k1
            .add(&k2.scale(2.0))
            .add(&k3.scale(2.0))
            .add(&k4)
            .scale(1.0 / 6.0);

        state.add_scaled(&combined, dt)
    }
}

impl Default for RK4Integrator {
    fn default() -> Self {
        Self::new()
    }
}
