#pragma once

#include <unordered_map>

#include <margen/Core/SectorId.h>
#include <mythic/Math/Vector3.h>

namespace margen {

using SectorLocationMap = std::unordered_map<SectorId, mythic::Vector3i>;

} // namespace margen
