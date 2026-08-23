#pragma once

#include <cstdint>

namespace margen {

namespace StoryConnectionAttribute {
	constexpr std::uint32_t none = 0; // Not a connection
	constexpr std::uint32_t direct = 1 << 0; // The nodes are neighbors and directly connected other than possible gates
	constexpr std::uint32_t unlocks = 1 << 1; // The first node contains the means for unlocking the path to the second node
}

} // namespace margen
