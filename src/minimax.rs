use std::cmp::max;
use std::cmp::min;
use std::time::Instant;

pub const INF: i32 = 100;

#[derive(Debug, Clone, Copy)]
pub enum Player {
    One,
    Two,
}

pub trait GameState {
    fn new() -> Self where Self: Sized;

    fn get_move_order(&self) -> Vec<usize>;

    fn make_move(&self, next_state: &mut Self, move_index: usize);

    fn heuristic(&self) -> i32;

    fn get_turn(&self) -> Player;

    fn is_game_over(&self) -> bool;

    fn is_move_valid(&self, move_index: usize) -> bool;

    fn print(&self);

    //fn test() -> &mut dyn GameState;
}

// impl<'a, T> GameState for &'a T where T: GameState {}
// impl<'a, T> GameState for &'a mut T where T: GameState {}

// enum SolveMethod {
//     Minimax,
// }

pub struct Solver<'a, T: GameState> {
    pub start_game_state: &'a T,
    pub depth: u32,
}

impl<'a, T: GameState> Solver<'a, T> {
    pub fn solve(&self) {
        println!("Depth: {}", self.depth);
        let now = Instant::now();
        let evaluation = minimax(self.start_game_state, self.depth, -1 * INF, INF);
        let time = now.elapsed().as_millis();
        println!("Evaluation: {}", evaluation);
        println!("{}ms\n", time);
    }
}

pub fn minimax<T: GameState>(
    game_state: &T, 
    depth: u32, 
    mut alpha: i32, 
    mut beta: i32) -> i32 {

    //game_state.print();

    if depth == 0 || game_state.is_game_over() {
        return game_state.heuristic();
    }

    // get new game state (hopefully this gets inlined so the struct never gets copied)
    let mut child_game_state = T::new();

    match game_state.get_turn() {
        Player::One => {
            let mut value = -1 * INF;
            
            // TODO return a vec as some moves wont be valid, or check if a move is valid
            let move_order = game_state.get_move_order();
            
            for move_index in move_order {
                if game_state.is_move_valid(move_index) {
                    // make each move and store each result in child_game_state
                    game_state.make_move(&mut child_game_state, move_index);
                    // recursively evaluate the child game state
                    value = max(minimax(&child_game_state, depth - 1, alpha, beta), value);

                    alpha = max(alpha, value);

                    if value >= beta {
                        break;
                    }
                }
            }

            return value;
        }
        Player::Two => {
            let mut value = INF;

            let move_order = game_state.get_move_order();
            for move_index in move_order {
                if game_state.is_move_valid(move_index) {
                    // make each move and store each result in child_game_state
                    game_state.make_move(&mut child_game_state, move_index);
                    // recursively evaluate the child game state
                    value = min(minimax(&child_game_state, depth - 1, alpha, beta), value);

                    beta = min(beta, value);

                    if value <= alpha {
                        break;
                    }
                }
            }

            return value;
        }
    }
}
