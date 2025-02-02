use crate::{PPFOVGrid, PPFOVTile};

impl<const W: usize, const H: usize> PPFOVGrid for [[PPFOVTile; W]; H] {
    fn get_square(&self, x: u32, y: u32) -> Option<PPFOVTile> {
        self.get(y as usize)?.get(x as usize).copied()
    }

    fn max_coord(&self) -> (u32, u32) {
        (W as u32, H as u32)
    }
}
