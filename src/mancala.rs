use std::time::Instant;
use std::cmp::max;
use std::cmp::min;

use crate::endgame;

const INF: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    One,
    Two,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    // number of stones in each slot
    pub slots: [u32; 14],
    // who's turn is it next
    pub turn: Player,
    pub game_over: bool,
}

impl Position {
    #[allow(dead_code)]
    fn print(&self) {
        println!("");
        println!("  {} {} {} {} {} {}  ", self.slots[12], self.slots[11], self.slots[10], self.slots[9], self.slots[8], self.slots[7]);
        println!("{}             {}", self.slots[13], self.slots[6]);
        println!("  {} {} {} {} {} {}  ", self.slots[0], self.slots[1], self.slots[2], self.slots[3], self.slots[4], self.slots[5]);
        println!("Turn: Player {:?}", self.turn);
        println!("Evaluation at depth 0: {}", self.evaluate());
    }

    // TODO
    #[allow(dead_code)]
    fn evaluate(&self) -> i32 {
        let mut value = self.slots[6] as i32 - self.slots[13] as i32;
        if self.game_over {
            if value > 0 {
                value += 50;
            }
            else if value < 0 {
                value -= 50;
            }
        }
        return value;
    }
}

fn half_clone_position(position: &Position, child_position: &mut Position) {
    child_position.slots = position.slots.clone();
    child_position.game_over = position.game_over;
}

fn handle_game_over(child_position: &mut Position) {
    // handle game over
    let mut player_one_stones = 0;
    for i in 0..6 {
        player_one_stones += child_position.slots[i];
    }

    let mut player_two_stones = 0;
    for i in 7..13 {
        player_two_stones += child_position.slots[i];
    }

    if player_one_stones == 0 {
        child_position.slots[13] += player_two_stones;
        for i in 7..13 {
            child_position.slots[i] = 0;
        }
        child_position.game_over = true;
    }
    else if player_two_stones == 0 {
        child_position.slots[6] += player_one_stones;
        for i in 0..6 {
            child_position.slots[i] = 0;
        }
        child_position.game_over = true;
    }
}

// move index will be between 0 and 5 inclusive
fn player_one_move_stones(child_position: &mut Position, move_index: usize) -> usize {
    // take stones out of slot
    let number_stones = child_position.slots[move_index];
    child_position.slots[move_index] = 0;

    for i in 0..13 {
        // add stones for each loop around the board (usually 0)
        child_position.slots[i] += number_stones / 13;
        // then possibly add a stone for the remainder
        // could make this branchless
        if (i + 12 - move_index) % 13 < number_stones as usize % 13 {
            child_position.slots[i] += 1;
        }
    }

    return (move_index + number_stones as usize) % 13;
}

// move index will be between 7 and 12 inclusive
fn player_two_move_stones(child_position: &mut Position, move_index: usize) -> usize {
    // take stones out of slot
    let number_stones = child_position.slots[move_index];
    child_position.slots[move_index] = 0;

    // need to skip slot 6 so do this in 2 loops
    for i in 0..6 {
        // add stones for each loop around the board (usually 0)
        child_position.slots[i] += number_stones / 13;
        // then possibly add a stone for the remainder
        if (i + 13 - move_index) % 13 < number_stones as usize % 13 {
            child_position.slots[i] += 1;
        }
    }

    // skip slot 6 and include slot 13
    for i in 7..14 {
        // add stones for each loop around the board (usually 0)
        child_position.slots[i] += number_stones / 13;
        // then possibly add a stone for the remainder
        if (i + 12 - move_index) % 13 < number_stones as usize % 13 {
            child_position.slots[i] += 1;
        }
    }

    // this is a way to find the final slot while skipping slot 6
    return (7 + (move_index - 7 + number_stones as usize) % 13) % 14;
}

fn player_one_handle_final_slot(child_position: &mut Position, final_slot_index: usize, player_goal: usize) -> bool {
    let mut capture_ocurred = false;

    // if it landed in the player's goal, give them another turn,
    // but if not, test if they capture the enemy stones
    if final_slot_index != player_goal {
        child_position.turn = Player::Two;
        // test if the final slot is on the player's side
        if final_slot_index < player_goal {
            // test if the final slot is empty
            if child_position.slots[final_slot_index] == 0 {
                // if so, capture those stones
                let opposite_slot = 12 - final_slot_index;
                child_position.slots[player_goal] += child_position.slots[opposite_slot];
                child_position.slots[opposite_slot] = 0;
                capture_ocurred = true;
            }
        }
    }
    else {
        child_position.turn = Player::One;
    }

    return capture_ocurred;
}

