use crate::dependency::{
	get_immediate_node_ancestors_count, get_node_ancestors, DagConnection, DagConnectionId,
	DagConnectionMap, DagGraph,
};
use crate::story::types::story_connection_attribute;
use mythic::Dice;

/*
 * Converts an abstract story DAG to a slightly less abstract branching graph.
 *
 * Goal: every switch/key requires some backtracking (no key right next to its gate).
 */

pub fn story_to_location_branching(
	dice: &mut Dice,
	graph: &DagGraph,
	max_connections_per_node: i32,
) -> DagConnectionMap {
	let mut new_connections = DagConnectionMap::new();

	for &node in &graph.nodes {
		let ancestors = get_node_ancestors(&graph.connections, node);
		let mut options: Vec<(DagConnectionId, DagConnection)> = Vec::new();

		for (key, connection) in &ancestors {
			if connection.has_attribute(story_connection_attribute::UNLOCKS) && key.1 == node {
				continue;
			}

			let count = get_immediate_node_ancestors_count(&new_connections, key.0);
			if count < max_connections_per_node {
				options.push((*key, *connection));
			}
		}

		if !options.is_empty() {
			let option = dice.from_slice(&options);
			let (from, _) = option.0;
			new_connections.insert((from, node), DagConnection::default());
		}
	}

	new_connections
}

pub fn story_to_location_branching_default(
	dice: &mut Dice,
	graph: &DagGraph,
) -> DagConnectionMap {
	story_to_location_branching(dice, graph, 4)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::story::generate_story_graph;

	#[test]
	fn matches_unreal_assertions() {
		for node_count in 3..=9 {
			for seed in 1..=2 {
				let mut dice = Dice::new(seed);
				let story = generate_story_graph(&mut dice, node_count);
				let branching =
					story_to_location_branching(&mut dice, &story.graph, 4);
				assert_eq!(branching.len(), (node_count - 1) as usize);
				assert!(!branching.contains_key(&(2, 3)));
			}
		}
	}
}
