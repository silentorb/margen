#pragma once

#include <margen/Core/WorldCell.h>

namespace margen {

void connectCellsWithPath(WorldCell& cell, WorldCell& other, int direction);
void connectCellsWithPath(CellGrid& grid, const mythic::Vector3i& first, const mythic::Vector3i& second);
void connectCellsWithPaths(CellGrid& grid, std::initializer_list<mythic::Vector3i> locations);

} // namespace margen
