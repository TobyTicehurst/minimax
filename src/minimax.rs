use std::cmp::max;
use std::cmp::min;

// GameState should encode all the specifics of a game and none of the specifics of the solver algorithm
pub trait GameState<T> {
    fn get_children(&self) -> Vec<T>;
    fn is_game_over(&self) -> bool;
    fn heuristic(&self) -> i32;
    fn is_maximising_player(&self) -> bool;
}

pub struct MinimaxSolver<T: GameState<T>> {
    pub start_game_state: T,
}

impl<T: GameState<T> + std::fmt::Debug> MinimaxSolver<T> {
    pub fn new(start_game_state: T) -> Self {
        MinimaxSolver { start_game_state }
    }

    pub fn solve(&self, depth: u32) -> i32 {
        //Self::minimax(&self.start_game_state, depth)
        Self::alphabeta(&self.start_game_state, depth, i32::MIN, i32::MAX)
    }

    fn minimax(game_state: &T, depth: u32) -> i32 {
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

    fn alphabeta(game_state: &T, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
        if depth == 0 || game_state.is_game_over() {
            return game_state.heuristic();
        }

        let mut value;

        // if player 1
        if game_state.is_maximising_player() {
            value = i32::MIN;
            for child in game_state.get_children() {
                value = max(value, Self::alphabeta(&child, depth - 1, alpha, beta));
                if value > beta {
                    break;
                }
                alpha = max(alpha, value);
            }
        }
        // if player 2
        else {
            value = i32::MAX;
            for child in game_state.get_children() {
                value = min(value, Self::alphabeta(&child, depth - 1, alpha, beta));
                if value < alpha {
                    break;
                }
                beta = min(beta, value);
            }
        }

        value
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
