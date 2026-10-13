use crate::placement::{ActorPlacement, ActorPlacements};
use crate::PLACEABLE_PLAYER_SPAWN;
use mythic_math::Vector3i;

/// Inputs needed for the skeletal populate pass (expand when slots/distribution land).
#[derive(Clone, Debug)]
pub struct PopulateInput {
	pub player_start: Vector3i,
}

/// Append a single player-spawn placement at `player_start`.
pub fn append_player_start_placement(player_start: Vector3i, placements: &mut ActorPlacements) {
	placements.push(ActorPlacement::new(player_start, PLACEABLE_PLAYER_SPAWN));
}

/// Build placements containing only the player-spawn marker.
pub fn populate_player_start(player_start: Vector3i) -> ActorPlacements {
	let mut placements = ActorPlacements::new();
	append_player_start_placement(player_start, &mut placements);
	placements
}

/// Top-level populate (Unreal `populateWorld` math side). First pass: player spawn only.
pub fn populate_world(input: &PopulateInput) -> ActorPlacements {
	populate_player_start(input.player_start)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::PLACEABLE_PLAYER_SPAWN;
	use mythic_math::Vector3i;

	#[test]
	fn populate_player_start_one_placement_at_origin() {
		let start = Vector3i::zero();
		let placements = populate_player_start(start);
		assert_eq!(placements.len(), 1);
		assert_eq!(placements[0].placeable_id, PLACEABLE_PLAYER_SPAWN);
		assert_eq!(placements[0].locations, vec![start]);
		assert_eq!(placements[0].sector, 0);
		assert_eq!(placements[0].biome_id, 0);
	}

	#[test]
	fn populate_world_delegates_to_player_start() {
		let start = Vector3i::new(3, -1, 2);
		let placements = populate_world(&PopulateInput {
			player_start: start,
		});
		assert_eq!(placements.len(), 1);
		assert_eq!(placements[0].locations, vec![start]);
		assert_eq!(placements[0].placeable_id, PLACEABLE_PLAYER_SPAWN);
	}
}
