#pragma once

#include <cstdint>
#include <unordered_map>

#include <margen/Core/SectorId.h>
#include <mythic/Math/Vector3.h>
#include <mythic/Mythic/Containers.h>

namespace margen {

using SectorAttributes = std::int32_t;

namespace SectorAttribute {
	constexpr SectorAttributes none = 0;
	constexpr SectorAttributes supportsUpOffset = 1 << 0;
} // namespace SectorAttribute

struct Sector {
	SectorId id = 0;
	mythic::InlineVector<mythic::Vector3i, 32> cells;
	mythic::InlineVector<SectorId, 16> neighbors;
	SectorAttributes attributes = 0;

	bool hasAttribute(SectorAttributes attribute) const {
		return (attributes & attribute) != 0;
	}

	void addAttributes(SectorAttributes newAttributes) {
		attributes |= newAttributes;
	}

	void removeAttributes(SectorAttributes newAttributes) {
		attributes &= ~newAttributes;
	}
};

using SectorMap = std::unordered_map<SectorId, Sector>;
using CellSectorMap = std::unordered_map<mythic::Vector3i, SectorId>;

struct SectorData {
	SectorMap sectors;
	CellSectorMap cellMap;
	SectorId origin = 0;
};

} // namespace margen
