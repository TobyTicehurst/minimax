#![allow(dead_code)]

use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::SystemTime;
use tracing::info;

// GameState should encode all the specifics of a game and none of the specifics of the solver algorithm
pub trait GameState<T>: PartialEq + Eq + Hash + Clone + Debug {
    fn get_children(&self) -> Vec<T>;
    fn is_game_over(&self) -> bool;
    fn heuristic(&self) -> i32;
    fn is_maximising_player(&self) -> bool;
}

pub struct MinimaxSolver;

impl MinimaxSolver {
    pub fn solve<T: GameState<T>>(start_game_state: &T, depth: u32) -> i32 {
        //Self::minimax(start_game_state, depth)
        Self::alphabeta(start_game_state, depth, i32::MIN, i32::MAX)

        //Self::mtdf(start_game_state, 1, depth)
    }

    fn minimax<T: GameState<T>>(game_state: &T, depth: u32) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children() {
                value = max(value, Self::minimax(&child, depth - 1));
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children() {
                value = min(value, Self::minimax(&child, depth - 1));
            }
        }

        value
    }

    fn alphabeta<T: GameState<T>>(
        game_state: &T,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children().into_iter().rev() {
                value = max(value, Self::alphabeta(&child, depth - 1, alpha, beta));
                if value >= beta {
                    break;
                }
                alpha = max(alpha, value);
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children().into_iter().rev() {
                value = min(value, Self::alphabeta(&child, depth - 1, alpha, beta));
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
    }

    fn mtdf_no_memory<T: GameState<T>>(game_state: &T, mut guess: i32, depth: u32) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            // info!(
            //     "Guess: {}, Beta: {}, Lower {}, Upper: {}",
            //     guess, beta, lower_bound, upper_bound
            // );
            guess = Self::alphabeta(game_state, depth, beta - 1, beta);
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    fn alphabeta_with_memory<T: GameState<T>>(
        transposition_table: &mut TranspositionTable<T>,
        game_state: &T,
        max_depth: u32,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> i32 {
        if max_depth - depth <= transposition_table.max_depth {
            // if we have previously explored this node
            if let Some(lookup_result) = transposition_table.lookup(&game_state) {
                // these values are only valid if we reached this position at an equal or deeper depth
                if lookup_result.lower_bound.depth >= depth {
                    if lookup_result.lower_bound.bound >= beta {
                        return lookup_result.lower_bound.bound;
                    }
                    alpha = max(alpha, lookup_result.lower_bound.bound);
                }

                if lookup_result.upper_bound.depth >= depth {
                    if lookup_result.upper_bound.bound <= alpha {
                        return lookup_result.upper_bound.bound;
                    }
                    beta = min(beta, lookup_result.upper_bound.bound)
                }
            }
        }

        let mut value;

        if depth == 0 || game_state.is_game_over() {
            value = game_state.heuristic();
        }
        // if player 1
        else if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children().into_iter().rev() {
                value = max(
                    value,
                    Self::alphabeta_with_memory(
                        transposition_table,
                        &child,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                    ),
                );
                if value >= beta {
                    break;
                }
                alpha = max(alpha, value);
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children().into_iter().rev() {
                value = min(
                    value,
                    Self::alphabeta_with_memory(
                        transposition_table,
                        &child,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                    ),
                );
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        if max_depth - depth <= transposition_table.max_depth {
            // store this in the transposition table
            let transposition_table_element = transposition_table.get(&game_state);
            // fail low: we have a new upper bound
            if value <= alpha {
                transposition_table_element.upper_bound.bound = value;
                transposition_table_element.upper_bound.depth = depth;
            }
            // accurate value for alpha (won't happen in a zero window)
            if value > alpha && value < beta {
                transposition_table_element.lower_bound.bound = value;
                transposition_table_element.lower_bound.depth = depth;
                transposition_table_element.upper_bound.bound = value;
                transposition_table_element.upper_bound.depth = depth;
            }
            // fail high: we have a new lower bound
            if value >= beta {
                transposition_table_element.lower_bound.bound = value;
                transposition_table_element.lower_bound.depth = depth;
            }
        }

        value
    }

    fn mtdf_with_memory<T: GameState<T>>(
        transposition_table: &mut TranspositionTable<T>,
        game_state: &T,
        mut guess: i32,
        depth: u32,
    ) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            // info!(
            //     "Guess: {}, Beta: {}, Lower {}, Upper: {}",
            //     guess, beta, lower_bound, upper_bound
            // );
            guess = Self::alphabeta_with_memory(
                transposition_table,
                game_state,
                depth,
                depth,
                beta - 1,
                beta,
            );
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    pub fn mtdf_with_memory_iterative_deepening<T: GameState<T>>(
        start_game_state: &T,
        max_depth: u32,
        max_table_depth: u32,
        max_table_capacity: usize,
    ) -> i32 {
        let mut transposition_table =
            TranspositionTable::with_capacity(max_table_capacity, max_table_depth);
        let mut guess = 0;
        let mut depth = max_depth % 2;
        while depth <= max_depth {
            // start timer
            let now = SystemTime::now();
            // run analysis
            guess =
                Self::mtdf_with_memory(&mut transposition_table, start_game_state, guess, depth);
            // stop timer
            let time = now.elapsed().unwrap();
            // print results
            info!(
                "Value: {}, Depth: {}, Time: {}ms",
                guess,
                depth,
                time.as_millis()
            );
            // info!("size: {}", transposition_table.data.capacity());
            // for elem in transposition_table.data.iter() {
            //     info!("{:?}, {:?}", elem.0, elem.1);
            // }
            depth += 2;
        }

        guess
    }
}

struct TranspositionTable<T: GameState<T>> {
    data: HashMap<T, TranspositionTableElement>,
    max_depth: u32,
}

impl<T: GameState<T>> TranspositionTable<T> {
    fn new(max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::new(),
            max_depth: max_depth,
        }
    }

    fn with_capacity(capacity: usize, max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::with_capacity(capacity),
            max_depth: max_depth,
        }
    }

    fn lookup(&self, game_state: &T) -> Option<&TranspositionTableElement> {
        self.data.get(game_state)
    }

    fn get(&mut self, game_state: &T) -> &mut TranspositionTableElement {
        // TODO avoid cloning here if possible
        self.data
            .entry(game_state.clone())
            .or_insert(TranspositionTableElement::default())
    }
}

#[derive(Debug)]
struct TranspositionTableElement {
    lower_bound: Bound,
    upper_bound: Bound,
}

#[derive(Debug)]
struct Bound {
    bound: i32,
    depth: u32,
}

impl TranspositionTableElement {
    fn default() -> Self {
        TranspositionTableElement {
            lower_bound: Bound {
                bound: i32::MIN,
                depth: 0,
            },
            upper_bound: Bound {
                bound: i32::MAX,
                depth: 0,
            },
        }
    }
}

// pub trait GameState {
//     type Move: Copy;

//     fn new() -> Self where Self: Sized;

//     fn get_move_order(&self) -> [Self::Move; 6];

//     fn make_move(&self, next_state: &mut Self, player_move: Self::Move);

//     fn heuristic(&self) -> i32;

//     fn get_turn(&self) -> Player;

//     fn is_game_over(&self) -> bool;

//     fn is_move_valid(&self, player_move: Self::Move) -> bool;

//     fn print(&self);
// }

// pub struct Solver<'a, T: GameState> {
//     pub start_game_state: &'a T,
//     pub depth: u32,
// }

// impl<'a, T: GameState> Solver<'a, T> {
//     pub fn solve(&mut self) {
//         println!("Depth: {}", self.depth);
//         let now = Instant::now();

//         let evaluation: i32;
//         if self.depth == 0 || self.start_game_state.is_game_over() {
//             evaluation = self.start_game_state.heuristic()
//         }
//         else {
//             //evaluation = self.minimax(self.start_game_state, self.depth - 1, -1 * INF, INF);
//             evaluation = self.mtdf(self.start_game_state, 0);
//         }

//         let time = now.elapsed().as_millis();
//         println!("Evaluation: {}", evaluation);
//         println!("{}ms\n", time);
//     }

//     pub fn mtdf(
//         &mut self,
//         game_state: &T,
//         mut guess: i32) -> i32 {

//         let mut depth: u32 = 2;

//         while depth <= self.depth {
//             let now = Instant::now();

//             let mut beta: i32;
//             let mut lower_bound: i32 = -1 * INF;
//             let mut upper_bound: i32 = INF;

//             while lower_bound < upper_bound {
//                 beta = max(guess, lower_bound + 1);
//                 guess = self.minimax(game_state, depth - 1, beta - 1, beta);
//                 if guess < beta {
//                     upper_bound = guess;
//                 }
//                 else {
//                     lower_bound = guess;
//                 }
//             }

//             let time = now.elapsed().as_millis();
//             println!("Evaluation: {}, depth: {}, time: {}ms", guess, depth, time);
//             depth += 2;
//         }

//         return guess;
//     }

//     pub fn minimax(
//         &mut self,
//         game_state: &T,
//         depth: u32,
//         mut alpha: i32,
//         mut beta: i32) -> i32 {

//         // TODO return on first call if depth is 0 or game over

//         //game_state.print();

//         // get new game state (hopefully this gets inlined so the struct never gets copied)
//         let mut child_game_state = T::new();

//         match game_state.get_turn() {
//             Player::One => {
//                 let mut value = -1 * INF;

//                 // TODO return a vec as some moves wont be valid, or check if a move is valid
//                 let move_order = game_state.get_move_order();

//                 for move_index in move_order {
//                     if game_state.is_move_valid(move_index) {
//                         // make each move and store each result in child_game_state
//                         game_state.make_move(&mut child_game_state, move_index);
//                         let evaluation: i32;
//                         // recursively evaluate the child game state
//                         if depth == 0 || child_game_state.is_game_over() {
//                             evaluation = child_game_state.heuristic();
//                         }
//                         else {
//                             evaluation = self.minimax(&child_game_state, depth - 1, alpha, beta);
//                             // if not too deep (pass a context variable) (could test outside of for loop)
//                             //     child_opening_index = context.openingTree.get_child(opening_index, move)
//                             // evaluation = minimax(context, &child_game_state, depth - 1, alpha, beta, child_opening_index);
//                         }

//                         value = max(evaluation, value);

//                         alpha = max(alpha, value);

//                         // openingTree.update(index, move, value, alpha, beta, depth)

//                         if value >= beta {
//                             break;
//                         }
//                     }
//                 }

//                 return value;
//             }
//             Player::Two => {
//                 let mut value = INF;

//                 let move_order = game_state.get_move_order();
//                 for move_index in move_order {
//                     if game_state.is_move_valid(move_index) {
//                         // make each move and store each result in child_game_state
//                         game_state.make_move(&mut child_game_state, move_index);
//                         let evaluation: i32;
//                         // recursively evaluate the child game state
//                         if depth == 0 || child_game_state.is_game_over() {
//                             evaluation = child_game_state.heuristic();
//                         }
//                         else {
//                             evaluation = self.minimax(&child_game_state, depth - 1, alpha, beta);
//                         }
//                         value = min(evaluation, value);

//                         beta = min(beta, value);

//                         if value <= alpha {
//                             break;
//                         }
//                     }
//                 }

//                 return value;
//             }
//         }
//     }
// }
