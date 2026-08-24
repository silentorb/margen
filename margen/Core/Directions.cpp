#include <margen/Core/Directions.h>

#include <array>
#include <cassert>
#include <cstdint>

namespace margen {

namespace {

std::array<mythic::Vector3i, allDirectionsCount> gridDirectionVectors = {
	mythic::Vector3i{0, -1, 0},
	mythic::Vector3i{1, -1, 0},
	mythic::Vector3i{1, 0, 0},
	mythic::Vector3i{0, 1, 0},
	mythic::Vector3i{-1, 1, 0},
	mythic::Vector3i{-1, 0, 0},

	mythic::Vector3i{0, -1, 1},
	mythic::Vector3i{1, -1, 1},
	mythic::Vector3i{1, 0, 1},
	mythic::Vector3i{0, 1, 1},
	mythic::Vector3i{-1, 1, 1},
	mythic::Vector3i{-1, 0, 1},

	mythic::Vector3i{0, -1, -1},
	mythic::Vector3i{1, -1, -1},
	mythic::Vector3i{1, 0, -1},
	mythic::Vector3i{0, 1, -1},
	mythic::Vector3i{-1, 1, -1},
	mythic::Vector3i{-1, 0, -1},

	mythic::Vector3i{0, 0, 1},
	mythic::Vector3i{0, 0, -1},
};

} // namespace

const std::array<mythic::Vector3i, allDirectionsCount>& getDirectionVectors() {
	return gridDirectionVectors;
}

DirectionIndex getDirectionIndexFromVector(const mythic::Vector3i& direction) {
	int i = 0;
	for (const auto& d : gridDirectionVectors) {
		if (d == direction)
			return i;
		++i;
	}
	return -1;
}

const mythic::Vector3i& getGridDirectionVector(const DirectionIndex direction) {
	return gridDirectionVectors[static_cast<std::uint8_t>(direction)];
}

mythic::Vector3i getDirectionVectorWithHeight(const DirectionIndex direction, const int height) {
	return getGridDirectionVector(direction) + mythic::Vector3i{0, 0, height};
}

DirectionIndex rotateDirectionIndex(const DirectionIndex direction, const int turns) {
	if (direction >= Directions::up)
		return direction;

	const auto layer = direction / flatDirectionCount;
	return (direction + turns + flatDirectionCount) % flatDirectionCount + layer * flatDirectionCount;
}

DirectionIndex rotateDirectionIndexPitch(const DirectionIndex direction, const int height) {
	if (height == 0)
		return direction;

	if (direction < flatDirectionCount) {
		if (height > 0)
			return direction + flatDirectionCount;
		return direction + flatDirectionCount * 2;
	}

	if (direction < flatDirectionCount * 2) {
		if (height == -1)
			return direction - flatDirectionCount;
		if (height < -1)
			return direction + flatDirectionCount;
		return direction;
	}

	if (direction < flatDirectionCount * 3) {
		if (height == 1)
			return direction - flatDirectionCount * 2;
		if (height > 1)
			return direction - flatDirectionCount;
		return direction;
	}

	return direction;
}

DirectionIndex rotateDirectionIndex(const DirectionIndex direction, const int yaw, const int pitch) {
	return rotateDirectionIndexPitch(rotateDirectionIndex(direction, yaw), pitch);
}

DirectionIndex getOppositeDirectionIndex(const DirectionIndex direction) {
	switch (direction) {
		case Directions::southeast: return Directions::northwest;
		case Directions::south: return Directions::north;
		case Directions::southwest: return Directions::northeast;
		case Directions::northwest: return Directions::southeast;
		case Directions::north: return Directions::south;
		case Directions::northeast: return Directions::southwest;

		case Directions::up_southeast: return Directions::down_northwest;
		case Directions::up_south: return Directions::down_north;
		case Directions::up_southwest: return Directions::down_northeast;
		case Directions::up_northwest: return Directions::down_southeast;
		case Directions::up_north: return Directions::down_south;
		case Directions::up_northeast: return Directions::down_southwest;

		case Directions::down_southeast: return Directions::up_northwest;
		case Directions::down_south: return Directions::up_north;
		case Directions::down_southwest: return Directions::up_northeast;
		case Directions::down_northwest: return Directions::up_southeast;
		case Directions::down_north: return Directions::up_south;
		case Directions::down_northeast: return Directions::up_southwest;

		case Directions::up: return Directions::down;
		case Directions::down: return Directions::up;
		default:
			assert(false && "Can't get opposite of invalid direction");
			return direction;
	}
}

mythic::FixedArray<mythic::Vector3i, 6>& getSurroundingOffsets() {
static mythic::FixedArray<mythic::Vector3i, 6> data = {
	mythic::Vector3i{0, -1, 0},
	mythic::Vector3i{1, -1, 0},
	mythic::Vector3i{1, 0, 0},
	mythic::Vector3i{0, 1, 0},
	mythic::Vector3i{-1, 1, 0},
	mythic::Vector3i{-1, 0, 0},
};

	return data;
}

mythic::FixedArray<mythic::Vector3i, 7>& getSelfAndSurroundingOffsets() {
	static mythic::FixedArray<mythic::Vector3i, 7> data = {
		mythic::Vector3i{0, 0, 0},
		mythic::Vector3i{0, -1, 0},
		mythic::Vector3i{1, -1, 0},
		mythic::Vector3i{1, 0, 0},
		mythic::Vector3i{0, 1, 0},
		mythic::Vector3i{-1, 1, 0},
		mythic::Vector3i{-1, 0, 0},
	};

	return data;
}

} // namespace margen
