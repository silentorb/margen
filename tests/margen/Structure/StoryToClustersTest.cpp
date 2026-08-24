#include <catch2/catch_test_macros.hpp>

#include <margen/Structure/Story/GenerateStoryGrid.h>
#include <mythic/Mythic/Dice.h>

TEST_CASE("storyToClusters matches Unreal assertions", "[margen][structure][storyToClusters]") {
	for (int nodeCount = 3; nodeCount <= 8; ++nodeCount) {
		for (int seed = 1; seed <= 3; ++seed) {
			mythic::Dice dice(seed);
			const auto result = margen::generateStoryGrid(dice, static_cast<margen::DagNodeId>(nodeCount));

			REQUIRE(result.success);
			REQUIRE(result.grid.size() == static_cast<std::size_t>(nodeCount));
			REQUIRE(result.portalConnections.empty());
		}
	}
}
