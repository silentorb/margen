#include <catch2/catch_test_macros.hpp>

#include <margen/Dependency/DependencyGraph.h>
#include <margen/Dependency/Querying.h>

TEST_CASE("getNodeDescendants follows opposite endpoints along a chain", "[margen][dependency]") {
	margen::DagConnectionMap connections;
	connections.emplace(margen::DagConnectionId{1, 2}, margen::DagConnection{});
	connections.emplace(margen::DagConnectionId{2, 3}, margen::DagConnection{});

	const auto descendants = margen::getNodeDescendants(connections, 1);

	REQUIRE(descendants.size() == 2);
	REQUIRE(descendants.contains({1, 2}));
	REQUIRE(descendants.contains({2, 3}));
}
