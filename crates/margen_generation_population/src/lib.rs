//! Population math: placeables → abstract actor placements (no host scene work).
//!
//! First pass: player-spawn placement only. Slot gather, category distribution, and
//! host instantiation stay deferred / in the engine host.

mod placeable;
mod placement;
mod populate;

pub use placeable::{PlaceableId, PLACEABLE_PLAYER_SPAWN};
pub use placement::{ActorPlacement, ActorPlacements};
pub use populate::{append_player_start_placement, populate_player_start, populate_world, PopulateInput};
