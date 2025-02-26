#![allow(dead_code)]

use crate::mancala::{MancalaEndgamesTable, MancalaGameState, MancalaMove};
use crate::minimax::{EndgamesTable, GameState, Solver};
use std::rc::Rc;
use std::time::SystemTime;
use tracing::info;

// from a the given starting game state, make each of the given moves and return the resulting game state
fn make_moves(start_game_state: MancalaGameState, moves: &Vec<MancalaMove>) -> MancalaGameState {
    let mut game_state = start_game_state;
    for mancala_move in moves {
        let mut move_index = mancala_move.to_index();
        let players_store;
        let opponents_store;
        if game_state.turn {
            players_store = MancalaGameState::PLAYER_1_STORE;
            opponents_store = MancalaGameState::PLAYER_2_STORE;
        } else {
            move_index += MancalaGameState::PLAYER_1_STORE + 1;
            players_store = MancalaGameState::PLAYER_2_STORE;
            opponents_store = MancalaGameState::PLAYER_1_STORE;
        }

        if game_state.pits[move_index] == 0 {
            println!("Fatal Error. move_index: {}", move_index);
        }

        game_state.make_move(move_index, players_store, opponents_store);
    }

    game_state
}

// strongly solve the given position
// See: https://en.wikipedia.org/wiki/Solved_game
// `endgames_table` can be read using `read_endgames_from_file()` or calculated with `calculate_endgames()`
// it is recommend to have an endgames table out to at least 24 stones (if solving Kalah(6, 4))
fn strongly_solve(
    start_game_state: MancalaGameState,
    endgames_table: &Rc<MancalaEndgamesTable>,
) -> (usize, i32) {
    // the parameters used here are magic numbers
    let max_depth = 28;
    let max_table_depth = 20;
    let transposition_table_capacity = 10000000;

    let mut minimax_solver = Solver::new(
        start_game_state,
        max_depth,
        max_table_depth,
        transposition_table_capacity,
        endgames_table,
    );

    // this assumes that a depth of 10000 will never be reached
    // this can be assumed for Kalah(6, 4) since a stone must be scored every 6 moves at a minimum (48 * 6 = 288)
    // this isn't generally true though and it would be better to have a dynamically growing vector which is doubled when memory runs out
    let mut game_state_cache = MancalaGameState::generate_children_memory(10000);

    let mut guess = 0;
    let mut depth = 0;
    // firstly, use iterative deepening to find a good estimate (mtdf is faster with a good guess)
    while depth <= max_depth {
        guess = minimax_solver.mtdf_with_memory(guess, depth, &mut game_state_cache);
        depth += 2;
    }
    // secondly, fully explore the decision tree
    let eval = minimax_solver.mtdf_with_memory_no_depth_limit(guess, &mut game_state_cache);

    // get move
    let children = start_game_state.get_children_naive();
    let valid_moves = start_game_state.get_valid_moves();

    for (child, child_move) in children.iter().zip(valid_moves) {
        if let Some(lookup) = endgames_table.lookup(child) {
            if lookup == eval {
                return (child_move, eval);
            }
        } else if let Some(lookup) = minimax_solver.transposition_table.lookup(child) {
            if lookup.lower_bound.bound == lookup.upper_bound.bound
                && lookup.lower_bound.bound == eval
            {
                return (child_move, eval);
            }
        }
    }

    println!("Move not found");

    (404, eval)
}

pub enum SolverAlgorithm {
    Minimax,
    Alphabeta,
    MtdfNoMemory,
    MtdfMemory,
}

