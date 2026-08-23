#include <catch2/catch_test_macros.hpp>

#include <margen/Story/GenerateStory.h>
#include <margen/Story/LocationBranching.h>
#include <margen/Story/StoryGraph.h>
#include <mythic/Mythic/Dice.h>

TEST_CASE("storyToLocationBranching matches Unreal assertions", "[margen][story]") {
	for (int nodeCount = 3; nodeCount < 10; nodeCount++) {
		for (int seed = 1; seed < 3; ++seed) {
			mythic::Dice dice(seed);
			const auto storyGraph = margen::generateStoryGraph(dice, static_cast<margen::DagNodeId>(nodeCount));
			const auto branchingConnections = margen::storyToLocationBranching(dice, storyGraph.graph);

			REQUIRE(static_cast<int>(branchingConnections.size()) == nodeCount - 1);
			REQUIRE_FALSE(branchingConnections.contains({2, 3}));
		}
	}
}
