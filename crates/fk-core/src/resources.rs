use bevy::prelude::*;

use crate::types::{Difficulty, InputContext, Team};

#[derive(Resource, Debug, Clone)]
pub struct GameClock {
    pub match_time: f64,
    pub point_time: f64,
    pub half: u8,
    pub point_number: u32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            match_time: 0.0,
            point_time: 0.0,
            half: 1,
            point_number: 1,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Scoreboard {
    pub home_score: u32,
    pub away_score: u32,
    pub points_to_win: u32,
    pub cap: Option<u32>,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            home_score: 0,
            away_score: 0,
            points_to_win: 15,
            cap: None,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct StallCount {
    pub count: f32,
    pub active: bool,
}

impl Default for StallCount {
    fn default() -> Self {
        Self {
            count: 0.0,
            active: false,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MatchConfig {
    pub points_to_win: u32,
    pub time_cap_seconds: Option<f64>,
    pub wind_enabled: bool,
    pub difficulty: Difficulty,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            points_to_win: 15,
            time_cap_seconds: None,
            wind_enabled: true,
            difficulty: Difficulty::Casual,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameInput {
    pub move_dir: Vec2,
    pub aim_dir: Vec2,
    pub throw_power: f32,
    pub throw_charging: bool,
    pub throw_released: bool,
    pub throw_type: Option<frisbee_physics::ThrowType>,
    pub sprint: bool,
    pub layout_bid: bool,
    pub switch_player: bool,
    pub context: InputContext,
}

impl Default for GameInput {
    fn default() -> Self {
        Self {
            move_dir: Vec2::ZERO,
            aim_dir: Vec2::ZERO,
            throw_power: 0.0,
            throw_charging: false,
            throw_released: false,
            throw_type: None,
            sprint: false,
            layout_bid: false,
            switch_player: false,
            context: InputContext::Menu,
        }
    }
}

/// Which team the human player controls. Defaults to Home.
#[derive(Resource, Debug, Clone)]
pub struct HumanTeam(pub Team);

impl Default for HumanTeam {
    fn default() -> Self {
        Self(Team::Home)
    }
}
