/// Engine-agnostic placeable identity (Unreal `UPlaceable` id fields).
pub type PlaceableId = u32;

/// Player spawn location marker. Hosts map this id to a placeholder actor.
pub const PLACEABLE_PLAYER_SPAWN: PlaceableId = 1;
