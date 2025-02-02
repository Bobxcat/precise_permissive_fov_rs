use glam::UVec2;
use precise_permissive_fov::*;
use rand::random_bool;

fn format_grid<T: Into<String>>(
    width: usize,
    height: usize,
    print_cell: impl Fn(UVec2) -> T,
) -> String {
    let mut s = String::new();
    for y in 0..height {
        for x in 0..width {
            s.push_str(&print_cell(UVec2::new(x as u32, y as u32)).into());
            s.push(' ');
        }
        s.push('\n');
    }
    s
}

fn run_with_map<const W: usize, const H: usize>(map: [[PPFOVTile; W]; H]) {
    let visible = build_view_grid(&map);
    let visible_sorted = {
        let mut v = visible.iter().copied().collect::<Vec<_>>();
        v.sort();
        v
    };
    let map_str = format_grid(W, H, |UVec2 { x, y }| {
        if x == 0 && y == 0 {
            return '@';
        }
        let t = map[y as usize][x as usize];
        match t {
            PPFOVTile::Empty => '.',
            PPFOVTile::Obstacle => '#',
        }
    });
    let visible_str = format_grid(W, H, |UVec2 { x, y }| match visible.contains(&(x, y)) {
        true => 'O',
        false => 'X',
    });
    let vismap_str = format_grid(W, H, |UVec2 { x, y }| {
        if x == 0 && y == 0 {
            return "@";
        }
        match (visible.contains(&(x, y)), map[y as usize][x as usize]) {
            (true, PPFOVTile::Empty) => ".",
            (true, PPFOVTile::Obstacle) => "#",
            (false, PPFOVTile::Empty) => "░",
            (false, PPFOVTile::Obstacle) => "█",
        }
    });
    println!("\n\n======\n");
    println!(
        "visible={visible_sorted:?}\nvis=\n{visible_str}\nmap=\n{map_str}\nvismap=\n{vismap_str}"
    );
}

fn run_rand<const W: usize, const H: usize>() {
    run_with_map::<W, H>(std::array::from_fn(|_| {
        std::array::from_fn(|_| match random_bool(0.3) {
            true => PPFOVTile::Obstacle,
            false => PPFOVTile::Empty,
        })
    }));
}

fn main() {
    // run_rand::<15, 7>();
    let c = [
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Obstacle,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Obstacle,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
    ];
    run_with_map(c);
    let a = [
        [
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
        ],
    ];
    let b = [
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
        ],
        [
            PPFOVTile::Empty,
            PPFOVTile::Empty,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
            PPFOVTile::Obstacle,
        ],
    ];

    // run_with_map(a);
    // run_with_map(b);
}
