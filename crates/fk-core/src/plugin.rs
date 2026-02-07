use bevy::prelude::*;

use crate::events::*;
use crate::resources::*;
use crate::states::*;

pub struct FkCorePlugin;

impl Plugin for FkCorePlugin {
    fn build(&self, app: &mut App) {
        // Register states
        app.init_state::<AppState>();
        app.add_sub_state::<GamePhase>();

        // Register messages (buffered events)
        app.add_message::<DiscThrown>();
        app.add_message::<DiscCaught>();
        app.add_message::<DiscDropped>();
        app.add_message::<DiscOutOfBounds>();
        app.add_message::<DiscLanded>();
        app.add_message::<PointScored>();
        app.add_message::<TurnoverOccurred>();
        app.add_message::<StallOutEvent>();
        app.add_message::<PlayerSwitched>();
        app.add_message::<SwagEvent>();

        // Register resources with defaults
        app.init_resource::<GameClock>();
        app.init_resource::<Scoreboard>();
        app.init_resource::<StallCount>();
        app.init_resource::<MatchConfig>();
        app.init_resource::<GameInput>();
        app.init_resource::<HumanTeam>();
    }
}
