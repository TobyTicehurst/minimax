#![allow(dead_code)]

mod mancala;
mod minimax;

use mancala::MancalaGameState;
use minimax::{GameState, MinimaxSolver};
use std::time::SystemTime;
use tracing::info;
use tracing::level_filters::LevelFilter;

fn iterative_deepening(max_depth: u32, max_table_depth: u32, max_table_capacity: usize) {
    MinimaxSolver::mtdf_with_memory_iterative_deepening(
        &MancalaGameState::default(),
        max_depth,
        max_table_depth,
        max_table_capacity,
    );
}

fn analyse(max_depth: u32) {
    for depth in 0..(max_depth + 1) {
        let now = SystemTime::now();
        // let test_position = MancalaGameState {
        //     pits: [0, 0, 0, 0, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0],
        //     turn: true,
        //     game_over: false
        // };
        let value = MinimaxSolver::solve(&MancalaGameState::default(), depth);
        let time = now.elapsed().unwrap();
        info!(
            "Value: {}, Depth: {}, Time: {}ms",
            value,
            depth,
            time.as_millis()
        );
    }
}

fn play_game(max_depth: u32) {
    let mut position = MancalaGameState::default();
    loop {
        info!("Position: {:?}", position);

        // get valid moves (for easier tracking)
        let mut valid_moves = Vec::new();
        if position.turn {
            for i in 0..6 {
                if position.pits[i] != 0 {
                    valid_moves.push(i);
                }
            }
        } else {
            for i in 7..13 {
                if position.pits[i] != 0 {
                    valid_moves.push(i);
                }
            }
        }
        info!("Valid moves: {:?}", valid_moves);

        let children: Vec<MancalaGameState> = position.get_children().into_iter().rev().collect();
        let mut values = Vec::new();
        for child in &children {
            let value = MinimaxSolver::solve(child, max_depth);
            values.push(value);
        }

        info!("Values: {:?}", values);

        if position.turn {
            if let Some(best_position_index) = values
                .iter()
                .enumerate()
                .max_by_key(|(_, &value)| value)
                .map(|(index, _)| index)
            {
                info!("Best move: {:?}", valid_moves[best_position_index]);
                position = children[best_position_index];
                if position.is_game_over() {
                    return;
                }
            } else {
                return;
            }
        } else {
            if let Some(best_position_index) = values
                .iter()
                .enumerate()
                .min_by_key(|(_, &value)| value)
                .map(|(index, _)| index)
            {
                info!("Best move: {:?}", valid_moves[best_position_index]);
                position = children[best_position_index];
                if position.is_game_over() {
                    return;
                }
            } else {
                return;
            }
        }
    }
}

fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_ansi(false)
        .without_time()
        .init();

    //play_game(15);
    iterative_deepening(16, 10, 10000000);
}
