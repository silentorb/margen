//! Story and DAG graphing for world generation.

pub mod dependency;
pub mod graphing;
pub mod story;

pub use dependency::{
	get_immediate_node_ancestors, get_immediate_node_ancestors_count, get_immediate_node_descendants,
	get_node_ancestors, get_node_descendants, DagConnection, DagConnectionId, DagConnectionMap,
	DagGraph, DagNodeId,
};
pub use graphing::{
	create_gated_waypoints, sectors_to_dag, waypoints_to_switches, SimpleDagGraph, SimpleDagNode,
};
pub use story::{
	generate_story_graph, story_connection_attribute, story_to_location_branching,
	story_to_location_branching_default, StoryGraph,
};
