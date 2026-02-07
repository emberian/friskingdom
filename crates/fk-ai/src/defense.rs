use bevy::prelude::*;
use fk_core::components::*;
use fk_core::types::*;

// ---------------------------------------------------------------------------
// System: assign each unassigned defender to an offensive opponent (man D)
// ---------------------------------------------------------------------------

pub fn assign_defenders_system(
    mut commands: Commands,
    unassigned: Query<
        (Entity, &TeamMember),
        (With<AiControlled>, With<OnDefense>, Without<DefenseAssignment>),
    >,
    already_assigned: Query<&DefenseAssignment, With<OnDefense>>,
    opponents: Query<(&Player, &TeamMember), With<OnOffense>>,
) {
    // Bail out fast when nothing to do.
    if unassigned.is_empty() {
        return;
    }

    // Collect IDs of opponents that are already being marked.
    let taken: Vec<PlayerId> = already_assigned.iter().map(|da| da.marking).collect();

    // Build a list of available opponents.
    let mut available: Vec<(PlayerId, Team)> = opponents
        .iter()
        .filter(|(p, _)| !taken.contains(&p.id))
        .map(|(p, tm)| (p.id, tm.team))
        .collect();

    for (entity, team_member) in &unassigned {
        // Find an opponent on the opposite team.
        if let Some(idx) = available
            .iter()
            .position(|(_, t)| *t == team_member.team.opposite())
        {
            let (opponent_id, _) = available.remove(idx);
            commands
                .entity(entity)
                .insert(DefenseAssignment { marking: opponent_id });
        }
    }
}

// ---------------------------------------------------------------------------
// System: drive defensive positioning based on man-defense assignments
// ---------------------------------------------------------------------------

pub fn defense_ai_system(
    disc_q: Query<&Position, With<Disc>>,
    opponents: Query<(&Player, &Position, &Velocity), With<OnOffense>>,
    mut defenders: Query<
        (&mut AiMoveTarget, &DefenseAssignment, &Position),
        (With<AiControlled>, With<OnDefense>),
    >,
) {
    let disc_pos = match disc_q.iter().next() {
        Some(p) => p.0,
        None => return,
    };

    for (mut ai_move, assignment, _def_pos) in &mut defenders {
        // Find the opponent we are marking.
        let opponent = opponents
            .iter()
            .find(|(p, _, _)| p.id == assignment.marking);

        let (_, opp_pos, opp_vel) = match opponent {
            Some(o) => o,
            None => continue, // opponent entity may have despawned
        };

        let opp_p = opp_pos.0;

        // Position between the opponent and the disc: 70% toward opponent, 30% toward disc.
        let mut target = opp_p * 0.7 + disc_pos * 0.3;

        // Offset slightly toward the centre of the field from the opponent (force side).
        // This pushes the defender 1-2 m toward x=0 relative to the opponent.
        let force_offset = if opp_p.x > 0.0 { -1.5 } else { 1.5 };
        target.x += force_offset;

        // Keep on the ground.
        target.y = 0.0;

        // Clamp inside the field rectangle with a small margin.
        target.x = target.x.clamp(-17.0, 17.0);
        target.z = target.z.clamp(2.0, 98.0);

        // Sprint when the opponent is sprinting (velocity above walk threshold).
        let opp_speed = opp_vel.0.length();
        let opponent_sprinting = opp_speed > 6.0;

        ai_move.target = target;
        ai_move.sprint = opponent_sprinting;
    }
}
