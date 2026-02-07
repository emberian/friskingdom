//! `frisbee-physics` -- standalone disc aerodynamics simulation.
//!
//! This crate has **zero** Bevy dependency. It uses only `glam` for math and
//! `libm` (via glam's feature) for deterministic floating-point functions.

pub mod aerodynamics;
pub mod disc;
pub mod integration;
pub mod simulator;
pub mod throws;
pub mod wind;

// Re-export public API at crate root for convenience.
pub use disc::{DiscDerivatives, DiscParams, DiscState, Euler};
/// Re-export the specific glam version used by this crate's types.
pub use glam::DVec3 as PhysDVec3;
pub use integration::RK4Integrator;
pub use simulator::{FlightResult, Simulator};
pub use throws::{
    create_throw_state, SpinDirection, ThrowModifications, ThrowParams, ThrowType,
};
pub use wind::{ConstantWind, NullWind, WindField};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    /// Helper: simulate a throw and return the flight result.
    fn simulate_throw(
        throw_type: ThrowType,
        power: f64,
        wind: &dyn WindField,
    ) -> FlightResult {
        let mods = ThrowModifications {
            power,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        };
        let release_pos = DVec3::new(0.0, 1.5, 0.0); // release at 1.5 m height
        let facing = 0.0; // throwing along +Z
        let initial = create_throw_state(throw_type, &mods, release_pos, facing);
        let params = DiscParams::ultrastar();
        let sim = Simulator::new(params);
        sim.simulate_flight(initial, wind, 10.0, 0.0)
    }

    #[test]
    fn test_backhand_flight() {
        let wind = NullWind;
        let result = simulate_throw(ThrowType::Backhand, 1.0, &wind);

        println!(
            "Backhand: dist={:.1}m, height={:.1}m, time={:.2}s, landing=({:.1}, {:.1}, {:.1})",
            result.distance,
            result.max_height,
            result.flight_time,
            result.landing_position.x,
            result.landing_position.y,
            result.landing_position.z,
        );

        // Distance should be roughly 30-60 m.
        assert!(
            result.distance > 30.0 && result.distance < 60.0,
            "Backhand distance {:.1} m out of expected range [30, 60]",
            result.distance
        );

        // Max height should be roughly 2-6 m above release.
        assert!(
            result.max_height > 2.0 && result.max_height < 6.0,
            "Backhand max height {:.1} m out of expected range [2, 6]",
            result.max_height
        );

        // Flight time should be roughly 2-4 s.
        assert!(
            result.flight_time > 2.0 && result.flight_time < 4.0,
            "Backhand flight time {:.2} s out of expected range [2, 4]",
            result.flight_time
        );

        // Backhand (CCW spin, positive hyzer) should fade LEFT.
        // In our coordinate system with facing along +Z and Y up, left is -X.
        assert!(
            result.landing_position.x < 0.0,
            "Backhand should fade left (negative X), got x={:.2}",
            result.landing_position.x
        );
    }

    #[test]
    fn test_forehand_flight() {
        let wind = NullWind;
        let result = simulate_throw(ThrowType::Forehand, 1.0, &wind);

        println!(
            "Forehand: dist={:.1}m, height={:.1}m, time={:.2}s, landing=({:.1}, {:.1}, {:.1})",
            result.distance,
            result.max_height,
            result.flight_time,
            result.landing_position.x,
            result.landing_position.y,
            result.landing_position.z,
        );

        // Should travel a reasonable distance.
        assert!(
            result.distance > 20.0 && result.distance < 60.0,
            "Forehand distance {:.1} m out of expected range [20, 60]",
            result.distance
        );

        // Forehand (CW spin, negative hyzer/anhyzer) should fade RIGHT.
        // Right is +X in our system.
        assert!(
            result.landing_position.x > 0.0,
            "Forehand should fade right (positive X), got x={:.2}",
            result.landing_position.x
        );
    }

    #[test]
    fn test_hammer_flight() {
        let wind = NullWind;
        let result = simulate_throw(ThrowType::Hammer, 1.0, &wind);

        println!(
            "Hammer: dist={:.1}m, height={:.1}m, time={:.2}s, landing=({:.1}, {:.1}, {:.1})",
            result.distance,
            result.max_height,
            result.flight_time,
            result.landing_position.x,
            result.landing_position.y,
            result.landing_position.z,
        );

        // Hammer should reach higher than a backhand.
        assert!(
            result.max_height > 8.0 && result.max_height < 20.0,
            "Hammer max height {:.1} m out of expected range [8, 20]",
            result.max_height
        );

        // Distance should be shorter than backhand (it's an upside-down throw).
        assert!(
            result.distance < 50.0,
            "Hammer distance {:.1} m unexpectedly large",
            result.distance
        );
    }

    #[test]
    fn test_no_wind_symmetry() {
        // Throw straight: zero hyzer, no wind. Should not drift much laterally.
        let mods = ThrowModifications {
            power: 0.5,
            aim_angle: 0.0,
            hyzer_adjust: -ThrowType::Backhand.default_params().hyzer_angle, // cancel default hyzer
            nose_adjust: 0.0,
        };
        let release_pos = DVec3::new(0.0, 1.5, 0.0);
        let facing = 0.0;
        let initial = create_throw_state(ThrowType::Backhand, &mods, release_pos, facing);
        let params = DiscParams::ultrastar();
        let sim = Simulator::new(params);
        let wind = NullWind;
        let result = sim.simulate_flight(initial, &wind, 10.0, 0.0);

        println!(
            "Symmetry: x_drift={:.2}m, dist={:.1}m",
            result.landing_position.x, result.distance
        );

        // Lateral drift should be small (< 2 m).
        assert!(
            libm::fabs(result.landing_position.x) < 2.0,
            "Symmetry test: lateral drift {:.2} m exceeds 2 m",
            result.landing_position.x
        );
    }

    #[test]
    fn test_wind_effect() {
        // Compare same throw in no-wind vs 10 m/s crosswind (+X direction).
        let mods = ThrowModifications {
            power: 0.5,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        };
        let release_pos = DVec3::new(0.0, 1.5, 0.0);
        let facing = 0.0;
        let params = DiscParams::ultrastar();

        let initial_no_wind =
            create_throw_state(ThrowType::Backhand, &mods, release_pos, facing);
        let sim = Simulator::new(params);
        let no_wind = NullWind;
        let result_no_wind = sim.simulate_flight(initial_no_wind, &no_wind, 10.0, 0.0);

        let initial_wind =
            create_throw_state(ThrowType::Backhand, &mods, release_pos, facing);
        let crosswind = ConstantWind::new(DVec3::new(10.0, 0.0, 0.0));
        let result_wind = sim.simulate_flight(initial_wind, &crosswind, 10.0, 0.0);

        let drift_no_wind = result_no_wind.landing_position.x;
        let drift_wind = result_wind.landing_position.x;
        let difference = libm::fabs(drift_wind - drift_no_wind);

        println!(
            "Wind effect: no_wind x={:.2}m, wind x={:.2}m, diff={:.2}m",
            drift_no_wind, drift_wind, difference
        );

        // The crosswind should cause significantly more lateral deviation.
        assert!(
            difference > 2.0,
            "Wind should cause >2 m additional lateral drift, got {:.2} m",
            difference
        );
    }

    #[test]
    fn test_disc_hits_ground() {
        let wind = NullWind;
        let result = simulate_throw(ThrowType::Backhand, 0.5, &wind);

        // Disc should hit the ground (y <= 0).
        assert!(
            result.landing_position.y <= 0.0,
            "Disc should hit ground, final y={:.3}",
            result.landing_position.y
        );

        // Flight time should be finite and positive.
        assert!(
            result.flight_time > 0.0 && result.flight_time < 10.0,
            "Flight time {:.2} s out of expected range",
            result.flight_time
        );
    }

}
