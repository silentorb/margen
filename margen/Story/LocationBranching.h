#pragma once

#include <margen/Dependency/DependencyGraph.h>

namespace mythic {
class Dice;
}

namespace margen {

/*
 * Max connections defaults to 4 because it is intended for flat hex grid use cases, (4 giving plenty of elbow room)
 * but this function is general enought to be used for other cases.
 */
DagConnectionMap storyToLocationBranching(mythic::Dice& dice, const DagGraph& graph,
										  int maxConnectionsPerNode = 4);

} // namespace margen
