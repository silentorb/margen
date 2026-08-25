//! C ABI for margen world generation.

use margen::{generate_story_grid, DagNodeId, StoryGridResult};
use mythic::{Dice, Vector3i};
use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::os::raw::c_int;
use std::ptr;

thread_local! {
	static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(message: &str) {
	LAST_ERROR.with(|slot| {
		*slot.borrow_mut() = Some(CString::new(message).unwrap_or_else(|_| {
			CString::new("invalid error message").expect("static")
		}));
	});
}

fn clear_last_error() {
	LAST_ERROR.with(|slot| {
		*slot.borrow_mut() = None;
	});
}

/// Opaque story-grid handle.
pub struct MargenStoryGrid {
	result: StoryGridResult,
	cells: Vec<(Vector3i, margen::WorldCell)>,
	portals: Vec<(DagNodeId, DagNodeId)>,
}

impl MargenStoryGrid {
	fn from_result(result: StoryGridResult) -> Self {
		let mut cells: Vec<_> = result
			.grid
			.iter()
			.map(|(loc, cell)| (*loc, cell.clone()))
			.collect();
		cells.sort_by_key(|(loc, _)| (loc.x, loc.y, loc.z));
		let portals: Vec<_> = result.portal_connections.iter().copied().collect();
		Self {
			result,
			cells,
			portals,
		}
	}
}

#[repr(C)]
pub struct MargenCell {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub attributes: i64,
	pub paths: u32,
	pub biome_id: u32,
	pub sector: i16,
	pub _pad: i16,
}

#[repr(C)]
pub struct MargenPortal {
	pub from: i16,
	pub to: i16,
}

pub type MargenStatus = c_int;

pub const MARGEN_OK: MargenStatus = 0;
pub const MARGEN_ERR_INVALID_ARGUMENT: MargenStatus = 1;
pub const MARGEN_ERR_GENERATION_FAILED: MargenStatus = 2;

#[no_mangle]
pub extern "C" fn margen_version() -> *const c_char {
	static VERSION: &[u8] = b"0.1.0\0";
	VERSION.as_ptr().cast()
}

#[no_mangle]
pub extern "C" fn margen_last_error() -> *const c_char {
	LAST_ERROR.with(|slot| match slot.borrow().as_ref() {
		Some(msg) => msg.as_ptr(),
		None => ptr::null(),
	})
}

/// Generate a story grid. On success writes a non-null handle and returns `MARGEN_OK`.
#[no_mangle]
pub unsafe extern "C" fn margen_generate_story_grid(
	seed: i32,
	node_count: i16,
	origin_x: i32,
	origin_y: i32,
	origin_z: i32,
	out_handle: *mut *mut MargenStoryGrid,
) -> MargenStatus {
	clear_last_error();
	if out_handle.is_null() {
		set_last_error("out_handle is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	if node_count < 0 {
		set_last_error("node_count must be non-negative");
		*out_handle = ptr::null_mut();
		return MARGEN_ERR_INVALID_ARGUMENT;
	}

	let mut dice = Dice::new(seed);
	let origin = Vector3i::new(origin_x, origin_y, origin_z);
	let result = generate_story_grid(&mut dice, node_count, origin);
	if !result.success {
		set_last_error("story grid generation failed");
		*out_handle = ptr::null_mut();
		return MARGEN_ERR_GENERATION_FAILED;
	}

	let handle = Box::new(MargenStoryGrid::from_result(result));
	*out_handle = Box::into_raw(handle);
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_free(handle: *mut MargenStoryGrid) {
	if !handle.is_null() {
		drop(Box::from_raw(handle));
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_success(handle: *const MargenStoryGrid) -> c_int {
	if handle.is_null() {
		return 0;
	}
	if (*handle).result.success {
		1
	} else {
		0
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_cell_count(handle: *const MargenStoryGrid) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).cells.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_get_cell(
	handle: *const MargenStoryGrid,
	index: usize,
	out_cell: *mut MargenCell,
) -> MargenStatus {
	if handle.is_null() || out_cell.is_null() {
		set_last_error("null handle or out_cell");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let grid = &*handle;
	let Some((loc, cell)) = grid.cells.get(index) else {
		set_last_error("cell index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_cell = MargenCell {
		x: loc.x,
		y: loc.y,
		z: loc.z,
		attributes: cell.attributes,
		paths: cell.paths,
		biome_id: cell.biome_id,
		sector: cell.sector,
		_pad: 0,
	};
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_portal_count(handle: *const MargenStoryGrid) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).portals.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_get_portal(
	handle: *const MargenStoryGrid,
	index: usize,
	out_portal: *mut MargenPortal,
) -> MargenStatus {
	if handle.is_null() || out_portal.is_null() {
		set_last_error("null handle or out_portal");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let grid = &*handle;
	let Some(&(from, to)) = grid.portals.get(index) else {
		set_last_error("portal index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_portal = MargenPortal { from, to };
	MARGEN_OK
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ffi_story_grid_smoke() {
		unsafe {
			let mut handle: *mut MargenStoryGrid = ptr::null_mut();
			let status = margen_generate_story_grid(1, 5, 0, 0, 0, &mut handle);
			assert_eq!(status, MARGEN_OK);
			assert!(!handle.is_null());
			assert_eq!(margen_story_grid_success(handle), 1);
			assert_eq!(margen_story_grid_cell_count(handle), 5);
			assert_eq!(margen_story_grid_portal_count(handle), 0);

			let mut cell = MargenCell {
				x: 0,
				y: 0,
				z: 0,
				attributes: 0,
				paths: 0,
				biome_id: 0,
				sector: 0,
				_pad: 0,
			};
			assert_eq!(margen_story_grid_get_cell(handle, 0, &mut cell), MARGEN_OK);

			margen_story_grid_free(handle);
		}
	}

	#[test]
	fn version_is_nul_terminated() {
		let ptr = margen_version();
		assert!(!ptr.is_null());
	}
}
