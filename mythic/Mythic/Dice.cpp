#include <mythic/Mythic/Dice.h>

namespace mythic {

Dice staticDice;

Dice& getStaticDice() {
	return staticDice;
}

} // namespace mythic
