mod grid_impls;

use std::{
    collections::HashSet,
    ops::{Add, Sub},
};

use glam::{I64Vec2, IVec2, UVec2};

pub trait PPFOVGrid {
    fn get_square(&self, x: u32, y: u32) -> Option<PPFOVTile>;
    fn max_coord(&self) -> (u32, u32);
}

impl PPFOVGrid for () {
    fn get_square(&self, _: u32, _: u32) -> Option<PPFOVTile> {
        None
    }
    fn max_coord(&self) -> (u32, u32) {
        (0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PPFOVTile {
    Empty,
    Obstacle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IAngle(IVec2);

impl IAngle {
    /// Rotates this angle by 90 degrees clockwise
    #[must_use]
    fn rot_90_cw(self) -> Self {
        IAngle(IVec2::new(self.0.y, -self.0.x))
    }

    /// Rotates this angle by 90 degrees clockwise, `n` times
    #[must_use]
    fn rot_90_cw_multi(mut self, n: u8) -> Self {
        for _ in 0..n {
            self = self.rot_90_cw();
        }
        self
    }

    /// Returns the quadrant of this angle, from `1, 2, 3, 4`
    ///
    /// 2 1
    /// 3 4
    fn quadrant(self) -> u8 {
        let (x_pos, y_pos) = (!self.0.x.is_negative(), !self.0.y.is_negative());

        match (x_pos, y_pos) {
            (true, true) => 1,
            (false, true) => 2,
            (false, false) => 3,
            (true, false) => 4,
        }
    }

    /// Checks whether or not this angle lies between `a` and `b`
    ///
    /// More specifically, imagine rotating counter-clockwise from `a` until reaching `b`.
    /// Iff the line `self`, extending in both directions, is crossed at any point, return `true`
    fn within_pair_bidirectional(self, a: IAngle, b: IAngle, open: bool) -> bool {
        // First, rotate all angles so `a` lies in quadrant 1
        let times_to_rotate = a.quadrant() - 1;
        let a = a.rot_90_cw_multi(times_to_rotate);
        assert_eq!(a.quadrant(), 1);
        let b = b.rot_90_cw_multi(times_to_rotate);
        let c = self.rot_90_cw_multi(times_to_rotate);

        let qb = b.quadrant();
        let qc = c.quadrant();

        let lt = |a: IAngle, b: IAngle| -> bool {
            match open {
                true => a < b,
                false => a <= b,
            }
        };

        print!(
            "is {}(q{qc}) within {}(q1),{}(q{qb})? (open={open}) ",
            c.0, a.0, b.0
        );

        if qb > qc {
            println!("YES: qb>qc (q{qb} > q{qc})");
            return true;
        }
        if qc > qb {
            println!("NO: qc>qb (q{qc} > q{qb})");
            return false;
        }
        if qb == qc {
            if !lt(c, b) {
                // if lt(b, c) {
                println!("NO: b<c ({} < {})", b.0, c.0);
                return false;
            }
        }
        if 1 == qc {
            if !lt(a, c) {
                // if lt(c, a) {
                println!("NO: c<a ({} < {})", c.0, a.0);
                return false;
            }
        }

        println!("YES: All other conditions passed");
        true
    }
}

impl Ord for IAngle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.0;
        let b = other.0;
        // cmp(slope_self, slope_other)
        //
        // a.y/a.x < b.y/b.x
        // iff
        // a.y*b.x < a.x*b.y
        Ord::cmp(&(a.y * b.x), &(a.x * b.y))
    }
}

impl PartialOrd for IAngle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl std::ops::Neg for IAngle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// A line that extends infinitely far in both directions
#[derive(Clone, Copy, PartialEq, Eq)]
struct Line {
    start: UVec2,
    dir: IVec2,
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(start={}, dir={})", self.start, self.dir)
    }
}

impl Line {
    fn from_to(from: UVec2, to: UVec2) -> Self {
        Self {
            start: from,
            dir: IVec2::try_from(to).unwrap() - IVec2::try_from(from).unwrap(),
        }
    }

