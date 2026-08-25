//! General-purpose utilities — not world generation.

mod dice;
mod distribution;
mod random;
mod vector2;
mod vector3;

pub use dice::{get_static_dice, Dice};
pub use distribution::{select_many_ordered, shuffle_vec};
pub use random::psuedorandom;
pub use vector2::Vector2i;
pub use vector3::Vector3i;
