use crate::dependency::{DagConnection, DagNodeId};
use crate::story::story_graph::StoryGraph;
use crate::story::types::story_connection_attribute;
use mythic_dice::Dice;

pub fn generate_story_graph(_dice: &mut Dice, node_count: DagNodeId) -> StoryGraph {
	let mut story_graph = StoryGraph::default();
	let graph = &mut story_graph.graph;
	if node_count > 0 {
		graph.root = 1;
	}

	for i in 0..node_count {
		let id = i + 1;
		graph.nodes.insert(id);
		let previous_id = i;

		if previous_id > 0 {
			let connection_attributes = if previous_id == 1 {
				story_connection_attribute::DIRECT
			} else {
				story_connection_attribute::UNLOCKS
			};
			graph
				.connections
				.insert((previous_id, id), DagConnection::new(connection_attributes));
		}
	}

	// empty-input-sentinel: do not plant a fake endpoint when node_count == 0
	if node_count > 0 {
		story_graph.endpoints.insert(node_count);
	}

	story_graph
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn matches_unreal_assertions() {
		for node_count in 3..=5 {
			for seed in 1..=9 {
				let mut dice = Dice::new(seed);
				let story = generate_story_graph(&mut dice, node_count);
				assert_eq!(story.graph.nodes.len(), node_count as usize);
				for ((from, _to), connection) in &story.graph.connections {
					if *from == 1 {
						assert!(!connection.has_attribute(story_connection_attribute::UNLOCKS));
					} else {
						assert!(!connection.has_attribute(story_connection_attribute::DIRECT));
					}
				}
				assert_eq!(story.endpoints.len(), 1);
				assert!(story.endpoints.contains(&node_count));
			}
		}
	}

	#[test]
	fn empty_endpoints_for_zero_nodes() {
		let mut dice = Dice::new(1);
		let story = generate_story_graph(&mut dice, 0);
		assert!(story.endpoints.is_empty());
		assert!(story.graph.nodes.is_empty());
	}
}
