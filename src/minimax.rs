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
    fn get_children<'a>(&self, children_memory: &'a mut Vec<T>) -> &'a Vec<T>;
    fn is_game_over(&self) -> bool;
    fn heuristic(&self) -> i32;
    fn is_maximising_player(&self) -> bool;
}

pub struct MinimaxSolver;

impl MinimaxSolver {
    pub fn minimax<T: GameState<T>>(
        game_state: &T,
        depth: u32,
        children_memory: &mut [Vec<T>],
    ) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_memory, future_memory) = children_memory.split_at_mut(1);

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_memory[0]) {
                value = max(value, Self::minimax(child, depth - 1, future_memory));
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children(&mut current_memory[0]) {
                value = min(value, Self::minimax(child, depth - 1, future_memory));
            }
        }

        value
    }

    pub fn alphabeta<T: GameState<T>>(
        game_state: &T,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        children_memory: &mut [Vec<T>],
    ) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_memory, future_memory) = children_memory.split_at_mut(1);

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_memory[0]) {
                value = max(
                    value,
                    Self::alphabeta(child, depth - 1, alpha, beta, future_memory),
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
            for child in game_state.get_children(&mut current_memory[0]) {
                value = min(
                    value,
                    Self::alphabeta(child, depth - 1, alpha, beta, future_memory),
                );
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
    }

    pub fn mtdf_no_memory<T: GameState<T>>(
        game_state: &T,
        mut guess: i32,
        depth: u32,
        children_memory: &mut [Vec<T>],
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
            guess = Self::alphabeta(game_state, depth, beta - 1, beta, children_memory);
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    pub fn alphabeta_with_memory<T: GameState<T>>(
        transposition_table: &mut TranspositionTable<T>,
        game_state: &T,
        max_depth: u32,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        children_memory: &mut [Vec<T>],
    ) -> i32 {
        if max_depth - depth <= transposition_table.max_depth {
            // if we have previously explored this node
            if let Some(lookup_result) = transposition_table.lookup(game_state) {
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

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_memory, future_memory) = children_memory.split_at_mut(1);

        let mut value;

        if depth == 0 || game_state.is_game_over() {
            value = game_state.heuristic();
        }
        // if player 1
        else if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_memory[0]) {
                value = max(
                    value,
                    Self::alphabeta_with_memory(
                        transposition_table,
                        child,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                        future_memory,
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
            for child in game_state.get_children(&mut current_memory[0]) {
                value = min(
                    value,
                    Self::alphabeta_with_memory(
                        transposition_table,
                        child,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                        future_memory,
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
            let transposition_table_element = transposition_table.get(game_state);
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
        children_memory: &mut [Vec<T>],
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
                children_memory,
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
        children_memory: &mut [Vec<T>],
    ) -> i32 {
        let mut transposition_table =
            TranspositionTable::with_capacity(max_table_capacity, max_table_depth);
        let mut guess = 0;
        let mut depth = max_depth % 2;
        while depth <= max_depth {
            // start timer
            let now = SystemTime::now();
            // run analysis
            guess = Self::mtdf_with_memory(
                &mut transposition_table,
                start_game_state,
                guess,
                depth,
                children_memory,
            );
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
            depth += 1;
        }

        guess
    }
}

pub struct TranspositionTable<T: GameState<T>> {
    data: HashMap<T, TranspositionTableElement>,
    max_depth: u32,
}

impl<T: GameState<T>> TranspositionTable<T> {
    fn new(max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::new(),
            max_depth,
        }
    }

    fn with_capacity(capacity: usize, max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::with_capacity(capacity),
            max_depth,
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
