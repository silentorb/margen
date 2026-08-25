use crate::dependency::graph::{DagConnection, DagConnectionId, DagConnectionMap, DagNodeId};
use std::collections::{BTreeMap, VecDeque};

pub type DagConnections = Vec<(DagConnectionId, DagConnection)>;

fn get_immediate_node_connections(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
	match_second: bool,
) -> DagConnections {
	let mut result = DagConnections::new();
	for (key, value) in connections {
		let matched = if match_second { key.1 } else { key.0 };
		if matched == endpoint {
			result.push((*key, *value));
		}
	}
	result
}

fn get_immediate_node_connections_count(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
	match_second: bool,
) -> i32 {
	let mut result = 0;
	for key in connections.keys() {
		let matched = if match_second { key.1 } else { key.0 };
		if matched == endpoint {
			result += 1;
		}
	}
	result
}

fn get_layered_node_connections(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
	match_second: bool,
) -> DagConnectionMap {
	let mut result = BTreeMap::new();
	let mut queue = VecDeque::new();
	queue.push_back(endpoint);

	while let Some(node) = queue.pop_front() {
		let parents = get_immediate_node_connections(connections, node, match_second);
		if parents.is_empty() {
			break;
		}
		for (key, value) in &parents {
			result.insert(*key, *value);
		}
		for (key, _) in &parents {
			// incorrect-traversal-endpoint: enqueue the opposite end of the matched index
			let next = if match_second { key.0 } else { key.1 };
			queue.push_back(next);
		}
	}

	result
}

pub fn get_immediate_node_ancestors(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
) -> DagConnections {
	get_immediate_node_connections(connections, endpoint, true)
}

pub fn get_immediate_node_ancestors_count(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
) -> i32 {
	get_immediate_node_connections_count(connections, endpoint, true)
}

pub fn get_node_ancestors(connections: &DagConnectionMap, endpoint: DagNodeId) -> DagConnectionMap {
	get_layered_node_connections(connections, endpoint, true)
}

pub fn get_immediate_node_descendants(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
) -> DagConnections {
	get_immediate_node_connections(connections, endpoint, false)
}

pub fn get_immediate_node_descendants_count(
	connections: &DagConnectionMap,
	endpoint: DagNodeId,
) -> i32 {
	get_immediate_node_connections_count(connections, endpoint, false)
}

pub fn get_node_descendants(connections: &DagConnectionMap, endpoint: DagNodeId) -> DagConnectionMap {
	get_layered_node_connections(connections, endpoint, false)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dependency::graph::DagConnection;

	#[test]
	fn get_node_descendants_follows_chain() {
		let mut connections = DagConnectionMap::new();
		connections.insert((1, 2), DagConnection::default());
		connections.insert((2, 3), DagConnection::default());
		let descendants = get_node_descendants(&connections, 1);
		assert_eq!(descendants.len(), 2);
		assert!(descendants.contains_key(&(1, 2)));
		assert!(descendants.contains_key(&(2, 3)));
	}
}
