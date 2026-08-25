//! Spatial distribution helpers (lazy points, etc.).

pub mod lazy_point;

pub use lazy_point::{get_or_create_lazy_point, sample_lazy_points};
