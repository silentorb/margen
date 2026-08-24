#pragma once

#include <cstddef>
#include <functional>
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

constexpr Vector3i operator+(const Vector3i& a, const Vector3i& b) {
	return {a.x + b.x, a.y + b.y, a.z + b.z};
}

constexpr Vector3i operator-(const Vector3i& a, const Vector3i& b) {
	return {a.x - b.x, a.y - b.y, a.z - b.z};
}

constexpr Vector3i operator*(const Vector3i& v, const int scalar) {
	return {v.x * scalar, v.y * scalar, v.z * scalar};
}

constexpr Vector3i operator*(const int scalar, const Vector3i& v) {
	return v * scalar;
}

constexpr bool operator==(const Vector3i& a, const Vector3i& b) {
	return a.x == b.x && a.y == b.y && a.z == b.z;
}

constexpr bool operator!=(const Vector3i& a, const Vector3i& b) {
	return !(a == b);
}

} // namespace mythic

template <>
struct std::hash<mythic::Vector3i> {
	std::size_t operator()(const mythic::Vector3i& v) const noexcept {
		const auto x = static_cast<std::size_t>(static_cast<std::uint32_t>(v.x));
		const auto y = static_cast<std::size_t>(static_cast<std::uint32_t>(v.y));
		const auto z = static_cast<std::size_t>(static_cast<std::uint32_t>(v.z));
		return x ^ (y << 10) ^ (z << 20);
	}
};
