/// The kind of object occupying a cell on the 8×8 board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Empty,
    Bomb,
    Stone,
    Banana,
}

/// A single cell on the game board.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub kind: CellKind,
    /// Strength / value of the cell (0–3 for bombs and stones, ignored for banana/empty).
    pub value: i32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            kind: CellKind::Empty,
            value: 0,
        }
    }
}

/// Outcome of the game from the human player's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Running,
    Won,
    Lost,
    Drawn,
}

/// Which axis is currently selected for the next move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// A full column (vertical) is active – the player must pick a row in that column.
    Column(usize),
    /// A full row (horizontal) is active – the player must pick a column in that row.
    Row(usize),
}

/// Cumulative win/loss/draw statistics across multiple rounds.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Statistics {
    pub player_wins: u32,
    pub computer_wins: u32,
    pub draws: u32,
}

impl Statistics {
    pub fn record(&mut self, outcome: GameOutcome) {
        match outcome {
            GameOutcome::Won => self.player_wins += 1,
            GameOutcome::Lost => self.computer_wins += 1,
            GameOutcome::Drawn => self.draws += 1,
            GameOutcome::Running => {}
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
