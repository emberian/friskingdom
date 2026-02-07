use glam::DVec3;

use crate::disc::{DiscParams, DiscState};
use crate::integration::RK4Integrator;
use crate::wind::WindField;

/// Result of a complete flight simulation.
#[derive(Debug, Clone)]
pub struct FlightResult {
    /// Sampled states at ~60 Hz (every 4 integration steps at 240 Hz).
    pub states: Vec<DiscState>,
    /// Total flight time (seconds).
    pub flight_time: f64,
    /// Horizontal distance from start to landing (metres).
    pub distance: f64,
    /// Maximum height above the starting Y coordinate (metres).
    pub max_height: f64,
    /// Final landing position.
    pub landing_position: DVec3,
}

/// High-level simulator that owns an integrator and disc parameters.
pub struct Simulator {
    pub integrator: RK4Integrator,
    pub params: DiscParams,
}

impl Simulator {
    /// Create a simulator with default RK4 integrator and the given disc params.
    pub fn new(params: DiscParams) -> Self {
        Self {
            integrator: RK4Integrator::new(),
            params,
        }
    }

    /// Advance the disc state by one integration timestep.
    pub fn step(&self, state: &DiscState, wind: &dyn WindField) -> DiscState {
        self.integrator.step(state, &self.params, wind)
    }

    /// Run a full flight from `initial` until the disc hits the ground
    /// (`state.position.y <= ground_y`) or `max_time` is exceeded.
    ///
    /// States are recorded every 4 integration steps (i.e. at ~60 Hz when the
    /// integrator runs at 240 Hz).
    pub fn simulate_flight(
        &self,
        initial: DiscState,
        wind: &dyn WindField,
        max_time: f64,
        ground_y: f64,
    ) -> FlightResult {
        let start_pos = initial.position;
        let mut state = initial;
        let mut states = Vec::with_capacity(1024);
        let mut max_height: f64 = state.position.y - start_pos.y;
        let mut step_count: u64 = 0;

        // Record the initial state.
        states.push(state);

        loop {
            state = self.integrator.step(&state, &self.params, wind);
            step_count += 1;

            // Track max height.
            let height = state.position.y - start_pos.y;
            if height > max_height {
                max_height = height;
            }

            // Record every 4th step (~60 Hz).
            if step_count % 4 == 0 {
                states.push(state);
            }

            // Termination: hit the ground.
            if state.position.y <= ground_y {
                // Always record the final state.
                if step_count % 4 != 0 {
                    states.push(state);
                }
                break;
            }

            // Termination: max time exceeded.
            if state.time >= max_time {
                if step_count % 4 != 0 {
                    states.push(state);
                }
                break;
            }
        }

        // Horizontal distance (XZ plane).
        let dx = state.position.x - start_pos.x;
        let dz = state.position.z - start_pos.z;
        let distance = libm::sqrt(dx * dx + dz * dz);

        FlightResult {
            states,
            flight_time: state.time,
            distance,
            max_height,
            landing_position: state.position,
        }
    }
}
