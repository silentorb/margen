#pragma once

#include <utility>

#include <margen/Dependency/DependencyGraph.h>
#include <margen/Graphing/Dependency/DagNodeId.h>
#include <mythic/Mythic/Containers.h>

namespace margen {

using DagNodes = mythic::InlineVector<DagNodeId, 32>;
using DagConnections = mythic::InlineVector<std::pair<DagConnectionId, DagConnection>, 32>;
using NodeSet = std::unordered_set<DagNodeId>;

DagConnections getImmediateNodeAncestors(const DagConnectionMap& connections, DagNodeId endpoint);
int getImmediateNodeAncestorsCount(const DagConnectionMap& connections, DagNodeId endpoint);
DagConnectionMap getNodeAncestors(const DagConnectionMap& connections, DagNodeId endpoint);

DagConnections getImmediateNodeDescendants(const DagConnectionMap& connections, DagNodeId endpoint);
int getImmediateNodeDescendantsCount(const DagConnectionMap& connections, DagNodeId endpoint);
DagConnectionMap getNodeDescendants(const DagConnectionMap& connections, DagNodeId endpoint);

} // namespace margen
