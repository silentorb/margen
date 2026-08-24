#pragma once

#include <unordered_set>

#include <margen/Core/WorldCell.h>
#include <margen/Dependency/DependencyGraph.h>
#include <margen/Structure/Story/Types.h>
#include <mythic/Math/Vector3.h>
#include <mythic/Mythic/Dice.h>

namespace margen {

bool storyToClusters(mythic::Dice& dice, const DagGraph& graph, const mythic::Vector3i& origin, CellGrid& grid,
                     SectorLocationMap& sectorMap, std::unordered_set<DagConnectionId, DagConnectionIdHash>& portalConnections);

} // namespace margen
