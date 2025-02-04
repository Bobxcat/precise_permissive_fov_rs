//! From: https://www.roguebasin.com/index.php/Permissive_Field_of_View_in_Python
//! Original code available at `src/python_impl_original.py`
//!
//! This module is almost exactly a line-by-line translation of the python version

use std::{collections::HashSet, rc::Rc};

use glam::IVec2;

/// * `start_x`, `start_y` - The (x, y) coordinate on the grid that's the center of view
/// * `map_width`, `map_height` - The maximum extents of the grid. The minimum extents are assumed to be both zero
/// * `radius` - How far the field of view may extend in either direction along the x and y axis
/// * `func_visit_tile` - Callback that sets a tile as "visible"
/// * `func_tile_blocked` - Returns `true` if the given tile is an obstacle
pub fn field_of_view(
    start_x: i32,
    start_y: i32,
    map_width: i32,
    map_height: i32,
    radius: i32,
    mut func_visit_tile: impl FnMut(IVec2),
    func_tile_blocked: impl Fn(IVec2) -> bool,
) {
    let mut visited = HashSet::<IVec2>::new();

    func_visit_tile(IVec2 {
        x: start_x,
        y: start_y,
    });
    visited.insert(IVec2 {
        x: start_x,
        y: start_y,
    });

    let min_extent_x = if start_x < radius { start_x } else { radius };
    let max_extent_x = if map_width - start_x - 1 < radius {
        map_width - start_x - 1
    } else {
        radius
    };

    let min_extent_y = if start_y < radius { start_y } else { radius };
    let max_extent_y = if map_height - start_y - 1 < radius {
        map_height - start_y - 1
    } else {
        radius
    };

    check_quadrant(
        &mut visited,
        start_x,
        start_y,
        1,
        1,
        max_extent_x,
        max_extent_y,
        &mut func_visit_tile,
        &func_tile_blocked,
    );
    check_quadrant(
        &mut visited,
        start_x,
        start_y,
        1,
        -1,
        max_extent_x,
        min_extent_y,
        &mut func_visit_tile,
        &func_tile_blocked,
    );
    check_quadrant(
        &mut visited,
        start_x,
        start_y,
        -1,
        -1,
        min_extent_x,
        min_extent_y,
        &mut func_visit_tile,
        &func_tile_blocked,
    );
    check_quadrant(
        &mut visited,
        start_x,
        start_y,
        -1,
        1,
        min_extent_x,
        max_extent_y,
        &mut func_visit_tile,
        &func_tile_blocked,
    );
}

#[derive(Debug, Clone)]
struct Line {
    xi: i32,
    yi: i32,
    xf: i32,
    yf: i32,
}

impl Line {
    fn dx(&self) -> i32 {
        self.xf - self.xi
    }

    fn dy(&self) -> i32 {
        self.yf - self.yi
    }

    fn p_below(&self, x: i32, y: i32) -> bool {
        self.relative_slope(x, y) > 0
    }
    fn p_below_or_collinear(&self, x: i32, y: i32) -> bool {
        self.relative_slope(x, y) >= 0
    }
    fn p_above(&self, x: i32, y: i32) -> bool {
        self.relative_slope(x, y) < 0
    }
    fn p_above_or_collinear(&self, x: i32, y: i32) -> bool {
        self.relative_slope(x, y) <= 0
    }
    fn p_collinear(&self, x: i32, y: i32) -> bool {
        self.relative_slope(x, y) == 0
    }
    fn line_collinear(&self, line: &Line) -> bool {
        self.p_collinear(line.xi, line.yi) && self.p_collinear(line.xf, line.yf)
    }

    fn relative_slope(&self, x: i32, y: i32) -> i32 {
        self.dy() * (self.xf - x) - self.dx() * (self.yf - y)
    }
}

#[derive(Debug, Clone)]
struct ViewBump {
    x: i32,
    y: i32,
    parent: Option<Rc<ViewBump>>,
}

impl ViewBump {
    fn new(x: i32, y: i32, parent: Option<Rc<ViewBump>>) -> Self {
        Self { x, y, parent }
    }
}

#[derive(Debug, Clone)]
struct View {
    shallow_line: Line,
    steep_line: Line,
    shallow_bump: Option<Rc<ViewBump>>,
    steep_bump: Option<Rc<ViewBump>>,
}

impl View {
    fn new(shallow_line: Line, steep_line: Line) -> Self {
        Self {
            shallow_line,
            steep_line,
            shallow_bump: None,
            steep_bump: None,
        }
    }
}

fn check_quadrant(
    visited: &mut HashSet<IVec2>,
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    extent_x: i32,
    extent_y: i32,
    func_visit_tile: &mut impl FnMut(IVec2),
    func_tile_blocked: &impl Fn(IVec2) -> bool,
) {
    let mut active_views = vec![];

    let shallow_line = Line {
        xi: 0,
        yi: 1,
        xf: extent_x,
        yf: 0,
    };
    let steep_line = Line {
        xi: 1,
        yi: 0,
        xf: 0,
        yf: extent_y,
    };
    active_views.push(View::new(shallow_line, steep_line));
    let view_index = 0;

    let max_i = extent_x + extent_y;
    let mut i = 1;
    while i != max_i + 1 && active_views.len() > 0 {
        let start_j = if 0 > i - extent_x { 0 } else { i - extent_x };
        let max_j = if i < extent_y { i } else { extent_y };

        let mut j = start_j;
        while j != max_j + 1 && view_index < active_views.len() {
            let x = i - j;
            let y = j;

            visit_coord(
                visited,
                start_x,
                start_y,
                x,
                y,
                dx,
                dy,
                view_index,
                &mut active_views,
                func_visit_tile,
                func_tile_blocked,
            );

            j += 1;
        }
        i += 1;
    }
}

