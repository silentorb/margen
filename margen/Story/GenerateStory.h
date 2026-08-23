#pragma once

#include <margen/Graphing/Dependency/DagNodeId.h>

namespace mythic {
class Dice;
}

namespace margen {

struct StoryGraph;

StoryGraph generateStoryGraph(mythic::Dice& dice, DagNodeId nodeCount);

} // namespace margen
