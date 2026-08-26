//! Sector partition and goal analysis.

mod cell_grouping;
mod goals;
mod headroom_clearance;
mod map_path_depths;
mod partition_sectors;
mod sector_propagation;
mod types;
mod validation;

pub use goals::{generate_goals, select_switch_cell_locations};
pub use partition_sectors::partition_sectors;
pub use types::{AnalysisInput, AnalysisResult, PartitionConfig, PathDepths};
pub use validation::validate_sector_map;