fn visit_coord(
    visited: &mut HashSet<IVec2>,
    start_x: i32,
    start_y: i32,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    mut view_index: usize,
    active_views: &mut Vec<View>,

    func_visit_tile: &mut impl FnMut(IVec2),
    func_tile_blocked: &impl Fn(IVec2) -> bool,
) {
    let top_left = (x, y + 1);
    let bottom_right = (x + 1, y);

    while view_index < active_views.len()
        && active_views[view_index]
            .steep_line
            .p_below_or_collinear(bottom_right.0, bottom_right.1)
    {
        view_index += 1;
    }

    if view_index == active_views.len()
        || active_views[view_index]
            .shallow_line
            .p_above_or_collinear(top_left.0, top_left.1)
    {
        return;
    }

    let real_x = x * dx;
    let real_y = y * dy;

    if !visited.contains(&IVec2 {
        x: start_x + real_x,
        y: start_y + real_y,
    }) {
        visited.insert(IVec2 {
            x: start_x + real_x,
            y: start_y + real_y,
        });
        func_visit_tile(IVec2 {
            x: start_x + real_x,
            y: start_y + real_y,
        })
    }

    let is_blocked = func_tile_blocked(IVec2 {
        x: start_x + real_x,
        y: start_y + real_y,
    });

    if !is_blocked {
        return;
    }

    if active_views[view_index]
        .shallow_line
        .p_above(bottom_right.0, bottom_right.1)
        && active_views[view_index]
            .steep_line
            .p_below(top_left.0, top_left.1)
    {
        active_views.remove(view_index);
    } else if active_views[view_index]
        .shallow_line
        .p_above(bottom_right.0, bottom_right.1)
    {
        add_shallow_bump(top_left.0, top_left.1, active_views, view_index);
        check_view(active_views, view_index);
    } else if active_views[view_index]
        .steep_line
        .p_below(top_left.0, top_left.1)
    {
        add_steep_bump(bottom_right.0, bottom_right.1, active_views, view_index);
        check_view(active_views, view_index);
    } else {
        let shallow_view_index = view_index;
        view_index += 1;
        let mut steep_view_index = view_index;

        active_views.insert(
            shallow_view_index,
            Clone::clone(&active_views[shallow_view_index]),
        );

        add_steep_bump(
            bottom_right.0,
            bottom_right.1,
            active_views,
            shallow_view_index,
        );
        if !check_view(active_views, shallow_view_index) {
            steep_view_index -= 1;
        }

        add_shallow_bump(top_left.0, top_left.1, active_views, steep_view_index);
        check_view(active_views, steep_view_index);
    }
}

fn add_shallow_bump(x: i32, y: i32, active_views: &mut Vec<View>, view_index: usize) {
    active_views[view_index].shallow_line.xf = x;
    active_views[view_index].shallow_line.yf = y;

    // The python implementers didn't consider that a hand-spun linked list SUCKS (especially in Rust)
    active_views[view_index].shallow_bump = Some(Rc::new(ViewBump::new(
        x,
        y,
        active_views[view_index].shallow_bump.clone(),
    )));

    let mut cur_bump = active_views[view_index].steep_bump.clone();
    while let Some(cur_bump_some) = &mut cur_bump {
        if active_views[view_index]
            .shallow_line
            .p_above(cur_bump_some.x, cur_bump_some.y)
        {
            active_views[view_index].shallow_line.xi = cur_bump_some.x;
            active_views[view_index].shallow_line.yi = cur_bump_some.y;
        }
        cur_bump = cur_bump_some.parent.clone();
    }
}

fn add_steep_bump(x: i32, y: i32, active_views: &mut Vec<View>, view_index: usize) {
    active_views[view_index].steep_line.xf = x;
    active_views[view_index].steep_line.yf = y;

    // The python implementers didn't consider that a hand-spun linked list SUCKS (especially in Rust)
    active_views[view_index].steep_bump = Some(Rc::new(ViewBump::new(
        x,
        y,
        active_views[view_index].steep_bump.clone(),
    )));

    let mut cur_bump = active_views[view_index].shallow_bump.clone();
    while let Some(cur_bump_some) = &mut cur_bump {
        if active_views[view_index]
            .steep_line
            .p_below(cur_bump_some.x, cur_bump_some.y)
        {
            active_views[view_index].steep_line.xi = cur_bump_some.x;
            active_views[view_index].steep_line.yi = cur_bump_some.y;
        }
        cur_bump = cur_bump_some.parent.clone();
    }
}

fn check_view(active_views: &mut Vec<View>, view_index: usize) -> bool {
    let shallow_line = active_views[view_index].shallow_line.clone();
    let steep_line = active_views[view_index].steep_line.clone();

    if shallow_line.line_collinear(&steep_line)
        && (shallow_line.p_collinear(0, 1) || shallow_line.p_collinear(1, 0))
    {
        active_views.remove(view_index);
        return false;
    } else {
        return true;
    }
}
