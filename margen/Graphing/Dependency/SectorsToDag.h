#pragma once

#include <margen/Core/Sector.h>
#include <margen/Graphing/Dependency/SimpleDependencyGraph.h>

namespace margen {

/*
 * Converts a sector graph to a dependency graph (Directed Acyclic Graph).
 */
void sectorsToDag(const SectorData& sectorData, SimpleDagGraph& graph);

} // namespace margen
