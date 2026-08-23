#include <catch2/catch_test_macros.hpp>

#include <mythic/Mythic/Containers.h>
#include <mythic/Mythic/Dice.h>

TEST_CASE("Dice is deterministic for the same seed", "[mythic][dice]") {
	mythic::Dice a(42);
	mythic::Dice b(42);

	REQUIRE(a.getInitialSeed() == 42);
	REQUIRE(a.getInt(100) == b.getInt(100));
	REQUIRE(a.getFloat(1.0f) == b.getFloat(1.0f));
	REQUIRE(a.getBoolean() == b.getBoolean());
}

TEST_CASE("removeFromArray clears a size-1 container", "[mythic][dice]") {
	mythic::InlineVector<int, 8> values{7};
	mythic::Dice dice(1);

	const auto removed = dice.removeFromArray(values);
	REQUIRE(removed == 7);
	REQUIRE(values.empty());
}

TEST_CASE("removeFromArray removes a chosen element from larger containers", "[mythic][dice]") {
	mythic::InlineVector<int, 8> values{1, 2, 3};
	mythic::Dice dice(5);

	const auto before = values.size();
	dice.removeFromArray(values);
	REQUIRE(values.size() == before - 1);
}
