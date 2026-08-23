#include <margen/Story/LocationBranching.h>

#include <margen/Dependency/Querying.h>
#include <margen/Story/Types.h>
#include <mythic/Mythic/Containers.h>
#include <mythic/Mythic/Dice.h>

namespace margen {

/*
 * This converts an abstract story DAG to a slightly less abstract branching graph
 * 
 * The goal is to ensure every switch/key requires *some* backtracking, i.e: No key is right next to its gate.
 *
 *	(C)
 *	 ↑  
 *	(B)					(C)
 *	 ↑		Becomes		 ↑
 *	(A)					(A) → (B)
 *
 * Where the needed player traversal between nodes would be:
 * 
 *	 A → B → A → C (ABAC)
 *	
 *	(D)
 *	 ↑ 
 *	(C)					(D)					(D)
 *	 ↑					 ↑					 ↑
 *	(B)					(A) → (C)			(B)
 *	 ↑		Becomes		 ↓			or		 ↑
 *	(A)					(B)					(A) → (C)
 *	
 *						ABACAD				ABACABD				
 *
 * Minimal cases resolve with no branching:
 * 
 *	(B)					(B)
 *	 ↑		Becomes		 ↑
 *	(A)					(A)
 */

DagConnectionMap storyToLocationBranching(mythic::Dice& dice, const DagGraph& graph, const int maxConnectionsPerNode) {
	DagConnectionMap newConnections;

	for (const auto& node : graph.nodes) {
		const auto ancestors = getNodeAncestors(graph.connections, node);
		mythic::InlineVector<std::pair<DagConnectionId, DagConnection>, 32> options;

		for (const auto& connection : ancestors) {
			if (connection.second.hasAttribute(StoryConnectionAttribute::unlocks) && connection.first.second == node)
				continue;

			const auto count = getImmediateNodeAncestorsCount(newConnections, connection.first.first);
			if (count < maxConnectionsPerNode) {
				options.push_back(connection);
			}
		}

		if (options.size() > 0) {
			const auto option = dice.fromArray(options);
			newConnections.emplace(DagConnectionId{option.first.first, node}, DagConnection{});
		}
	}

	return newConnections;
}

} // namespace margen
