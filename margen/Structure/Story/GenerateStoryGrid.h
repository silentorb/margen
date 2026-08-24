#pragma once

#include <unordered_set>

#include <margen/Core/WorldCell.h>
#include <margen/Dependency/DependencyGraph.h>
#include <margen/Story/GenerateStory.h>
#include <margen/Story/LocationBranching.h>
#include <margen/Structure/Story/StoryToClusters.h>
#include <margen/Structure/Story/Types.h>
#include <mythic/Mythic/Dice.h>

namespace margen {

struct StoryGridResult {
	DagGraph graph;
	CellGrid grid;
	SectorLocationMap sectorLocationMap;
	std::unordered_set<DagConnectionId, DagConnectionIdHash> portalConnections;
	bool success = false;
};

StoryGridResult generateStoryGrid(mythic::Dice& dice, DagNodeId nodeCount, const mythic::Vector3i& origin = mythic::Vector3i::ZeroValue());

} // namespace margen
