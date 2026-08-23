#include <catch2/catch_test_macros.hpp>

#include <mythic/Mythic/Containers.h>

TEST_CASE("InlineVector basics", "[mythic][containers]") {
	mythic::InlineVector<int, 4> values;
	REQUIRE(values.empty());

	values.push_back(1);
	values.push_back(2);
	REQUIRE(values.size() == 2);
	REQUIRE(values.contains(2));
	REQUIRE_FALSE(values.contains(3));
	REQUIRE(values.insert_unique(2) == false);
	REQUIRE(values.insert_unique(3) == true);
	REQUIRE(values.size() == 3);
}

TEST_CASE("FixedArray basics", "[mythic][containers]") {
	mythic::FixedArray<int, 4> values{10, 20};
	REQUIRE(values.size() == 2);
	REQUIRE(values.contains(10));
	values.push_back(30);
	REQUIRE(values.size() == 3);
}
