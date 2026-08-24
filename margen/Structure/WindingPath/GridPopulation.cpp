#include <margen/Structure/WindingPath/GridPopulation.h>

#include <margen/Core/Directions.h>

namespace margen {

void connectCellsWithPath(WorldCell& cell, WorldCell& other, const int direction) {
	const auto oppositeDirection = getOppositeDirectionIndex(static_cast<DirectionIndex>(direction));
	cell.addPath(static_cast<DirectionIndex>(direction));
	other.addPath(oppositeDirection);

	if (isUpwardDirection(static_cast<DirectionIndex>(direction))) {
		cell.addAttributes(CellAttribute::solid);
	} else if (isDownwardDirection(static_cast<DirectionIndex>(direction))) {
		other.addAttributes(CellAttribute::solid);
	}
}

void connectCellsWithPath(CellGrid& grid, const mythic::Vector3i& first, const mythic::Vector3i& second) {
	const auto a = grid.find(first);
	const auto b = grid.find(second);
	const auto direction = getDirectionIndexFromVector(second - first);
	if (a != grid.end() && b != grid.end() && direction != -1) {
		connectCellsWithPath(a->second, b->second, direction);
	}
}

void connectCellsWithPaths(CellGrid& grid, const std::initializer_list<mythic::Vector3i> locations) {
	if (locations.size() < 2)
		return;

	mythic::Vector3i previous;

	for (const auto location : locations) {
		if (location != *locations.begin()) {
			connectCellsWithPath(grid, previous, location);
		}
		previous = location;
	}
}

} // namespace margen
