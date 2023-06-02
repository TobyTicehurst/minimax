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
    type Move: Copy;

    fn new() -> Self where Self: Sized;

    fn get_move_order(&self) -> [Self::Move; 6];

    fn make_move(&self, next_state: &mut Self, player_move: Self::Move);

    fn heuristic(&self) -> i32;

    fn get_turn(&self) -> Player;

    fn is_game_over(&self) -> bool;

    fn is_move_valid(&self, player_move: Self::Move) -> bool;

    fn print(&self);

    //fn test() -> &mut dyn GameState;
}


pub struct Solver<'a, T: GameState> {
    pub start_game_state: &'a T,
    pub depth: u32,
}

impl<'a, T: GameState> Solver<'a, T> {
    pub fn solve(&self) {
        println!("Depth: {}", self.depth);
        let now = Instant::now();

        let evaluation: i32;
        if self.depth == 0 || self.start_game_state.is_game_over() {
            evaluation = self.start_game_state.heuristic()
        }
        else {
            evaluation = minimax(self.start_game_state, self.depth - 1, -1 * INF, INF);
        }

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

    // TODO return on first call if depth is 0 or game over

    //game_state.print();

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
                    let evaluation: i32;
                    // recursively evaluate the child game state
                    if depth == 0 || child_game_state.is_game_over() {
                        evaluation = child_game_state.heuristic();
                    }
                    else {
                        evaluation = minimax(&child_game_state, depth - 1, alpha, beta);
                    }
                    
                    value = max(evaluation, value);

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
                    let evaluation: i32;
                    // recursively evaluate the child game state
                    if depth == 0 || child_game_state.is_game_over() {
                        evaluation = child_game_state.heuristic();
                    }
                    else {
                        evaluation = minimax(&child_game_state, depth - 1, alpha, beta);
                    }
                    value = min(evaluation, value);

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
