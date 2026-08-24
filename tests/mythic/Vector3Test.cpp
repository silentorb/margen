#include <catch2/catch_test_macros.hpp>

#include <mythic/Math/Vector3.h>

TEST_CASE("Vector3i arithmetic and hash", "[mythic][vector3]") {
	const mythic::Vector3i a{1, 2, 3};
	const mythic::Vector3i b{4, -1, 0};

	REQUIRE(a + b == mythic::Vector3i{5, 1, 3});
	REQUIRE(b - a == mythic::Vector3i{3, -3, -3});
	REQUIRE(a * 2 == mythic::Vector3i{2, 4, 6});
	REQUIRE(3 * a == mythic::Vector3i{3, 6, 9});
	REQUIRE(mythic::Vector3i::ZeroValue() == mythic::Vector3i{0, 0, 0});
	REQUIRE(mythic::Vector3i::NoneValue() != mythic::Vector3i::ZeroValue());

	std::hash<mythic::Vector3i> hasher;
	REQUIRE(hasher(a) == hasher(mythic::Vector3i{1, 2, 3}));
	REQUIRE(hasher(a) != hasher(b));
}