fn player_two_handle_final_slot(child_position: &mut Position, final_slot_index: usize, player_goal: usize) -> bool {
    let mut capture_ocurred = false;

    // if it landed in the player's goal, give them another turn,
    // but if not, test if they capture the enemy stones
    if final_slot_index != player_goal {
        child_position.turn = Player::One;
        // test if the final slot is on the player's side
        if final_slot_index < player_goal {
            // test if the final slot is empty
            if child_position.slots[final_slot_index] == 0 {
                // if so, capture those stones
                let opposite_slot = 12 - final_slot_index;
                child_position.slots[player_goal] += child_position.slots[opposite_slot];
                child_position.slots[opposite_slot] = 0;
                capture_ocurred = true;
            }
        }
    }
    else {
        child_position.turn = Player::Two;
    }

    return capture_ocurred;
}

fn player_one_make_move(position: &Position, child_position: &mut Position, move_index: usize) {
    // deep copy position
    half_clone_position(position, child_position);

    // get the goal indices for later
    let player_goal = 6;

    // where did the final stone land?
    let final_slot_index = player_one_move_stones(child_position, move_index);
    let capture_ocurred = player_one_handle_final_slot(child_position, final_slot_index, player_goal);

    // these are the only ways the game can end on player one's turn
    if move_index == 5 || capture_ocurred {
        handle_game_over(child_position);
    }
}

fn player_two_make_move(position: &Position, child_position: &mut Position, move_index: usize) {
    // deep copy position
    half_clone_position(position, child_position);

    // get the goal indices for later
    let player_goal = 13;

    // where did the final stone land?
    let final_slot_index = player_two_move_stones(child_position, move_index);
    let capture_ocurred = player_two_handle_final_slot(child_position, final_slot_index, player_goal);

    // these are the only ways the game can end on player two's turn
    if move_index == 12 || capture_ocurred {
        handle_game_over(child_position);
    }
}

fn get_move_order_player_one(position: &Position, move_order: &mut [usize; 6], best_move: Option<usize>) {
    let mut priority_index: usize;
    let mut reverse_index: usize = 0;

    match best_move {
        Some(m) => {
            move_order[0] = m;
            priority_index = 1;
            for i in 0..6 {
                if i == m {
                    continue;
                }
                // if playing this move gives another turn
                if i as u32 + position.slots[i] == 6 {
                    move_order[priority_index] = i;
                    priority_index += 1;
                }
                else {
                    move_order[5 - reverse_index] = i;
                    reverse_index += 1;
                }
            }
        },
        None => {
            priority_index = 0;
            for i in 0..6 {
                // if playing this move gives another turn
                if i as u32 + position.slots[i] == 6 {
                    move_order[priority_index] = i;
                    priority_index += 1;
                }
                else {
                    move_order[5 - reverse_index] = i;
                    reverse_index += 1;
                }
            }
        }
    }
}

fn get_move_order_player_two(position: &Position, move_order: &mut [usize; 6], best_move: Option<usize>) {
    let mut priority_index: usize;
    let mut reverse_index: usize = 0;

    match best_move {
        Some(m) => {
            move_order[0] = m;
            priority_index = 1;
            for i in 7..13 {
                if i == m {
                    continue;
                }
                // if playing this move gives another turn
                if i as u32 + position.slots[i] == 13 {
                    move_order[priority_index] = i;
                    priority_index += 1;
                }
                else {
                    move_order[5 - reverse_index] = i;
                    reverse_index += 1;
                }
            }
        },
        None => {
            priority_index = 0;
            for i in 7..13 {
                // if playing this move gives another turn
                if i as u32 + position.slots[i] == 13 {
                    move_order[priority_index] = i;
                    priority_index += 1;
                }
                else {
                    move_order[5 - reverse_index] = i;
                    reverse_index += 1;
                }
            }
        }
    }
}

#[allow(dead_code)]
static mut COUNTER: u64 = 0;
#[allow(dead_code)]
static mut COUNTERS: [u64; 49] = [0; 49];

