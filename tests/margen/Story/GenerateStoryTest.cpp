#include <catch2/catch_test_macros.hpp>

#include <margen/Dependency/DependencyGraph.h>
#include <margen/Story/GenerateStory.h>
#include <margen/Story/StoryGraph.h>
#include <margen/Story/Types.h>
#include <mythic/Mythic/Dice.h>

TEST_CASE("generateStoryGraph matches Unreal assertions", "[margen][story]") {
	for (int nodeCount = 3; nodeCount < 6; nodeCount++) {
		for (int seed = 1; seed < 10; ++seed) {
			mythic::Dice dice(seed);
			const auto storyGraph = margen::generateStoryGraph(dice, static_cast<margen::DagNodeId>(nodeCount));
			const auto& graph = storyGraph.graph;

			REQUIRE(static_cast<int>(graph.nodes.size()) == nodeCount);

			int invalidDepth0Connections = 0;
			int invalidDepthPlus1Connections = 0;
			for (const auto& connection : graph.connections) {
				if (connection.first.first == 1 && connection.second.hasAttribute(margen::StoryConnectionAttribute::unlocks)) {
					++invalidDepth0Connections;
				}
				if (connection.first.first > 1 && connection.second.hasAttribute(margen::StoryConnectionAttribute::direct)) {
					++invalidDepthPlus1Connections;
				}
			}

			REQUIRE(invalidDepth0Connections == 0);
			REQUIRE(invalidDepthPlus1Connections == 0);
			REQUIRE(storyGraph.endpoints.size() == 1);
			REQUIRE(storyGraph.endpoints.contains(static_cast<margen::DagNodeId>(nodeCount)));
		}
	}
}

TEST_CASE("generateStoryGraph keeps endpoints empty for nodeCount 0", "[margen][story]") {
	mythic::Dice dice(1);
	const auto storyGraph = margen::generateStoryGraph(dice, 0);
	REQUIRE(storyGraph.endpoints.empty());
	REQUIRE(storyGraph.graph.nodes.empty());
}
