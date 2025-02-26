#![allow(dead_code)]

mod interactive;
mod mancala;
mod minimax;
//mod endgames;

use mancala::{MancalaEndgamesTable, MancalaGameState};
use minimax::{EndgamesTable, GameState, Solver};
use std::cmp::max;
use std::rc::Rc;
use std::time::SystemTime;
use tracing::info;
use tracing::level_filters::LevelFilter;

enum SolverAlgorithm {
    Minimax,
    Alphabeta,
    MtdfNoMemory,
    MtdfMemory,
}

fn analyse(
    solver: SolverAlgorithm,
    max_depth: u32,
    max_table_depth: u32,
    transposition_table_capacity: usize,
    endgames_table_max_stones: u32,
) {
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
    let mut game_state_cache = MancalaGameState::generate_children_memory(max_depth);
    let mut guess = 0;
    for depth in 0..(max_depth + 1) {
        // start timer
        let now = SystemTime::now();
        guess = match solver {
            SolverAlgorithm::Minimax => {
                minimax_solver.minimax(&MancalaGameState::default(), depth, &mut game_state_cache)
            }
            SolverAlgorithm::Alphabeta => minimax_solver.alphabeta(
                &MancalaGameState::default(),
                depth,
                50,
                i32::MAX,
                &mut game_state_cache,
            ),
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

    // // final - 1
    // let now = SystemTime::now();
    // let mut game_state_cache = MancalaGameState::generate_children_memory(10000);
    // let eval = minimax_solver.mtdf_test(guess, &mut game_state_cache);

    // let time = now.elapsed().unwrap();
    // info!(
    //     "Value: {}, Depth: {}, Time: {}ms",
    //     eval,
    //     10000,
    //     time.as_millis()
    // );

    // final - 2
    let now = SystemTime::now();
    let mut game_state_cache = MancalaGameState::generate_children_memory(10000);
    let eval = minimax_solver.mtdf_with_memory(guess, 10000, &mut game_state_cache);

    let time = now.elapsed().unwrap();
    info!(
        "Value: {}, Depth: {}, Time: {}ms",
        eval,
        10000,
        time.as_millis()
    );
}

fn depth_analysis(game_state: &MancalaGameState, depth: u32, max_depth: u32) -> u32 {
    //info!("{:?}", game_state);
    if game_state.is_game_over() {
        return depth;
    }

    let mut new_depth = depth;

    let mut cache: Vec<MancalaGameState> = Vec::with_capacity(6);
    for child in game_state.get_children(&mut cache) {
        new_depth = max(new_depth, depth_analysis(child, depth + 1, max_depth));
    }

    if new_depth == max_depth {
        info!("Test: {:?}", game_state.pits);
    }

    new_depth
}

fn all_positions(
    game_state: &mut MancalaGameState,
    remaining_stones: u32,
    pit_index: usize,
    mut depth: u32,
    max_depth: u32,
) -> u32 {
    if pit_index == 12 {
        game_state.pits[12] = remaining_stones;
        let new_depth = depth_analysis(&game_state, 0, max_depth);
        if new_depth == max_depth {
            info!("{:?}", game_state);
        }
        depth = max(depth, new_depth);
    } else if pit_index == 7 {
        depth = all_positions(
            game_state,
            remaining_stones,
            pit_index + 1,
            depth,
            max_depth,
        );
    } else {
        for i in 0..(remaining_stones + 1) {
            game_state.pits[pit_index] = i;
            depth = max(
                depth,
                all_positions(
                    game_state,
                    remaining_stones - i,
                    pit_index + 1,
                    depth,
                    max_depth,
                ),
            );
        }
    }

    depth
}

fn deepest(max_stones: u32) {
    let depth = all_positions(&mut MancalaGameState::new(), max_stones, 0, 0, 10000);
    info!("Depth: {:?}", depth);
    let _ = all_positions(&mut MancalaGameState::new(), max_stones, 0, 0, depth);
}

#[derive(Debug)]
enum MancalaMove {
    A,
    B,
    C,
    D,
    E,
    F,
}

use MancalaMove::{A, B, C, D, E, F};

impl MancalaMove {
    pub fn from_string(str: &str) -> Option<Self> {
        match str.to_ascii_uppercase().as_str() {
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            "E" => Some(Self::E),
            "F" => Some(Self::F),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            MancalaMove::A => "A",
            MancalaMove::B => "B",
            MancalaMove::C => "C",
            MancalaMove::D => "D",
            MancalaMove::E => "E",
            MancalaMove::F => "F",
        }
    }

    pub fn from_index(index: &usize) -> Option<Self> {
        match index % (MancalaGameState::PITS_PER_SIDE + 1) {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            4 => Some(Self::E),
            5 => Some(Self::F),
            _ => None,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            MancalaMove::A => 0,
            MancalaMove::B => 1,
            MancalaMove::C => 2,
            MancalaMove::D => 3,
            MancalaMove::E => 4,
            MancalaMove::F => 5,
        }
    }
}

fn make_all_moves(
    start_game_state: MancalaGameState,
    moves: &Vec<MancalaMove>,
) -> MancalaGameState {
    let mut game_state = start_game_state.clone();
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

fn full_solve(
    start_game_state: MancalaGameState,
    endgames_table: &Rc<MancalaEndgamesTable>,
) -> (usize, i32) {
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
    let mut game_state_cache = MancalaGameState::generate_children_memory(10000);

    let mut guess = 0;
    let mut depth = 0;
    while depth <= max_depth {
        guess = minimax_solver.mtdf_with_memory(guess, depth, &mut game_state_cache);
        depth += 2;
    }
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

fn testing() {
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
        game_state = make_all_moves(MancalaGameState::default(), &moves);
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
            let (move_index, eval) = full_solve(game_state, &endgames_table_rc);
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

fn endgames_to_file(num_stones: u32) {
    let filepath = "endgames.bin";
    let mut endgames_table = MancalaEndgamesTable::new(num_stones);
    endgames_table.calculate_endgames();
    endgames_table.write_to_file(filepath);
}

fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_ansi(false)
        .without_time()
        .init();

    //MancalaEndgamesTable::test_table_validity(3);
    //MancalaEndgamesTable::test_table_accuracy(5);
    //endgames_to_file(30);
    testing();

    // let test_state = MancalaGameState {
    //     //pits: [1, 0, 1, 11, 9, 0, 14, 0, 2, 1, 0, 0, 0, 9],
    //     //pits: [1, 0, 0, 11, 9, 0, 14, 0, 0, 2, 1, 0, 0, 10],
    //     pits:   [1, 0, 0, 11, 9, 0, 14, 0, 2, 0, 1, 0, 0, 10],
    //     turn: true,
    //     game_over: false,
    // };

    // let endgames_table_file = MancalaEndgamesTable::read_from_file("endgames.bin");
    // let lookup = endgames_table_file.lookup(&test_state);
    // println!("lookup: {:?}", lookup);

    // let mut endgames_table = MancalaEndgamesTable::new(0);
    // endgames_table.calculate_endgames();
    //let rc_endgames_table = Rc::new(endgames_table);

    // let mut minimax_solver = Solver::new(
    //     test_state,
    //     10000,
    //     10,
    //     1000000,
    //     &rc_endgames_table,
    // );
    // let mut game_state_cache = MancalaGameState::generate_children_memory(10000);

    // let eval = Solver::alphabeta_no_depth_limit(&test_state, i32::MIN, i32::MAX, &endgames_table);

    // println!("eval: {}", eval);
    //analyse(SolverAlgorithm::MtdfMemory, 28, 20, 10000000, 2);
    //interactive::play_game();

    // MancalaEndgamesTable::test_index_validity(8);
    // MancalaEndgamesTable::test_table_validity(4);
    // let mut endgames_table = MancalaEndgamesTable::new(10);
    // endgames_table.calculate_endgames();

    // test_state.turn = false;
    //test_state.pits = [0, 0, 0, 0, 0, 1, 46, 0, 0, 0, 0, 0, 1, 0];
    // let eval = endgames_table.lookup(&test_state);
    // println!("{:?}", eval);

    // let mut game_state = MancalaGameState::new();
    // game_state.pits[0] = 1;
    // game_state.pits[7] = 1;
    // depth_analysis(&game_state, 0);

    // info!("Alphabeta");
    // analyse(Solver::Alphabeta, max_depth);
    // info!("Mtdf");
    //analyse(SolverAlgorithm::Alphabeta, max_depth, 16, 10000000);
    //info!("Iterative Deepening");
    //iterative_deepening(max_depth, 16, 10000000);
    //interactive::play_game(max_depth);

    // for i in 0..10 {
    //     deepest(i);
    // }

    // let mut game_state = MancalaGameState {
    //     pits: [0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
    //     turn: true,
    //     game_over: false
    // };
    // depth_analysis(&mut game_state, 0);
}
