//! General-purpose utilities — not world generation.

mod dice;
mod random;
mod vector2;
mod vector3;

pub use dice::{get_static_dice, Dice};
pub use random::psuedorandom;
pub use vector2::Vector2i;
pub use vector3::Vector3i;
