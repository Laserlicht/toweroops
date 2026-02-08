use rand::Rng;

use super::types::{Cell, CellKind, Selection};

pub const BOARD_SIZE: usize = 8;

/// The 8×8 game board.
#[derive(Debug, Clone)]
pub struct Board {
    cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    /// Create a new randomly-populated board and an initial selection axis.
    pub fn new_random() -> (Self, Selection) {
        let mut rng = rand::thread_rng();
        let mut cells = [[Cell::default(); BOARD_SIZE]; BOARD_SIZE];

        for col in 0..BOARD_SIZE {
            for row in 0..BOARD_SIZE {
                // Determine cell kind (same probability distribution as the original)
                let kind = match rng.gen_range(0..11) {
                    0 => CellKind::Banana,
                    1..=6 => CellKind::Stone,
                    _ => CellKind::Bomb,
                };

                // Determine value (only relevant for Stone and Bomb)
                let value = match kind {
                    CellKind::Stone | CellKind::Bomb => match rng.gen_range(0..11) {
                        0 => 3,
                        1..=2 => 2,
                        3..=6 => 1,
                        _ => 0,
                    },
                    _ => 0,
                };

                cells[col][row] = Cell { kind, value };
            }
        }

        let selection = if rng.gen_bool(0.5) {
            Selection::Row(rng.gen_range(0..BOARD_SIZE))
        } else {
            Selection::Column(rng.gen_range(0..BOARD_SIZE))
        };

        (Self { cells }, selection)
    }

    pub fn get(&self, col: usize, row: usize) -> &Cell {
        &self.cells[col][row]
    }

    pub fn clear(&mut self, col: usize, row: usize) {
        self.cells[col][row] = Cell::default();
    }

    /// Check whether every cell in the given selection axis is empty.
    pub fn selection_exhausted(&self, selection: Selection) -> bool {
        for i in 0..BOARD_SIZE {
            let cell = match selection {
                Selection::Row(r) => &self.cells[i][r],
                Selection::Column(c) => &self.cells[c][i],
            };
            if cell.kind != CellKind::Empty {
                return false;
            }
        }
        true
    }
}