#[allow(dead_code)]
pub fn minimax(
    position: &Position, 
    endgame: &endgame::Endgame,
    depth: u32, 
    mut alpha: i32, 
    mut beta: i32) -> i32 {
    //position.print();
    
    if depth == 0 || position.game_over {
        unsafe {
            COUNTERS[(position.slots[6] + position.slots[13]) as usize] += 1;
        }
        let evaluation = position.evaluate();
        //println!("Leaf node. Evaluation: {}", evaluation);
        return evaluation;
    }

    let stones_remaining = 48 - position.slots[6] - position.slots[13];
    if stones_remaining <= endgame.max_stones {
        unsafe {
            COUNTER += 1;
        }
        return position.slots[6] as i32 - position.slots[13] as i32 + endgame.get_value(position);
    }

    // default initialisation
    let mut child_position = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
        game_over: false,
    };

    let mut value: i32;

    match position.turn {
        Player::One => {
            value = -1 * INF;
            
            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_one(position, &mut move_order, None);
            for move_index in move_order {
                //let move_index = 5 - i;
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_one_make_move(&position, &mut child_position, move_index);
                // get next position in opening tree
                //let next_node = opening_tree.get_child(current_node, move_index);
                // recursively evaluate the child position
                let evaluation = minimax(&child_position, endgame, depth - 1, alpha, beta);
                if evaluation > value {
                    value = evaluation;
                }

                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }
        }
        Player::Two => {
            value = INF;

            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_two(position, &mut move_order, None);
            for move_index in move_order {
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_two_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = minimax(&child_position, endgame, depth - 1, alpha, beta);
                if evaluation < value {
                    value = evaluation;
                }

                beta = min(beta, value);

                if value <= alpha {
                    break;
                }
            }
        }
    }

    return value;
}

#[allow(dead_code)]
pub fn minimax_no_depth(
    position: &Position, 
    endgame: &endgame::Endgame,
    mut alpha: i32, 
    mut beta: i32) -> i32 {
    //position.print();
    
    if position.game_over {
        return position.slots[6] as i32 - position.slots[13] as i32;
    }

    let stones_remaining = endgame.current_stones - position.slots[6] - position.slots[13];
    if stones_remaining <= endgame.max_stones {
        return position.slots[6] as i32 - position.slots[13] as i32 + endgame.get_value(position);
    }

    // default initialisation
    let mut child_position = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
        game_over: false,
    };

    let mut value: i32;

    match position.turn {
        Player::One => {
            value = -1 * INF;
            
            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_one(position, &mut move_order, None);
            for move_index in move_order {
                //let move_index = 5 - i;
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_one_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = minimax_no_depth(&child_position, endgame, alpha, beta);
                if evaluation > value {
                    value = evaluation;
                }

                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }
        }
        Player::Two => {
            value = INF;

            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_two(position, &mut move_order, None);
            for move_index in move_order {
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_two_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = minimax_no_depth(&child_position, endgame, alpha, beta);
                if evaluation < value {
                    value = evaluation;
                }

                beta = min(beta, value);

                if value <= alpha {
                    break;
                }
            }
        }
    }

    return value;
}

#[allow(dead_code)]
fn analyse(depth: u32, endgame: &endgame::Endgame) {
    let now = Instant::now();

    let position = Position {
        slots: [0, 0, 0, 0, 2, 0, 0, 1, 1, 1, 1, 1, 1, 0],
        turn: Player::One,
        game_over: false,
    };

    let evaluation = minimax(&position, endgame, 100, 50, INF);
    let time = now.elapsed().as_millis();
    println!("Evaluation: {}, depth: {}, time: {}ms", evaluation, depth, time);
}

#[allow(dead_code)]
pub fn mtdf(depth: u32, endgame: &endgame::Endgame) -> i32 {
    let mut current_depth: u32 = 2;
    //let mut transpositionTable: HashMap<u64, TranspositionTableEntry> = HashMap::with_capacity(10_000_000);
    let mut guess = 0;

    while current_depth <= depth {
        let now = Instant::now();

        let position = Position {
            slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
            turn: Player::One,
            game_over: false,
        };

        let mut beta: i32;
        let mut lower_bound: i32 = -1 * INF;
        let mut upper_bound: i32 = INF;

        loop {
            if guess == lower_bound {
                beta = guess + 1;
            }
            else {
                beta = guess;
            }
            // println!("Guess: {}, Alpha: {}, Beta: {}", guess, beta - 1, beta);
            //guess = alphaBetaWithMemory(&mut transpositionTable, &position, current_depth, beta - 1, beta);
            guess = minimax(&position, endgame, current_depth, beta - 1, beta);
            
            //println!("New Guess: {}", guess);
            if guess < beta {
                upper_bound = guess;
            }
            else {
                lower_bound = guess;
            }
            //println!("lower_bound: {}, upper_bound: {}", lower_bound, upper_bound);

            if lower_bound >= upper_bound {
                break;
            }
        }

        let time = now.elapsed().as_millis();
        println!("Evaluation: {}, depth: {}, time: {}ms", guess, current_depth, time);
        // unsafe {
        //     for i in 0..49 {
        //         print!("{},", counters[i]);
        //         counters[i] = 0;
        //     }
        //     println!("");
        //     println!("{}", counter);
        // }
        
        current_depth += 2;
    }

    return guess;
}

#[allow(dead_code)]
fn analyse_all(depth: u32, endgame: &endgame::Endgame) {
    println!("analyseAll");
    let mut current_depth: u32 = 2;
    while current_depth <= depth {
        analyse(current_depth, endgame);
        current_depth += 2;
    }
}