// similar to the `strongly_solve()` function except only analyse to a given depth
pub fn analyse_position(
    start_game_state: MancalaGameState,
    solver: SolverAlgorithm,
    max_depth: u32,
    endgames_table: &Rc<MancalaEndgamesTable>,
) {
    let max_table_depth = 20;
    let transposition_table_capacity = 10000000;

    let mut minimax_solver = Solver::new(
        start_game_state,
        max_depth,
        max_table_depth,
        transposition_table_capacity,
        endgames_table,
    );
    let mut game_state_cache = MancalaGameState::generate_children_memory(max_depth);
    let mut guess = 0;
    for depth in 0..(max_depth + 1) {
        // start timer
        let now = SystemTime::now();
        guess = match solver {
            SolverAlgorithm::Minimax => Solver::<MancalaGameState, MancalaEndgamesTable>::minimax(
                &MancalaGameState::default(),
                depth,
                &mut game_state_cache,
            ),
            SolverAlgorithm::Alphabeta => {
                Solver::<MancalaGameState, MancalaEndgamesTable>::alphabeta(
                    &MancalaGameState::default(),
                    depth,
                    50,
                    i32::MAX,
                    &mut game_state_cache,
                )
            }
            SolverAlgorithm::MtdfNoMemory => {
                minimax_solver.mtdf_no_memory(guess, depth, &mut game_state_cache)
            }
            SolverAlgorithm::MtdfMemory => {
                minimax_solver.mtdf_with_memory(guess, depth, &mut game_state_cache)
            }
        };
        let time = now.elapsed().unwrap();
        info!(
            "Value: {}, Depth: {}, Time: {}ms",
            guess,
            depth,
            time.as_millis()
        );
    }
}

pub fn player_vs_mancala_bot() {
    println!("Hi there! I am Mancala Bot 🤖.");
    println!("I'm doing a big think, please wait 🤔💭...");
    // let mut endgames_table = MancalaEndgamesTable::new(26);
    // endgames_table.calculate_endgames();
    // endgames_table.write_to_file("test.bin");
    // let endgames_table_rc = Rc::new(endgames_table.clone());

    let endgames_table = MancalaEndgamesTable::read_from_file("endgames.bin");
    let endgames_table_rc = Rc::new(endgames_table);
    println!("Woah! I'm ready!");

    // get user player
    let mut user_player = true;
    loop {
        println!("Would you like to be player 1 or player 2?");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();

        let input_correct = match buffer.trim() {
            "1" => {
                println!("You chose player 1");
                user_player = true;
                true
            }
            "2" => {
                println!("You chose player 2");
                user_player = false;
                true
            }
            _ => false,
        };

        if input_correct {
            break;
        } else {
            println!(
                "Unexpected input. You entered: {}. Expected: [1, 2].",
                buffer
            );
        }
    }

    let mut game_state;
    let mut moves: Vec<MancalaMove> = Vec::new();

    loop {
        println!("Moves: {:?}", moves);

        // make all moves
        game_state = make_moves(MancalaGameState::default(), &moves);
        if game_state.is_game_over() {
            println!("Good game!");
            println!("Final game state: ");
            game_state.pretty_print();
            println!("I look forward to playing you again 👋");
            return;
        }

        // print the game state
        game_state.pretty_print();

        // if it's the player's turn
        if game_state.turn == user_player {
            loop {
                // get valid moves
                let valid_moves = game_state.get_valid_moves();
                let valid_moves_readable: Vec<String> = valid_moves
                    .iter()
                    .map(|move_index| {
                        MancalaMove::from_index(move_index)
                            .expect("Fatal Error")
                            .to_str()
                            .to_string()
                    })
                    .collect();

                println!("Your move! {:?} or undo", valid_moves_readable);
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();

                if let Some(mancala_move) = MancalaMove::from_string(buffer.trim()) {
                    if valid_moves.contains(&mancala_move.to_index()) {
                        moves.push(mancala_move);
                        break;
                    }
                }

                println!(
                    "Unexpected input. You entered: {}. Expected: {:?} or undo",
                    buffer.trim(),
                    valid_moves_readable
                );
            }
        } else {
            println!("Doing another big think 🤔... What move to make...");
            let (move_index, eval) = strongly_solve(game_state, &endgames_table_rc);
            let mancala_move = MancalaMove::from_index(&move_index).expect("Fatal Error");
            println!(
                "Making move: {}. I evaluate it as: {}",
                mancala_move.to_str(),
                eval
            );
            moves.push(mancala_move);
        }
    }
}
