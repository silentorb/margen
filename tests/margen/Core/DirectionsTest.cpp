#include <catch2/catch_test_macros.hpp>

#include <margen/Core/Directions.h>

TEST_CASE("Directions round-trip known vectors", "[margen][core][directions]") {
	REQUIRE(margen::getDirectionIndexFromVector({0, -1, 0}) == margen::Directions::north);
	REQUIRE(margen::getDirectionIndexFromVector({1, 0, 0}) == margen::Directions::southeast);
	REQUIRE(margen::getDirectionIndexFromVector({0, 0, 1}) == margen::Directions::up);

	const auto& north = margen::getGridDirectionVector(margen::Directions::north);
	REQUIRE(north == mythic::Vector3i{0, -1, 0});
	REQUIRE(margen::getOppositeDirectionIndex(margen::Directions::north) == margen::Directions::south);
	REQUIRE(margen::getSurroundingOffsets().size() == 6);
}
