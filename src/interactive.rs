#![allow(dead_code)]

use crate::mancala::{MancalaEndgamesTable, MancalaGameState};
use crate::minimax::{EndgamesTable, GameState, Solver};
use std::rc::Rc;
use std::time::SystemTime;
use tracing::info;

fn solve_game(minimax_solver: &mut Solver<MancalaGameState, MancalaEndgamesTable>) {
    let mut game_state_cache = MancalaGameState::generate_children_memory(minimax_solver.max_depth);
    let mut guess = 0;
    let mut depth = 0;
    while depth <= minimax_solver.max_depth {
        let now = SystemTime::now();
        guess = minimax_solver.mtdf_with_memory(guess, depth, &mut game_state_cache);
        let time = now.elapsed().unwrap();
        info!(
            "Value: {}, Depth: {}, Time: {}ms",
            guess,
            depth,
            time.as_millis()
        );

        depth += 2;
    }

    // final
    let now = SystemTime::now();
    let mut game_state_cache_10000 = MancalaGameState::generate_children_memory(10000);
    guess = minimax_solver.mtdf_with_memory(guess, 10000, &mut game_state_cache_10000);

    let time = now.elapsed().unwrap();
    info!(
        "Value: {}, Depth: {}, Time: {}ms",
        guess,
        10000,
        time.as_millis()
    );
}

// TODO optimise this and allow choice of algorithm
pub fn play_game() {
    // input
    let max_depth = 28;
    let endgames_table_max_stones = 22;
    let max_table_depth = 20;
    let transposition_table_capacity = 10000000;

    let mut endgames_table = MancalaEndgamesTable::new(endgames_table_max_stones);
    endgames_table.calculate_endgames();
    let rc_endgames_table = Rc::new(endgames_table);

    let mut minimax_solver = Solver::new(
        MancalaGameState::default(),
        max_depth,
        max_table_depth,
        transposition_table_capacity,
        &rc_endgames_table,
    );

    let now = SystemTime::now();
    solve_game(&mut minimax_solver);
    let time = now.elapsed().unwrap();
    info!("Total time to solve: {}ms", time.as_millis());

    let mut children_memory = MancalaGameState::generate_children_memory(max_depth);
    let mut game_state = MancalaGameState::default();

    for _move_number in 0..max_table_depth {
        info!("Game state: {:?}", game_state);
        if game_state.is_game_over() {
            return;
        }

        // get valid moves (for easier tracking)
        let mut valid_moves = Vec::new();
        if game_state.turn {
            for i in 0..6 {
                if game_state.pits[i] != 0 {
                    valid_moves.push(i);
                }
            }
        } else {
            for i in 7..13 {
                if game_state.pits[i] != 0 {
                    valid_moves.push(i);
                }
            }
        }
        info!("Valid moves: {:?}", valid_moves);

        let mut best_guess;
        if game_state.turn {
            best_guess = i32::MIN;
        } else {
            best_guess = i32::MAX;
        }
        let mut next_game_state = MancalaGameState::default();

        // TODO fix with smart reordering
        let children: Vec<MancalaGameState> = game_state
            .get_children(&mut children_memory[0])
            .to_owned()
            .into_iter()
            .rev()
            .collect();
        for (child, child_move) in children.iter().zip(valid_moves) {
            if let Some(lookup) = minimax_solver.transposition_table.lookup(child) {
                if lookup.lower_bound.bound == lookup.upper_bound.bound {
                    info!(
                        "    Move: {}, Eval: {}",
                        child_move, lookup.lower_bound.bound
                    );
                    if game_state.turn && lookup.lower_bound.bound > best_guess {
                        next_game_state = child.clone();
                        best_guess = lookup.lower_bound.bound;
                    } else if !game_state.turn && lookup.lower_bound.bound < best_guess {
                        next_game_state = child.clone();
                        best_guess = lookup.lower_bound.bound;
                    }
                } else {
                    info!(
                        "    Move: {}, Eval: {}:{}",
                        child_move, lookup.lower_bound.bound, lookup.upper_bound.bound
                    );
                }
            } else {
                info!("Oh nos")
            }
        }

        info!("best_guess");
        game_state = next_game_state.clone();
    }
}
