#pragma once

#include <cstdint>

namespace margen {

// 16 bit so that an ID connection pair is 32 bit, for efficient hashing.
// 16 bits should support a sufficient volume of nodes.
using DagNodeId = std::int16_t;

} // namespace margen
