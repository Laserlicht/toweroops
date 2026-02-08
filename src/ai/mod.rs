use rand::seq::SliceRandom;

use crate::game::field::{Board, BOARD_SIZE};
use crate::game::types::{CellKind, Selection};

/// 5 AI levels: 0 (random) .. 4 (deep minimax).
pub const MAX_AI_LEVEL: i32 = 4;

/// Calculate the best move for the given AI level.
/// Returns (col, row).
pub fn calculate_move(
    level: i32,
    board: &Board,
    selection: Selection,
    tower_self: i32,
    tower_opponent: i32,
) -> (usize, usize) {
    match level {
        0 => random_move(board, selection),
        1 => greedy_move(board, selection),
        2 => minimax_move(board, selection, tower_self, tower_opponent, 2),
        3 => minimax_move(board, selection, tower_self, tower_opponent, 4),
        4 => minimax_move(board, selection, tower_self, tower_opponent, 8),
        _ => minimax_move(board, selection, tower_self, tower_opponent, 8),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Level 0 – Random
// ════════════════════════════════════════════════════════════════════════════

fn random_move(board: &Board, selection: Selection) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let mut candidates: Vec<usize> = (0..BOARD_SIZE).collect();
    candidates.shuffle(&mut rng);

    for &i in &candidates {
        let (col, row) = sel_coords(selection, i);
        if board.get(col, row).kind != CellKind::Empty {
            return (col, row);
        }
    }
    sel_coords(selection, 0)
}

// ════════════════════════════════════════════════════════════════════════════
// Level 1 – Greedy (pick best immediate value)
// ════════════════════════════════════════════════════════════════════════════

fn greedy_move(board: &Board, selection: Selection) -> (usize, usize) {
    let mut best_score = i32::MIN;
    let mut best_candidates = Vec::new();

    for i in 0..BOARD_SIZE {
        let (col, row) = sel_coords(selection, i);
        let cell = board.get(col, row);
        if cell.kind == CellKind::Empty {
            continue;
        }
        let score = cell_value(cell.kind, cell.value);
        if score > best_score {
            best_score = score;
            best_candidates.clear();
            best_candidates.push(i);
        } else if score == best_score {
            best_candidates.push(i);
        }
    }

    let mut rng = rand::thread_rng();
    let &idx = best_candidates.choose(&mut rng).unwrap_or(&0);
    sel_coords(selection, idx)
}

// ════════════════════════════════════════════════════════════════════════════
// Levels 2–4 – Minimax with Alpha-Beta Pruning
// ════════════════════════════════════════════════════════════════════════════

const MAX_TOWER: i32 = 20;

/// State used during minimax search (to avoid cloning Board repeatedly).
#[derive(Clone)]
struct SearchState {
    board: Board,
    selection: Selection,
    tower_me: i32,  // the AI player ("maximizer")
    tower_opp: i32, // the human player ("minimizer")
}

fn minimax_move(
    board: &Board,
    selection: Selection,
    tower_self: i32,
    tower_opponent: i32,
    depth: i32,
) -> (usize, usize) {
    let mut rng = rand::thread_rng();

    let state = SearchState {
        board: board.clone(),
        selection,
        tower_me: tower_self,
        tower_opp: tower_opponent,
    };

    let mut best_score = i32::MIN;
    let mut best_candidates = Vec::new();

    // Evaluate all possible moves
    for i in 0..BOARD_SIZE {
        let (col, row) = sel_coords(selection, i);
        let cell = *board.get(col, row);
        if cell.kind == CellKind::Empty {
            continue;
        }

        let mut child = state.clone();
        apply_move_to(&mut child, col, row, true); // true = AI's move (maximizer)

        // Check for immediate terminal state
        if child.tower_me >= MAX_TOWER {
            return (col, row); // instant win – take it
        }

        let score = minimax(&child, depth - 1, i32::MIN, i32::MAX, false);

        if score > best_score {
            best_score = score;
            best_candidates.clear();
            best_candidates.push(i);
        } else if score == best_score {
            best_candidates.push(i);
        }
    }

    let &idx = best_candidates.choose(&mut rng).unwrap_or(&0);
    sel_coords(selection, idx)
}

/// Minimax with alpha-beta pruning.
/// `maximizing` = true means it's the AI's turn, false = opponent's turn.
fn minimax(
    state: &SearchState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    // Terminal conditions
    if state.tower_me >= MAX_TOWER {
        return 10000 + depth; // AI wins – prefer faster wins
    }
    if state.tower_opp >= MAX_TOWER {
        return -10000 - depth; // opponent wins
    }

    // Check if selection is exhausted (no moves available)
    if state.board.selection_exhausted(state.selection) {
        return evaluate_final(state);
    }

    if depth <= 0 {
        return evaluate(state);
    }

    if maximizing {
        let mut best = i32::MIN;
        for i in 0..BOARD_SIZE {
            let (col, row) = sel_coords(state.selection, i);
            let cell = *state.board.get(col, row);
            if cell.kind == CellKind::Empty {
                continue;
            }

            let mut child = state.clone();
            apply_move_to(&mut child, col, row, true);
            let score = minimax(&child, depth - 1, alpha, beta, false);

            best = best.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break; // beta cutoff
            }
        }
        if best == i32::MIN {
            evaluate_final(state) // no moves available
        } else {
            best
        }
    } else {
        let mut best = i32::MAX;
        for i in 0..BOARD_SIZE {
            let (col, row) = sel_coords(state.selection, i);
            let cell = *state.board.get(col, row);
            if cell.kind == CellKind::Empty {
                continue;
            }

            let mut child = state.clone();
            apply_move_to(&mut child, col, row, false);
            let score = minimax(&child, depth - 1, alpha, beta, true);

            best = best.min(score);
            beta = beta.min(score);
            if alpha >= beta {
                break; // alpha cutoff
            }
        }
        if best == i32::MAX {
            evaluate_final(state)
        } else {
            best
        }
    }
}

