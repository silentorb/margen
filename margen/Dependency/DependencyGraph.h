#pragma once

#include <cstdint>
#include <functional>
#include <unordered_map>
#include <unordered_set>
#include <utility>

#include <margen/Graphing/Dependency/DagNodeId.h>

namespace margen {

using DagConnectionAttributes = std::uint32_t;

struct DagConnection {
	DagConnectionAttributes attributes = 0;

	DagConnection() = default;

	explicit DagConnection(const DagConnectionAttributes attributes)
		: attributes(attributes) {
	}

	bool hasAttribute(const DagConnectionAttributes attribute) const {
		return (attributes & attribute) != 0;
	}
};

using DagConnectionId = std::pair<DagNodeId, DagNodeId>;

struct DagConnectionIdHash {
	std::size_t operator()(const DagConnectionId& id) const noexcept {
		const auto first = static_cast<std::uint16_t>(id.first);
		const auto second = static_cast<std::uint16_t>(id.second);
		return (static_cast<std::size_t>(first) << 16) ^ static_cast<std::size_t>(second);
	}
};

using DagConnectionMap = std::unordered_map<DagConnectionId, DagConnection, DagConnectionIdHash>;

struct DagGraph {
	std::unordered_set<DagNodeId> nodes;
	DagConnectionMap connections;
	DagNodeId root = 0;
};

} // namespace margen
