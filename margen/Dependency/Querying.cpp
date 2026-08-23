#include <margen/Dependency/Querying.h>

#include <queue>

namespace margen {

template <std::uint32_t TIndex>
DagConnections getImmediateNodeConnections(const DagConnectionMap& connections, const DagNodeId endpoint) {
	DagConnections result;

	for (const auto& connection : connections) {
		const auto& key = connection.first;
		const auto matched = TIndex == 0 ? key.first : key.second;
		if (matched == endpoint) {
			result.push_back({key, connection.second});
		}
	}

	return result;
}

template <std::uint32_t TIndex>
int getImmediateNodeConnectionsCount(const DagConnectionMap& connections, const DagNodeId endpoint) {
	int result = 0;

	for (const auto& [key, _] : connections) {
		const auto matched = TIndex == 0 ? key.first : key.second;
		if (matched == endpoint) {
			++result;
		}
	}

	return result;
}

template <std::uint32_t TIndex>
DagConnectionMap getLayeredNodeConnections(const DagConnectionMap& connections, const DagNodeId endpoint) {
	DagConnectionMap result;

	std::queue<DagNodeId> queue;
	queue.push(endpoint);

	while (!queue.empty()) {
		const auto node = queue.front();
		queue.pop();
		const auto parents = getImmediateNodeConnections<TIndex>(connections, node);

		if (parents.size() == 0) {
			break;
		} else {
			for (const auto& parent : parents) {
				result.emplace(parent.first, parent.second);
			}

			for (const auto& parent : parents) {
				// incorrect-traversal-endpoint: enqueue the opposite end of the matched index
				const auto& key = parent.first;
				const DagNodeId next = TIndex == 0 ? key.second : key.first;
				queue.push(next);
			}
		}
	}

	return result;
}

DagConnections getImmediateNodeAncestors(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getImmediateNodeConnections<1>(connections, endpoint);
}

int getImmediateNodeAncestorsCount(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getImmediateNodeConnectionsCount<1>(connections, endpoint);
}

DagConnectionMap getNodeAncestors(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getLayeredNodeConnections<1>(connections, endpoint);
}

DagConnections getImmediateNodeDescendants(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getImmediateNodeConnections<0>(connections, endpoint);
}

int getImmediateNodeDescendantsCount(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getImmediateNodeConnectionsCount<0>(connections, endpoint);
}

DagConnectionMap getNodeDescendants(const DagConnectionMap& connections, const DagNodeId endpoint) {
	return getLayeredNodeConnections<0>(connections, endpoint);
}

} // namespace margen
