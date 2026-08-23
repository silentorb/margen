#include <margen/Story/GenerateStory.h>

#include <margen/Dependency/DependencyGraph.h>
#include <margen/Story/StoryGraph.h>
#include <margen/Story/Types.h>
#include <mythic/Mythic/Dice.h>

namespace margen {

StoryGraph generateStoryGraph(mythic::Dice& dice, const DagNodeId nodeCount) {
	(void)dice;

	StoryGraph storyGraph;
	auto& graph = storyGraph.graph;
	if (nodeCount > 0) {
		graph.root = 1;
	}

	for (DagNodeId i = 0; i < nodeCount; i++) {
		const DagNodeId id = static_cast<DagNodeId>(i + 1);
		graph.nodes.emplace(id);
		const auto previousId = i;

		if (previousId > 0) {
			const auto connectionAttributes =
				previousId == 1
					? StoryConnectionAttribute::direct
					: StoryConnectionAttribute::unlocks;

			graph.connections.emplace(DagConnectionId{previousId, id}, DagConnection{connectionAttributes});
		}
	}

	// empty-input-sentinel: do not plant a fake endpoint when nodeCount == 0
	if (nodeCount > 0) {
		storyGraph.endpoints.insert(nodeCount);
	}

	return storyGraph;
}

} // namespace margen
