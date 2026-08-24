#include <margen/Structure/Story/StoryToClusters.h>

#include <margen/Core/Directions.h>
#include <margen/Core/WorldCell.h>
#include <margen/Dependency/Querying.h>
#include <margen/Structure/WindingPath/GridPopulation.h>

#include <queue>

namespace margen {

namespace {

void addSectorNode(const DagNodeId node, const mythic::Vector3i& origin, CellGrid& grid, SectorLocationMap& sectorMap) {
	auto& cell = grid[origin];
	cell.sector = node;
	cell.addAttributes(CellAttribute::path);
	sectorMap.emplace(node, origin);
}

NeighborsArray getNeighborOptions(const CellGrid& grid, const mythic::Vector3i& origin) {
	NeighborsArray options;

	for (const auto& offset : getSurroundingOffsets()) {
		const auto location = origin + offset;
		if (grid.find(location) == grid.end()) {
			options.push_back(location);
		}
	}

	return options;
}

mythic::Vector3i selectDistantCell(const mythic::Vector3i& origin, const CellGrid& grid, mythic::Dice& dice) {
	const auto direction = dice.getInt(5);
	const auto directionVector = getGridDirectionVector(static_cast<DirectionIndex>(direction));
	for (int i = 0; i < 1000; ++i) {
		const auto location = origin + directionVector * i;
		if (grid.find(location) == grid.end()) {
			return location;
		}
	}

	return mythic::Vector3i::NoneValue();
}

} // namespace

/*
 * This converts between abstract story nodes to spatial nodes.
 */
bool storyToClusters(mythic::Dice& dice, const DagGraph& graph, const mythic::Vector3i& origin, CellGrid& grid,
                     SectorLocationMap& sectorMap, std::unordered_set<DagConnectionId, DagConnectionIdHash>& portalConnections) {
	std::queue<DagNodeId> queue;
	queue.push(graph.root);

	addSectorNode(graph.root, origin, grid, sectorMap);

	while (!queue.empty()) {
		const auto node = queue.front();
		queue.pop();
		const auto location = sectorMap.at(node);
		const auto descendants = getImmediateNodeDescendants(graph.connections, node);
		auto locationOptions = getNeighborOptions(grid, location);

		for (const auto& descendant : descendants) {
			const auto key = descendant.first.second;
			if (sectorMap.find(key) != sectorMap.end())
				continue;

			mythic::Vector3i neighborLocation;

			if (!locationOptions.empty()) {
				neighborLocation = dice.removeFromArray(locationOptions);
				connectCellsWithPath(grid, location, neighborLocation);
			} else {
				neighborLocation = selectDistantCell(location, grid, dice);
				portalConnections.insert({node, key});
			}

			if (neighborLocation == mythic::Vector3i::NoneValue()) {
				return false;
			}

			addSectorNode(key, neighborLocation, grid, sectorMap);
			queue.push(key);
		}
	}

	return true;
}

} // namespace margen
