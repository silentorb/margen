#include <catch2/catch_test_macros.hpp>

#include <margen/Core/Sector.h>
#include <margen/Graphing/Dependency/SectorsToDag.h>

TEST_CASE("sectorsToDag builds layered connections", "[margen][graphing][sectorsToDag]") {
	margen::SectorData sectorData;
	sectorData.origin = 1;

	auto& sectorOne = sectorData.sectors[1];
	sectorOne.id = 1;
	sectorOne.neighbors.push_back(2);

	auto& sectorTwo = sectorData.sectors[2];
	sectorTwo.id = 2;
	sectorTwo.neighbors.push_back(1);
	sectorTwo.neighbors.push_back(3);

	auto& sectorThree = sectorData.sectors[3];
	sectorThree.id = 3;
	sectorThree.neighbors.push_back(2);

	margen::SimpleDagGraph graph;
	margen::sectorsToDag(sectorData, graph);

	REQUIRE(graph.nodes.size() == 3);
	REQUIRE(graph.nodes.at(1).depth == 0);
	REQUIRE(graph.nodes.at(2).depth == 1);
	REQUIRE(graph.nodes.at(3).depth == 2);
	REQUIRE(graph.nodes.at(1).getOutputs().size() == 1);
	REQUIRE(graph.nodes.at(1).getOutputs()[0] == 2);
	REQUIRE(graph.nodes.at(2).getInputs().size() == 1);
	REQUIRE(graph.nodes.at(2).getInputs()[0] == 1);
}
