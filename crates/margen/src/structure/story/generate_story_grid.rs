use crate::core::world_cell::CellGrid;
use crate::dependency::{DagConnectionId, DagGraph, DagNodeId};
use crate::story::{generate_story_graph, story_to_location_branching};
use crate::structure::story::story_to_clusters::{story_to_clusters, SectorLocationMap};
use mythic::{Dice, Vector3i};
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub struct StoryGridResult {
	pub graph: DagGraph,
	pub grid: CellGrid,
	pub sector_location_map: SectorLocationMap,
	pub portal_connections: HashSet<DagConnectionId>,
	pub success: bool,
}

pub fn generate_story_grid(
	dice: &mut Dice,
	node_count: DagNodeId,
	origin: Vector3i,
) -> StoryGridResult {
	let mut result = StoryGridResult::default();

	let story_graph = generate_story_graph(dice, node_count);
	let branching_connections = story_to_location_branching(dice, &story_graph.graph, 4);

	result.graph.root = story_graph.graph.root;
	result.graph.nodes = story_graph.graph.nodes;
	result.graph.connections = branching_connections;

	result.success = story_to_clusters(
		dice,
		&result.graph,
		origin,
		&mut result.grid,
		&mut result.sector_location_map,
		&mut result.portal_connections,
	);

	result
}

pub fn generate_story_grid_at_origin(dice: &mut Dice, node_count: DagNodeId) -> StoryGridResult {
	generate_story_grid(dice, node_count, Vector3i::zero())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn story_to_clusters_matches_unreal_assertions() {
		for node_count in 3..=8 {
			for seed in 1..=3 {
				let mut dice = Dice::new(seed);
				let result = generate_story_grid_at_origin(&mut dice, node_count);
				assert!(result.success, "seed={seed} nodes={node_count}");
				assert_eq!(result.grid.len(), node_count as usize);
				assert!(result.portal_connections.is_empty());
			}
		}
	}
}