    /// `true` iff this line, extending infinitely in both directions, intersects `square`
    fn intersects_square(self, square: UVec2, open: bool) -> bool {
        let square = IVec2::try_from(I64Vec2::from(square) - I64Vec2::from(self.start)).unwrap();

        let top_left = IAngle(square + IVec2::new(0, 1));
        let bot_right = IAngle(square + IVec2::new(1, 0));
        let ray_angle = IAngle(self.dir.try_into().unwrap());

        ray_angle.within_pair_bidirectional(bot_right, top_left, open)
            || (-ray_angle).within_pair_bidirectional(bot_right, top_left, open)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct View {
    top: Line,
    bot: Line,
    prev_top_bumps: Vec<UVec2>,
    prev_bot_bumps: Vec<UVec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SquareViewIntersect {
    /// The top line intersects the square
    TopBump,
    /// The top line is above the square, the bottom line is below the square
    Between,
    /// The bottom line intersects the square
    BotBump,
    /// Both lines intersect the square
    Blocking,
    /// Above or below
    NoIntersect,
}

struct ProcessSquareRes {
    square_made_visible: Option<UVec2>,
    new_view: NewView,
}

#[derive(Debug, Clone)]
enum NewView {
    ViewBlocked {},
    OneView { new_view: View },
    Split { new_views: [View; 2] },
}

#[derive(Debug, Clone, Copy)]
enum BumpSide {
    Top,
    Bot,
}

impl View {
    fn is_line(&self) -> bool {
        self.bot == self.top
            || ((self.bot.dir == self.top.dir && self.bot.dir.x == self.bot.dir.y)
                && (self.bot.start.x == self.bot.start.y && self.top.start.x == self.top.start.y))
    }

    fn intersect_with_square(&self, square: UVec2) -> SquareViewIntersect {
        if square == UVec2::ZERO {
            println!("WARNING: INTERSECTING WITH SQUARE ZERO");
        }

        let br = IVec2::try_from(square).unwrap() + IVec2::new(1, 0);
        let tl = IVec2::try_from(square).unwrap() + IVec2::new(0, 1);
        let self_start = IVec2::try_from(self.top.start).unwrap();

        // let contains_top = IAngle(tl - self_start).within_pair_bidirectional(
        //     IAngle(self.bot.dir),
        //     IAngle(self.top.dir),
        //     true,
        // );
        // let intersects_top = self.top.intersects_square(square, false);
        // // let contains_bot = Angle(self.bot.dir) <= Angle(br - self.bot.start);
        // let contains_bot = IAngle(br - self_start).within_pair_bidirectional(
        //     IAngle(self.bot.dir),
        //     IAngle(self.top.dir),
        //     true,
        // );
        // let intersects_bot = self.bot.intersects_square(square, false);

        // println!("view={self:#?}\nsquare={square}\nintersects_top={intersects_top}\nintersects_bot={intersects_bot}\ncontains_top={contains_top}\ncontains_bot={contains_bot}");

        if self.top.intersects_square(square, false) && self.bot.intersects_square(square, false) {
            SquareViewIntersect::Blocking
        } else if self.top.intersects_square(square, true) {
            SquareViewIntersect::TopBump
        } else if self.bot.intersects_square(square, true) {
            SquareViewIntersect::BotBump
        } else if IAngle(tl - self_start).within_pair_bidirectional(
            IAngle(self.bot.dir),
            IAngle(self.top.dir),
            false,
        ) && IAngle(br - self_start).within_pair_bidirectional(
            IAngle(self.bot.dir),
            IAngle(self.top.dir),
            false,
        ) {
            SquareViewIntersect::Between
        } else {
            SquareViewIntersect::NoIntersect
        }

        // let contains_top = IAngle(tl - self_start).within_pair_bidirectional(
        //     IAngle(self.bot.dir),
        //     IAngle(self.top.dir),
        //     true,
        // );
        // let intersects_top = self.top.intersects_square(square, false);
        // // let contains_bot = Angle(self.bot.dir) <= Angle(br - self.bot.start);
        // let contains_bot = IAngle(br - self_start).within_pair_bidirectional(
        //     IAngle(self.bot.dir),
        //     IAngle(self.top.dir),
        //     true,
        // );
        // let intersects_bot = self.bot.intersects_square(square, false);

        // println!("view={self:#?}\nsquare={square}\nintersects_top={intersects_top}\nintersects_bot={intersects_bot}\ncontains_top={contains_top}\ncontains_bot={contains_bot}");

        // if intersects_top && intersects_bot {
        //     SquareViewIntersect::Blocking
        // } else if intersects_top {
        //     SquareViewIntersect::TopBump
        // } else if intersects_bot {
        //     SquareViewIntersect::BotBump
        // } else if contains_top && contains_bot {
        //     SquareViewIntersect::Between
        // } else {
        //     SquareViewIntersect::NoIntersect
        // }
    }

    #[cfg(none)]
    fn _process_top_bump(self, square: UVec2) -> NewView {
        let new_top_line_end = square + UVec2::new(1, 0);
        #[derive(Debug, Clone, Copy)]
        struct Candidate {
            square: UVec2,
            line: Line,
        }
        // Find the steepest possible line, that connects the player to `new_top_line_end`
        let mut new_top_line_candidates = vec![
            Candidate {
                square: UVec2::new(0, 0),
                line: Line::from_to(UVec2::new(1, 0), new_top_line_end),
            },
            Candidate {
                square: UVec2::new(0, 0),
                line: Line::from_to(UVec2::new(1, 1), new_top_line_end),
            },
        ];

        for candidate_square in self.prev_bot_bumps.iter().cloned() {
            let candidate_corner = candidate_square + UVec2::new(0, 1);
            new_top_line_candidates.push(Candidate {
                square: candidate_square,
                line: Line::from_to(candidate_corner, new_top_line_end),
            });
        }

        new_top_line_candidates.retain(|&candidate| {
            let mut candidate_failed = false;

            // Check for intersection with previous squares
            for possible_block in self
                .prev_bot_bumps
                .iter()
                .chain(self.prev_top_bumps.iter())
                .cloned()
            {
                if possible_block == candidate.square {
                    continue;
                }
                if candidate.line.intersects_square(possible_block) {
                    candidate_failed = true;
                    break;
                }
            }

            // Check for intersection with player
            if !candidate.line.intersects_square(UVec2::ZERO) {
                candidate_failed = true;
            }

            !candidate_failed
        });

        if new_top_line_candidates.is_empty() {
            NewView::ViewBlocked {}
        } else {
            // All candidates are valid, find the one with the highest angle (the steepest)
            let final_candidate = new_top_line_candidates
                .into_iter()
                .max_by_key(|c| {
                    if c.line.dir.y < 0 {
                        IAngle(-c.line.dir)
                    } else {
                        IAngle(c.line.dir)
                    }
                })
                .unwrap();
            NewView::OneView {
                new_view: View {
                    top: final_candidate.line,
                    bot: self.bot.clone(),
                    prev_top_bumps: [self.prev_top_bumps.clone(), vec![]].concat(),
                    prev_bot_bumps: self.prev_bot_bumps.clone(),
                },
            }
        }
    }

    fn process_obstacle_bump(self, square: UVec2, bump_side: BumpSide) -> NewView {
        let new_line_end = square
            + match bump_side {
                BumpSide::Top => UVec2::new(1, 0),
                BumpSide::Bot => UVec2::new(0, 1),
            };

        println!("Processing bump (type:{bump_side:?})...\nEnding line at: {new_line_end}");

        #[derive(Debug, Clone, Copy)]
        struct Candidate {
            // square: UVec2,
            line: Line,
        }
        // Find the steepest possible line, that connects the player to `new_line_end`
        let mut new_line_candidates = vec![
            // Candidate {
            //     // square: UVec2::new(0, 0),
            //     line: Line::from_to(
            //         match bump_side {
            //             BumpSide::Top => UVec2::new(1, 0),
            //             BumpSide::Bot => UVec2::new(0, 1),
            //         },
            //         new_line_end,
            //     ),
            // },
            // Candidate {
            //     // square: UVec2::new(0, 0),
            //     line: Line::from_to(UVec2::new(1, 1), new_line_end),
            // },
            Candidate {
                line: Line::from_to(
                    match bump_side {
                        BumpSide::Top => self.top.start.clone(),
                        BumpSide::Bot => self.bot.start.clone(),
                    },
                    new_line_end,
                ),
            },
        ];

        for candidate_square in match bump_side {
            BumpSide::Top => self.prev_bot_bumps.iter().cloned(),
            BumpSide::Bot => self.prev_top_bumps.iter().cloned(),
        } {
            let candidate_corner = candidate_square
                + match bump_side {
                    BumpSide::Top => UVec2::new(1, 0),
                    BumpSide::Bot => UVec2::new(0, 1),
                };
            new_line_candidates.push(Candidate {
                // square: candidate_square,
                line: Line::from_to(candidate_corner, new_line_end),
            });
        }

        println!("Bump candidates: {:?}", new_line_candidates);

        new_line_candidates.retain(|&candidate| {
            if candidate.line.dir == IVec2::ZERO {
                return false;
            }

            let other_line = match bump_side {
                BumpSide::Top => self.bot.clone(),
                BumpSide::Bot => self.top.clone(),
            };
            if candidate.line.dir == other_line.dir && candidate.line.start == other_line.start {
                return false;
            }

            // Check for intersection with previous squares
            for possible_block in self
                .prev_bot_bumps
                .iter()
                .chain(self.prev_top_bumps.iter())
                .cloned()
            {
                // if possible_block == candidate.square {
                //     continue;
                // }
                if candidate.line.intersects_square(possible_block, true) {
                    return false;
                }
            }

            // Check for intersection with player
            if !candidate.line.intersects_square(UVec2::ZERO, false) {
                return false;
            }

            true
        });

        if new_line_candidates.is_empty() {
            NewView::ViewBlocked {}
        } else {
            // All candidates are valid, find the one with the highest angle (the steepest)
            let key_extractor = |c: &Candidate| {
                if c.line.dir.y < 0 {
                    IAngle(-c.line.dir)
                } else {
                    IAngle(c.line.dir)
                }
            };
            let final_candidate = match bump_side {
                BumpSide::Top => new_line_candidates
                    .into_iter()
                    .max_by_key(key_extractor)
                    .unwrap(),
                BumpSide::Bot => new_line_candidates
                    .into_iter()
                    .min_by_key(key_extractor)
                    .unwrap(),
            };
            NewView::OneView {
                new_view: match bump_side {
                    BumpSide::Top => View {
                        top: final_candidate.line,
                        bot: self.bot.clone(),
                        prev_top_bumps: [self.prev_top_bumps.clone(), vec![square]].concat(),
                        prev_bot_bumps: self.prev_bot_bumps.clone(),
                    },
                    BumpSide::Bot => View {
                        top: self.top.clone(),
                        bot: final_candidate.line,
                        prev_top_bumps: self.prev_top_bumps.clone(),
                        prev_bot_bumps: [self.prev_bot_bumps.clone(), vec![square]].concat(),
                    },
                },
            }
        }
    }

    fn process_square(self, square: UVec2, square_ty: PPFOVTile) -> ProcessSquareRes {
        let intersect = self.intersect_with_square(square);
        println!("intersect_result: {intersect:?}");

        match (intersect, square_ty) {
            (SquareViewIntersect::TopBump, PPFOVTile::Obstacle) => ProcessSquareRes {
                square_made_visible: Some(square),
                new_view: self.process_obstacle_bump(square, BumpSide::Top),
            },
            (SquareViewIntersect::BotBump, PPFOVTile::Obstacle) => ProcessSquareRes {
                square_made_visible: Some(square),
                new_view: self.process_obstacle_bump(square, BumpSide::Bot),
            },
            (SquareViewIntersect::Between, PPFOVTile::Obstacle) => {
                let bot_view = self.clone().process_obstacle_bump(square, BumpSide::Top);
                let top_view = self.clone().process_obstacle_bump(square, BumpSide::Bot);
                let mut new_views = vec![];
                for v in [bot_view, top_view] {
                    match v {
                        NewView::ViewBlocked {} => (),
                        NewView::OneView { new_view: a } => new_views.push(a),
                        NewView::Split { new_views: [a, b] } => {
                            new_views.push(a);
                            new_views.push(b);
                            println!("WARNING: SplitView returned from `process_obstacle_bump`")
                        }
                    }
                }
                let new_view = match &new_views[..] {
                    [] => NewView::ViewBlocked {},
                    [a] => NewView::OneView {
                        new_view: a.clone(),
                    },
                    [a, b] => NewView::Split {
                        new_views: [a.clone(), b.clone()],
                    },
                    _ => unreachable!(),
                };
                ProcessSquareRes {
                    square_made_visible: Some(square),
                    new_view,
                }
            }

            (SquareViewIntersect::Blocking, PPFOVTile::Obstacle) => ProcessSquareRes {
                square_made_visible: Some(square),
                new_view: NewView::ViewBlocked {},
            },
            (SquareViewIntersect::NoIntersect, _) => ProcessSquareRes {
                square_made_visible: None,
                new_view: NewView::OneView { new_view: self },
            },
            (_, PPFOVTile::Empty) => ProcessSquareRes {
                square_made_visible: Some(square),
                new_view: NewView::OneView { new_view: self },
            },
        }
    }
}

fn build_view_grid_coords_iter() -> impl Iterator<Item = UVec2> {
    let mut x = 0u32;
    let mut y = 0u32;
    std::iter::from_fn(move || {
        // 9
        // 5 8
        // 2 4 7
        // 0 1 3 6
        if x == 0 {
            x = y + 1;
            y = 0;
        } else {
            x -= 1;
            y += 1;
        }
        Some(UVec2::new(x, y))
    })
}

pub fn build_view_grid(quadrant: &impl PPFOVGrid) -> HashSet<(u32, u32)> {
    let mut views: Vec<View> = vec![View {
        bot: Line::from_to(UVec2::new(0, 1), UVec2::new(10, 0)),
        top: Line::from_to(UVec2::new(1, 0), UVec2::new(0, 10)),
        prev_bot_bumps: Vec::new(),
        prev_top_bumps: Vec::new(),
    }];
    let mut visible: HashSet<UVec2> = HashSet::new();
    // Variable used within loop, kept here to maintain allocation
    let mut new_views: Vec<View> = Vec::new();
    for square in build_view_grid_coords_iter() {
        if views.is_empty() {
            break;
        }

        let limit = quadrant.max_coord();
        if square.x > limit.0 && square.y > limit.1 {
            break;
        }

        let Some(tile) = quadrant.get_square(square.x, square.y) else {
            continue;
        };

        println!("\n\n\n==={square}===");
        println!("==\nviews_before: {views:#?}\n==");
        new_views.clear();
        for view in &views {
            let ProcessSquareRes {
                square_made_visible,
                new_view,
            } = view.clone().process_square(square, tile);
            if let Some(sq) = square_made_visible {
                visible.insert(sq);
            }
            println!("=\nTransform:\n{:#?} ---> {:#?}\n=", view, new_view);
            match new_view {
                NewView::ViewBlocked {} => (),
                NewView::OneView { new_view: v } => new_views.push(v),
                NewView::Split { new_views: v } => new_views.extend(v),
            }
        }
        println!("==\nviews_after: {new_views:#?}\n==");
        std::mem::swap(&mut views, &mut new_views);
        views.retain(|view| !view.is_line());
    }

    visible.into_iter().map(|v| (v.x, v.y)).collect()
}
