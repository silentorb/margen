//! Seeded RNG and pseudo-random helpers.

mod dice;
mod random;

pub use dice::{get_static_dice, Dice};
pub use random::psuedorandom;
