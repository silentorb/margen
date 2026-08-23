#include <margen/Graphing/Dependency/SimpleDependencyGraph.h>

namespace margen {

void connectNodes(const DagNodeId nodeId, SimpleDagNode& node, const DagNodeId neighborId, SimpleDagNode& neighbor,
				  const int containerIndex) {
	auto& container = node.connections[containerIndex];

	if (!container.contains(neighborId)) {
		auto& neighborContainer = neighbor.connections[1 - containerIndex];
		container.push_back(neighborId);
		neighborContainer.push_back(nodeId);
	}
}

} // namespace margen
