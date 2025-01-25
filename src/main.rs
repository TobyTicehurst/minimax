#![allow(dead_code)]

mod mancala;
mod minimax;

use mancala::MancalaGameState;
use minimax::MinimaxSolver;
use std::time::SystemTime;
use tracing::info;
use tracing::level_filters::LevelFilter;

fn iterative_deepening(max_depth: u32, max_table_depth: u32, max_table_capacity: usize) {
    let mut children_memory = MancalaGameState::generate_children_memory(max_depth);

    MinimaxSolver::mtdf_with_memory_iterative_deepening(
        &MancalaGameState::default(),
        max_depth,
        max_table_depth,
        max_table_capacity,
        &mut children_memory,
    );
}

enum Solver {
    Minimax,
    Alphabeta,
    Mtdf,
}

fn analyse(solver: Solver, max_depth: u32) {
    let mut children_memory = MancalaGameState::generate_children_memory(max_depth);
    for depth in 0..(max_depth + 1) {
        // start timer
        let now = SystemTime::now();
        let value = match solver {
            Solver::Minimax => {
                MinimaxSolver::minimax(&MancalaGameState::default(), depth, &mut children_memory)
            }
            Solver::Alphabeta => MinimaxSolver::alphabeta(
                &MancalaGameState::default(),
                depth,
                i32::MIN,
                i32::MAX,
                &mut children_memory,
            ),
            Solver::Mtdf => MinimaxSolver::mtdf_no_memory(
                &MancalaGameState::default(),
                1,
                depth,
                &mut children_memory,
            ),
        };
        let time = now.elapsed().unwrap();
        info!(
            "Value: {}, Depth: {}, Time: {}ms",
            value,
            depth,
            time.as_millis()
        );
    }
}

fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_ansi(false)
        .without_time()
        .init();

    let max_depth = 26;
    // info!("Alphabeta");
    // analyse(Solver::Alphabeta, max_depth);
    // info!("Mtdf");
    // analyse(Solver::Mtdf, max_depth);
    info!("Iterative Deepening");
    iterative_deepening(max_depth, 16, 10000000);
}

// TODO optimise this and allow choice of algorithm
// fn play_game(max_depth: u32) {
//     let mut children_memory = MancalaGameState::generate_children_memory(max_depth);
//     let mut position = MancalaGameState::default();
//     loop {
//         info!("Position: {:?}", position);

//         // get valid moves (for easier tracking)
//         let mut valid_moves = Vec::new();
//         if position.turn {
//             for i in 0..6 {
//                 if position.pits[i] != 0 {
//                     valid_moves.push(i);
//                 }
//             }
//         } else {
//             for i in 7..13 {
//                 if position.pits[i] != 0 {
//                     valid_moves.push(i);
//                 }
//             }
//         }
//         info!("Valid moves: {:?}", valid_moves);

//         let children: Vec<MancalaGameState> = position.get_children(&mut children_memory[0]).to_owned().into_iter().rev().collect();
//         let mut values = Vec::new();
//         for child in &children {
//             let value = MinimaxSolver::solve(child, max_depth, &mut children_memory);
//             values.push(value);
//         }

//         info!("Values: {:?}", values);

//         if position.turn {
//             if let Some(best_position_index) = values
//                 .iter()
//                 .enumerate()
//                 .max_by_key(|(_, &value)| value)
//                 .map(|(index, _)| index)
//             {
//                 info!("Best move: {:?}", valid_moves[best_position_index]);
//                 position = children[best_position_index];
//                 if position.is_game_over() {
//                     return;
//                 }
//             } else {
//                 return;
//             }
//         } else {
//             if let Some(best_position_index) = values
//                 .iter()
//                 .enumerate()
//                 .min_by_key(|(_, &value)| value)
//                 .map(|(index, _)| index)
//             {
//                 info!("Best move: {:?}", valid_moves[best_position_index]);
//                 position = children[best_position_index];
//                 if position.is_game_over() {
//                     return;
//                 }
//             } else {
//                 return;
//             }
//         }
//     }
// }
