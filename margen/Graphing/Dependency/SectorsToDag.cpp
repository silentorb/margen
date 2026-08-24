#include <margen/Graphing/Dependency/SectorsToDag.h>

#include <algorithm>
#include <queue>

namespace margen {

namespace {

bool containsConnection(const SimpleDagConnections& container, const DagNodeId id) {
	return std::find(container.begin(), container.end(), id) != container.end();
}

void tryAddNeighborNode(const DagNodeId nodeId, SimpleDagNode& node, const DagNodeId neighborId, SimpleDagNode& neighbor,
                        const int containerIndex) {
	auto& container = node.connections[containerIndex];

	if (!containsConnection(container, neighborId)) {
		auto& neighborContainer = neighbor.connections[1 - containerIndex];
		container.push_back(neighborId);
		neighborContainer.push_back(nodeId);
	}
}

} // namespace

void sectorsToDag(const SectorData& sectorData, SimpleDagGraph& graph) {
	if (sectorData.sectors.empty())
		return;

	std::queue<SectorId> queue;
	queue.push(sectorData.origin);
	graph.nodes.emplace(sectorData.origin, SimpleDagNode{});

	while (!queue.empty()) {
		const auto sectorId = queue.front();
		queue.pop();
		const auto& sector = sectorData.sectors.at(sectorId);
		auto& node = graph.nodes.at(sectorId);

		for (const auto neighborId : sector.neighbors) {
			if (graph.nodes.find(neighborId) == graph.nodes.end()) {
				graph.nodes.emplace(neighborId, SimpleDagNode{.depth = node.depth + 1});
				queue.push(neighborId);
			} else {
				auto& neighbor = graph.nodes.at(neighborId);
				if (neighbor.depth != node.depth) {
					const auto containerIndex =
						neighbor.depth > node.depth ? SimpleDagNode::outputIndex : SimpleDagNode::inputIndex;

					tryAddNeighborNode(sectorId, node, neighborId, neighbor, containerIndex);
				}
			}
		}
	}
}

} // namespace margen
