//! Grid refinement: organic corner writers (path slopes first; wall-shift later).

mod corner_column;
mod slopes;

pub use slopes::{apply_path_slopes, cell_has_upward_paths};
