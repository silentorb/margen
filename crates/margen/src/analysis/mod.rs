mod cell_grouping;
mod headroom_clearance;
mod map_path_depths;
mod partition_sectors;
mod sector_propagation;
mod types;
mod validation;

pub use partition_sectors::partition_sectors;
pub use types::{AnalysisInput, AnalysisResult, PartitionConfig, PathDepths};
pub use validation::validate_sector_map;
