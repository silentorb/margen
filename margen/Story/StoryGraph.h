#pragma once

#include <unordered_set>

#include <margen/Dependency/DependencyGraph.h>
#include <margen/Graphing/Dependency/DagNodeId.h>

namespace margen {

struct StoryGraph {
	DagGraph graph;
	std::unordered_set<DagNodeId> endpoints;
};

} // namespace margen