/// Apply a move to a SearchState, modifying it in place.
fn apply_move_to(state: &mut SearchState, col: usize, row: usize, is_maximizer: bool) {
    let cell = *state.board.get(col, row);

    let tower = if is_maximizer {
        &mut state.tower_me
    } else {
        &mut state.tower_opp
    };

    match cell.kind {
        CellKind::Stone => {
            *tower = (*tower + cell.value + 1).min(MAX_TOWER);
        }
        CellKind::Bomb => {
            *tower = (*tower - cell.value - 1).max(0);
        }
        _ => {}
    }

    // Switch selection (banana keeps same axis)
    if cell.kind != CellKind::Banana {
        state.selection = match state.selection {
            Selection::Row(_) => Selection::Column(col),
            Selection::Column(_) => Selection::Row(row),
        };
    }

    state.board.clear(col, row);
}

/// Heuristic evaluation of a non-terminal position.
/// Positive = good for AI, negative = good for opponent.
fn evaluate(state: &SearchState) -> i32 {
    let tower_diff = (state.tower_me - state.tower_opp) * 100;

    // Evaluate the available moves for the current player on the active selection
    let mut axis_value = 0i32;
    let mut available_count = 0i32;
    for i in 0..BOARD_SIZE {
        let (col, row) = sel_coords(state.selection, i);
        let cell = state.board.get(col, row);
        if cell.kind != CellKind::Empty {
            axis_value += cell_value(cell.kind, cell.value);
            available_count += 1;
        }
    }

    // Look ahead at what the opponent will have access to
    let mut opponent_axis_value = 0i32;
    for col in 0..BOARD_SIZE {
        for row in 0..BOARD_SIZE {
            let cell = state.board.get(col, row);
            if cell.kind != CellKind::Empty {
                // Check if this cell is on a potential future selection
                let val = cell_value(cell.kind, cell.value);
                opponent_axis_value += val;
            }
        }
    }

    // Weighted combination
    tower_diff + axis_value * 8 - opponent_axis_value / (BOARD_SIZE as i32) + available_count * 5
}

/// Evaluate a terminal position (game over due to exhaustion or tower reached).
fn evaluate_final(state: &SearchState) -> i32 {
    if state.tower_me > state.tower_opp {
        5000 // AI wins
    } else if state.tower_me < state.tower_opp {
        -5000 // opponent wins
    } else {
        0 // draw
    }
}

/// Convert a selection + index to (col, row) coordinates.
fn sel_coords(selection: Selection, idx: usize) -> (usize, usize) {
    match selection {
        Selection::Row(r) => (idx, r),
        Selection::Column(c) => (c, idx),
    }
}

/// The immediate value of picking a cell. Positive = good for the picker.
fn cell_value(kind: CellKind, value: i32) -> i32 {
    match kind {
        CellKind::Empty => 0,
        CellKind::Stone => (value + 1) * 10, // stones are great (+1 to +4)
        CellKind::Bomb => -(value + 1) * 10, // bombs are bad (-1 to -4)
        CellKind::Banana => 1,               // banana is near-neutral
    }
}
