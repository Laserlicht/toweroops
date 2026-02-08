use super::field::{Board, BOARD_SIZE};
use super::types::{CellKind, GameOutcome, Selection, Statistics};
use crate::ai;

const MAX_TOWER_HEIGHT: i32 = 20;

/// Central game state holding everything needed for one round.
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub selection: Selection,
    pub tower_player: i32,
    pub tower_computer: i32,
    pub outcome: GameOutcome,
    pub moves_made: u32,
    pub ai_level: i32,
    pub tip: Option<(usize, usize)>,
    pub hovered: Option<(usize, usize)>,
    pub statistics: Statistics,
}

impl GameState {
    pub fn new() -> Self {
        let (board, selection) = Board::new_random();
        Self {
            board,
            selection,
            tower_player: 0,
            tower_computer: 0,
            outcome: GameOutcome::Running,
            moves_made: 0,
            ai_level: 2,
            tip: None,
            hovered: None,
            statistics: Statistics::default(),
        }
    }

    /// Start a fresh round, keeping statistics and AI level.
    pub fn new_game(&mut self) {
        let (board, selection) = Board::new_random();
        self.board = board;
        self.selection = selection;
        self.tower_player = 0;
        self.tower_computer = 0;
        self.outcome = GameOutcome::Running;
        self.moves_made = 0;
        self.tip = None;
        self.hovered = None;
    }

    /// Returns `true` if the cell at (col, row) is a valid target for the current selection.
    pub fn is_valid_move(&self, col: usize, row: usize) -> bool {
        if self.outcome != GameOutcome::Running {
            return false;
        }
        if col >= BOARD_SIZE || row >= BOARD_SIZE {
            return false;
        }
        let in_selection = match self.selection {
            Selection::Column(c) => col == c,
            Selection::Row(r) => row == r,
        };
        in_selection && self.board.get(col, row).kind != CellKind::Empty
    }

    /// Execute a move at (col, row). `is_player` indicates whether the human is acting.
    /// Does NOT automatically trigger the computer's turn – the caller is responsible.
    pub fn make_move(&mut self, col: usize, row: usize, is_player: bool) -> MoveResult {
        if !self.is_valid_move(col, row) {
            return MoveResult::Invalid;
        }

        let cell = *self.board.get(col, row);

        // Apply tower height change
        let tower = if is_player {
            &mut self.tower_player
        } else {
            &mut self.tower_computer
        };

        match cell.kind {
            CellKind::Stone => {
                *tower = (*tower + cell.value + 1).min(MAX_TOWER_HEIGHT);
            }
            CellKind::Bomb => {
                *tower = (*tower - cell.value - 1).max(0);
            }
            _ => {}
        }

        // Switch selection axis (banana keeps the same axis)
        if cell.kind != CellKind::Banana {
            self.selection = match self.selection {
                Selection::Row(_) => Selection::Column(col),
                Selection::Column(_) => Selection::Row(row),
            };
        }

        self.board.clear(col, row);
        self.moves_made += 1;
        self.tip = None;

        // Check win conditions
        if self.tower_player >= MAX_TOWER_HEIGHT {
            self.finish(GameOutcome::Won);
            return MoveResult::GameOver;
        }
        if self.tower_computer >= MAX_TOWER_HEIGHT {
            self.finish(GameOutcome::Lost);
            return MoveResult::GameOver;
        }

        // Check if all cells in the active selection are empty (no moves left)
        if self.board.selection_exhausted(self.selection) {
            let outcome = if self.tower_player > self.tower_computer {
                GameOutcome::Won
            } else if self.tower_player < self.tower_computer {
                GameOutcome::Lost
            } else {
                GameOutcome::Drawn
            };
            self.finish(outcome);
            return MoveResult::GameOver;
        }

        MoveResult::Continue
    }

    /// Let the AI pick a move. Returns the chosen (col, row).
    pub fn compute_ai_move(&self) -> (usize, usize) {
        ai::calculate_move(
            self.ai_level,
            &self.board,
            self.selection,
            self.tower_computer,
            self.tower_player,
        )
    }

    /// Let the AI pick and immediately execute a move.
    #[allow(dead_code)]
    pub fn computer_turn(&mut self) {
        if self.outcome != GameOutcome::Running {
            return;
        }
        let (col, row) = self.compute_ai_move();
        self.make_move(col, row, false);
    }

    /// Calculate and store a suggested move for the player.
    pub fn get_tip(&mut self) {
        if self.outcome != GameOutcome::Running {
            return;
        }
        let (col, row) = ai::calculate_move(
            ai::MAX_AI_LEVEL,
            &self.board,
            self.selection,
            self.tower_player,
            self.tower_computer,
        );
        self.tip = Some((col, row));
    }

    /// Player resigns the current game.
    pub fn surrender(&mut self) {
        self.finish(GameOutcome::Lost);
    }

    /// Update the hover position (for highlighting).
    pub fn update_hover(&mut self, col: usize, row: usize) {
        if col >= BOARD_SIZE || row >= BOARD_SIZE {
            self.hovered = None;
            return;
        }
        let valid = match self.selection {
            Selection::Column(c) => col == c,
            Selection::Row(r) => row == r,
        };
        self.hovered = if valid { Some((col, row)) } else { None };
    }

    pub fn clear_hover(&mut self) {
        self.hovered = None;
    }

    fn finish(&mut self, outcome: GameOutcome) {
        self.outcome = outcome;
        self.statistics.record(outcome);
        // Persist updated statistics; ignore errors to avoid breaking game flow.
        let _ = crate::storage::save_statistics(&self.statistics);
    }
}

/// Result of a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    /// Move was invalid / rejected.
    Invalid,
    /// Move applied, game is still running (opponent's turn next).
    Continue,
    /// Move applied, game is now over.
    GameOver,
}
