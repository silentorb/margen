#pragma once

#include <cstdint>
#include <unordered_map>

#include <margen/Graphing/Dependency/DagNodeId.h>
#include <mythic/Math/Vector3.h>
#include <mythic/Mythic/Containers.h>

namespace margen {

using SectorId = std::int16_t;

using SimpleDagConnections = mythic::InlineVector<DagNodeId, 16>;

struct SimpleDagNode {
	std::int32_t depth = 0;
	SimpleDagConnections connections[2];
	std::int32_t type = 0;

	static constexpr int inputIndex = 0;
	static constexpr int outputIndex = 1;

	SimpleDagConnections& getInputs() {
		return connections[inputIndex];
	}

	SimpleDagConnections& getOutputs() {
		return connections[outputIndex];
	}

	const SimpleDagConnections& getInputs() const {
		return connections[inputIndex];
	}

	const SimpleDagConnections& getOutputs() const {
		return connections[outputIndex];
	}
};

using DagNodeMap = std::unordered_map<DagNodeId, SimpleDagNode>;

struct SimpleDagGraph {
	DagNodeMap nodes;
	DagNodeId root = 0;
};

struct LevelSwitchEntry {
	SectorId sourceSector = 0;
	SectorId targetSector = 0;
	mythic::Vector3i sourceCellLocation = mythic::Vector3i::NoneValue();
};

} // namespace margen
