#pragma once

#include <cstdint>
#include <functional>
#include <unordered_map>

#include <margen/Core/Directions.h>
#include <margen/Core/Pathing.h>
#include <margen/Core/SectorId.h>
#include <mythic/Math/Vector3.h>

namespace margen {

using CellAttributes = std::int64_t;

namespace CellAttribute {
	constexpr CellAttributes none = 0;
	constexpr CellAttributes solid = 1 << 0;
	constexpr CellAttributes path = 1 << 1;
	constexpr CellAttributes slots = 1 << 2;
	constexpr CellAttributes headroom = 1 << 3;
	constexpr CellAttributes flat = 1 << 4;
	constexpr CellAttributes narrowWalls = 1 << 5;
	constexpr CellAttributes customMesh = 1 << 6;
	constexpr CellAttributes nonEssentialPath = 1 << 7;
	constexpr CellAttributes customWallShifting = 1 << 8;
	constexpr CellAttributes independentCellMesh = 1 << 9;
} // namespace CellAttribute

constexpr int cellMaterialCount = 8;

struct WorldCell {
	std::uint32_t step = 0;
	CellAttributes attributes = 0;
	CellPaths paths = 0;
	std::uint32_t biomeId = 0;
	SectorId sector = 0;

	bool hasAttribute(CellAttributes attribute) const {
		return (attributes & attribute) != 0;
	}

	void addAttributes(CellAttributes newAttributes) {
		attributes |= newAttributes;
	}

	void removeAttributes(CellAttributes newAttributes) {
		attributes &= ~newAttributes;
	}

	bool isPath() const {
		return hasAttribute(CellAttribute::path);
	}

	bool isSolid() const {
		return hasAttribute(CellAttribute::solid);
	}

	bool hasPath(int direction) const {
		return (paths & (1u << direction)) != 0;
	}

	void addPaths(CellPaths newPaths) {
		paths |= newPaths;
	}

	void addPath(DirectionIndex direction) {
		addPaths(1u << direction);
	}

	void removePath(DirectionIndex direction) {
		paths &= ~(1u << direction);
	}
};

using CellMap = std::function<const WorldCell*(const mythic::Vector3i& location)>;

class CellGrid : public std::unordered_map<mythic::Vector3i, WorldCell> {
public:
	const WorldCell* operator()(const mythic::Vector3i& location) const {
		const auto it = find(location);
		return it == end() ? nullptr : &it->second;
	}

	WorldCell* operator()(const mythic::Vector3i& location) {
		const auto it = find(location);
		return it == end() ? nullptr : &it->second;
	}
};

} // namespace margen
