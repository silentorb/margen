#pragma once

#include <array>
#include <cstdint>

#include <mythic/Math/Vector3.h>
#include <mythic/Mythic/Containers.h>

namespace margen {

constexpr std::int8_t allDirectionsCount = 20;
constexpr std::int8_t flatDirectionCount = 6;
constexpr std::int8_t horizontalDirectionCount = 18;
using DirectionIndex = std::int8_t;

namespace Directions {
	constexpr DirectionIndex north = 0;
	constexpr DirectionIndex northeast = 1;
	constexpr DirectionIndex southeast = 2;
	constexpr DirectionIndex south = 3;
	constexpr DirectionIndex southwest = 4;
	constexpr DirectionIndex northwest = 5;

	constexpr DirectionIndex up_north = 6;
	constexpr DirectionIndex up_northeast = 7;
	constexpr DirectionIndex up_southeast = 8;
	constexpr DirectionIndex up_south = 9;
	constexpr DirectionIndex up_southwest = 10;
	constexpr DirectionIndex up_northwest = 11;

	constexpr DirectionIndex down_north = 12;
	constexpr DirectionIndex down_northeast = 13;
	constexpr DirectionIndex down_southeast = 14;
	constexpr DirectionIndex down_south = 15;
	constexpr DirectionIndex down_southwest = 16;
	constexpr DirectionIndex down_northwest = 17;

	constexpr DirectionIndex up = 18;
	constexpr DirectionIndex down = 19;
} // namespace Directions

const std::array<mythic::Vector3i, allDirectionsCount>& getDirectionVectors();
DirectionIndex getDirectionIndexFromVector(const mythic::Vector3i& direction);
const mythic::Vector3i& getGridDirectionVector(DirectionIndex direction);
DirectionIndex rotateDirectionIndex(DirectionIndex direction, int turns);
DirectionIndex rotateDirectionIndexPitch(DirectionIndex direction, int height);
DirectionIndex rotateDirectionIndex(DirectionIndex direction, int yaw, int pitch);
DirectionIndex getOppositeDirectionIndex(DirectionIndex direction);

inline bool isUpwardDirection(DirectionIndex index) {
	return index == Directions::up || (index >= Directions::up_north && index <= Directions::up_northwest);
}

inline bool isDownwardDirection(DirectionIndex index) {
	return index == Directions::down || (index >= Directions::down_north && index <= Directions::down_northwest);
}

inline bool isHorizontalDirection(DirectionIndex index) {
	return index != Directions::up && index != Directions::down;
}

inline bool isVerticalDirection(DirectionIndex index) {
	return index == Directions::up || index == Directions::down;
}

inline DirectionIndex removeDirections(DirectionIndex direction, DirectionIndex minus) {
	return direction & ~static_cast<DirectionIndex>(1 << minus);
}

mythic::FixedArray<mythic::Vector3i, 6>& getSurroundingOffsets();
mythic::FixedArray<mythic::Vector3i, 7>& getSelfAndSurroundingOffsets();

} // namespace margen
