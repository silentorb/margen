#pragma once

#include <cstdint>
#include <random>
#include <vector>

#include <mythic/Mythic/Containers.h>

namespace mythic {

class Dice {
public:
	std::int32_t initialSeed = 0;
	std::mt19937 stream;

	explicit Dice()
		: Dice(0) {
	}

	explicit Dice(const std::int32_t seed)
		: initialSeed(seed)
		, stream(static_cast<std::mt19937::result_type>(seed)) {
	}

	std::int32_t getInitialSeed() const {
		return initialSeed;
	}

	std::int32_t getInt(const std::int32_t max) {
		return getInt(0, max);
	}

	std::int32_t getInt(const std::int32_t min, const std::int32_t max) {
		std::uniform_int_distribution<std::int32_t> distribution(min, max);
		return distribution(stream);
	}

	float getFloat(const float max) {
		return getFloat(0.0f, max);
	}

	float getFloat(const float min, const float max) {
		std::uniform_real_distribution<float> distribution(min, max);
		return distribution(stream);
	}

	float getRangedFloat(const float max) {
		return getFloat(-max, max);
	}

	bool getBoolean() {
		return getInt(1) == 0;
	}

	template <typename T>
	T fromStandardVector(std::vector<T>& container) {
		if (container.size() == 1)
			return container[0];

		const auto index = static_cast<std::size_t>(getInt(static_cast<std::int32_t>(container.size() - 1)));
		return container[index];
	}

	template <typename TValue, std::size_t N>
	TValue removeFromArray(InlineVector<TValue, N>& container) {
		if (container.size() == 1) {
			const auto result = container[0];
			container.clear();
			return result;
		}

		const auto index = static_cast<std::size_t>(getInt(static_cast<std::int32_t>(container.size() - 1)));
		const auto result = container[index];
		container.erase(container.begin() + static_cast<std::ptrdiff_t>(index));
		return result;
	}

	template <typename TValue, std::size_t N>
	TValue fromArray(const InlineVector<TValue, N>& container) {
		if (container.size() == 1)
			return container[0];

		const auto index = static_cast<std::size_t>(getInt(static_cast<std::int32_t>(container.size() - 1)));
		return container[index];
	}
};

Dice& getStaticDice();

} // namespace mythic
