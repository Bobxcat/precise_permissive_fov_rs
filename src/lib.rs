mod python_impl_translated;
pub use python_impl_translated::field_of_view;

use std::collections::HashSet;

use glam::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PPFOVTile {
    Empty,
    Obstacle,
}

/// * `start` - The coordinate on the grid that's the center of view
/// * `map_width`, `map_height` - The maximum extents of the grid. The minimum extents are assumed to be both zero
/// * `radius` - How far the field of view may extend in either direction along the x and y axis
/// * `get_tile` - Get the state of a given tile
pub fn build_fov_set(
    start: IVec2,
    map_width: i32,
    map_height: i32,
    radius: i32,
    get_tile: impl Fn(IVec2) -> PPFOVTile,
) -> HashSet<IVec2> {
    let mut visible = HashSet::new();
    field_of_view(
        start.x,
        start.y,
        map_width,
        map_height,
        radius,
        |v| {
            visible.insert(v);
        },
        |v| match get_tile(v) {
            PPFOVTile::Obstacle => true,
            PPFOVTile::Empty => false,
        },
    );
    visible
}
