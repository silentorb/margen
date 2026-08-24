#include <margen/Structure/Story/GenerateStoryGrid.h>

#include <margen/Story/StoryGraph.h>

namespace margen {

StoryGridResult generateStoryGrid(mythic::Dice& dice, const DagNodeId nodeCount, const mythic::Vector3i& origin) {
	StoryGridResult result;

	const auto storyGraph = generateStoryGraph(dice, nodeCount);
	const auto branchingConnections = storyToLocationBranching(dice, storyGraph.graph);

	result.graph.root = storyGraph.graph.root;
	result.graph.nodes = storyGraph.graph.nodes;
	result.graph.connections = branchingConnections;

	result.success = storyToClusters(dice, result.graph, origin, result.grid, result.sectorLocationMap,
	                                 result.portalConnections);

	return result;
}

} // namespace margen
