#pragma once

#include <limits>

namespace mythic {

struct Vector3i {
	int x = 0;
	int y = 0;
	int z = 0;

	static constexpr Vector3i ZeroValue() {
		return {};
	}

	static constexpr Vector3i NoneValue() {
		constexpr int n = std::numeric_limits<int>::min();
		return {n, n, n};
	}
};

constexpr bool operator==(const Vector3i& a, const Vector3i& b) {
	return a.x == b.x && a.y == b.y && a.z == b.z;
}

constexpr bool operator!=(const Vector3i& a, const Vector3i& b) {
	return !(a == b);
}

} // namespace mythic
