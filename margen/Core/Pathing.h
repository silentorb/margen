#pragma once

#include <cstdint>
#include <unordered_map>

#include <margen/Core/Directions.h>
#include <mythic/Math/Vector3.h>
#include <mythic/Mythic/Containers.h>

namespace margen {

using CellPaths = std::uint32_t;
using CellPathsArray = mythic::FixedArray<DirectionIndex, 6>;
using NeighborsArray = mythic::InlineVector<mythic::Vector3i, 6>;
using CellPathsMap = std::unordered_map<mythic::Vector3i, CellPathsArray>;
using NeighborsMap = std::unordered_map<mythic::Vector3i, NeighborsArray>;
using PathDepths = std::unordered_map<mythic::Vector3i, std::int32_t>;

} // namespace margen
