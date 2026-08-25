use crate::dependency::DagConnectionAttributes;

pub mod story_connection_attribute {
	use super::DagConnectionAttributes;

	pub const NONE: DagConnectionAttributes = 0;
	pub const DIRECT: DagConnectionAttributes = 1 << 0;
	pub const UNLOCKS: DagConnectionAttributes = 1 << 1;
}
