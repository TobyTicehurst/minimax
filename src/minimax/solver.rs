#![allow(dead_code)]

use crate::minimax::{EndgamesTable, GameState, TranspositionTable};
use std::cmp::max;
use std::cmp::min;
use std::rc::Rc;

pub struct Solver<'a, T: GameState<T>, E: EndgamesTable<T>> {
    start_game_state: T,
    pub max_depth: u32,
    pub transposition_table: TranspositionTable<T>,
    pub endgames_table: &'a Rc<E>,
}

impl<'a, T: GameState<T>, E: EndgamesTable<T>> Solver<'a, T, E> {
    pub fn new(
        start_game_state: T,
        max_depth: u32,
        max_table_depth: u32,
        transposition_table_capacity: usize,
        endgames_table: &'a Rc<E>,
    ) -> Self {
        Solver {
            start_game_state: start_game_state.clone(),
            max_depth,
            transposition_table: TranspositionTable::with_capacity(
                transposition_table_capacity,
                max_table_depth,
            ),
            endgames_table,
        }
    }

    pub fn minimax(game_state: &T, depth: u32, game_state_cache: &mut [Vec<T>]) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_cache, remaining_cache) = game_state_cache.split_at_mut(1);

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_cache[0]) {
                value = max(value, Self::minimax(child, depth - 1, remaining_cache));
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children(&mut current_cache[0]) {
                value = min(value, Self::minimax(child, depth - 1, remaining_cache));
            }
        }

        value
    }

    // can no longer use the cache optimisation as we don't know the maximum depth
    pub fn alphabeta_no_depth_limit(
        game_state: &T,
        mut alpha: i32,
        mut beta: i32,
        endgames_table: &E,
    ) -> i32 {
        if game_state.is_game_over() {
            return game_state.heuristic();
        }

        // see if we have been here before
        if let Some(eval) = endgames_table.lookup(game_state) {
            return eval;
        }

        // create memory for children
        let mut current_cache = game_state.get_children_cache();

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_cache) {
                value = max(
                    value,
                    Self::alphabeta_no_depth_limit(child, alpha, beta, endgames_table),
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
            for child in game_state.get_children(&mut current_cache) {
                value = min(
                    value,
                    Self::alphabeta_no_depth_limit(child, alpha, beta, endgames_table),
                );
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
    }

    // alpha-beta pruning is an optimisation to the minimax algorithm
    // alpha is the minimum value that the maximising player (player 1) is assured to get
    // beta is the maximum value that the minimising player (player 2) is assured to get
    // if we ever find that player 2 can make a move which results in an evaluation less than alpha then we can immediately discard this branch and return
    // vice-versa for player 1 and beta
    pub fn alphabeta(
        game_state: &T,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_cache, remaining_cache) = game_state_cache.split_at_mut(1);

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_cache[0]) {
                value = max(
                    value,
                    Self::alphabeta(child, depth - 1, alpha, beta, remaining_cache),
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
            for child in game_state.get_children(&mut current_cache[0]) {
                value = min(
                    value,
                    Self::alphabeta(child, depth - 1, alpha, beta, remaining_cache),
                );
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
    }

    // calls alphabeta with a "zero-window" - alpha is one less than beta
    // if the result comes back as alpha, we know the true result is alpha or less
    // if the result comes back as beta, we know the true result is beta or greater
    // by calling mtdf we can set either an upper or lower bound on the true result
    // calling mtdf multiple times we can narrow down the true result to a specific value
    // NOTE: mtdf only gives major speed-up when using a transposition table and iterative deepening
    pub fn mtdf_no_memory(
        &self,
        mut guess: i32,
        depth: u32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            guess = Self::alphabeta(
                &self.start_game_state.clone(),
                depth,
                beta - 1,
                beta,
                game_state_cache,
            );
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    // alpha-beta pruning optimisation with a transposition table to store previously explored game states
    pub fn alphabeta_with_memory(
        &mut self,
        game_state: &T,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        let mut value;

        // search endgame table first as it has more accurate results
        if let Some(eval) = self.endgames_table.lookup(game_state) {
            value = eval;
            alpha = eval;
            beta = eval;
        } else {
            // opening table
            if self.max_depth - depth <= self.transposition_table.max_depth {
                // if we have previously explored this node
                if let Some(lookup_result) = self.transposition_table.lookup(game_state) {
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
            let (current_cache, future_cache) = game_state_cache.split_at_mut(1);

            if depth == 0 || game_state.is_game_over() {
                value = game_state.heuristic();
            }
            // if player 1
            else if game_state.is_maximising_player() {
                let mut a = alpha;
                value = i32::MIN;
                for child in game_state.get_children(&mut current_cache[0]) {
                    value = max(
                        value,
                        self.alphabeta_with_memory(child, depth - 1, a, beta, future_cache),
                    );
                    if value >= beta {
                        break;
                    }
                    a = max(a, value);
                }
            }
            // if player 2
            else {
                let mut b = beta;
                value = i32::MAX;
                for child in game_state.get_children(&mut current_cache[0]) {
                    value = min(
                        value,
                        self.alphabeta_with_memory(child, depth - 1, alpha, b, future_cache),
                    );
                    if value <= alpha {
                        break;
                    }
                    b = min(b, value);
                }
            }
        }

        if self.max_depth - depth <= self.transposition_table.max_depth {
            // store this in the transposition table
            let transposition_table_element = self.transposition_table.get(game_state);
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

    pub fn mtdf_with_memory(
        &mut self,
        mut guess: i32,
        depth: u32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            guess = self.alphabeta_with_memory(
                &self.start_game_state.clone(),
                depth,
                beta - 1,
                beta,
                game_state_cache,
            );
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    // alpha-beta pruning optimisation with a transposition table to store previously explored game states
    pub fn alphabeta_with_memory_no_depth_limit(
        &mut self,
        game_state: &T,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        let mut value;

        // search endgame table first as it has more accurate results
        if let Some(eval) = self.endgames_table.lookup(game_state) {
            value = eval;
            alpha = eval;
            beta = eval;
        } else {
            // split off the data needed (see README for an explanation of this optimisation)
            let (current_cache, future_cache) = game_state_cache.split_at_mut(1);

            if game_state.is_game_over() {
                value = game_state.heuristic();
            }
            // if player 1
            else if game_state.is_maximising_player() {
                let mut a = alpha;
                value = i32::MIN;
                for child in game_state.get_children(&mut current_cache[0]) {
                    value = max(
                        value,
                        self.alphabeta_with_memory_no_depth_limit(
                            child,
                            depth + 1,
                            a,
                            beta,
                            future_cache,
                        ),
                    );
                    if value >= beta {
                        break;
                    }
                    a = max(a, value);
                }
            }
            // if player 2
            else {
                let mut b = beta;
                value = i32::MAX;
                for child in game_state.get_children(&mut current_cache[0]) {
                    value = min(
                        value,
                        self.alphabeta_with_memory_no_depth_limit(
                            child,
                            depth + 1,
                            alpha,
                            b,
                            future_cache,
                        ),
                    );
                    if value <= alpha {
                        break;
                    }
                    b = min(b, value);
                }
            }
        }

        if depth <= self.transposition_table.max_depth {
            // store this in the transposition table
            let transposition_table_element = self.transposition_table.get(game_state);
            // fail low: we have a new upper bound
            if value <= alpha {
                transposition_table_element.upper_bound.bound = value;
                transposition_table_element.upper_bound.depth = u32::MAX;
            }
            // accurate value for alpha (won't happen in a zero window)
            if value > alpha && value < beta {
                transposition_table_element.lower_bound.bound = value;
                transposition_table_element.lower_bound.depth = u32::MAX;
                transposition_table_element.upper_bound.bound = value;
                transposition_table_element.upper_bound.depth = u32::MAX;
            }
            // fail high: we have a new lower bound
            if value >= beta {
                transposition_table_element.lower_bound.bound = value;
                transposition_table_element.lower_bound.depth = u32::MAX;
            }
        }

        value
    }

    pub fn mtdf_with_memory_no_depth_limit(
        &mut self,
        mut guess: i32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            guess = self.alphabeta_with_memory_no_depth_limit(
                &self.start_game_state.clone(),
                0,
                beta - 1,
                beta,
                game_state_cache,
            );
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }

    pub fn test(
        &mut self,
        game_state: &T,
        mut alpha: i32,
        mut beta: i32,
        game_state_cache: &mut [Vec<T>],
    ) -> i32 {
        // search endgame table first as it has more accurate results
        if let Some(eval) = self.endgames_table.lookup(game_state) {
            return eval;
        }
        if game_state.is_game_over() {
            return game_state.heuristic();
        }

        // split off the data needed (see README for an explanation of this optimisation)
        let (current_cache, future_cache) = game_state_cache.split_at_mut(1);

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children(&mut current_cache[0]) {
                value = max(value, self.test(child, alpha, beta, future_cache));
                if value >= beta {
                    break;
                }
                alpha = max(alpha, value);
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children(&mut current_cache[0]) {
                value = min(value, self.test(child, alpha, beta, future_cache));
                if value <= alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
    }

    pub fn mtdf_test(&mut self, mut guess: i32, game_state_cache: &mut [Vec<T>]) -> i32 {
        let mut beta: i32;
        let mut lower_bound = i32::MIN;
        let mut upper_bound = i32::MAX;

        while lower_bound < upper_bound {
            beta = max(guess, lower_bound + 1);
            println!("test");
            guess = self.test(
                &self.start_game_state.clone(),
                beta - 1,
                beta,
                game_state_cache,
            );
            if guess < beta {
                upper_bound = guess;
            } else {
                lower_bound = guess;
            }
        }

        guess
    }
}
