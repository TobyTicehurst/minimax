use std::cmp::max;
use std::cmp::min;
use std::time::Instant;

// mod tree;
// use crate::tree::Tree;
// use crate::tree::Node;

mod minimax;
mod mancala;

const INF: i32 = 100;

#[derive(Debug, Clone, PartialEq)]
enum Player {
    One,
    Two,
}

#[derive(Debug)]
struct Position {
    // number of stones in each slot
    slots: [u32; 14],
    // who's turn is it next
    turn: Player,
    game_over: bool,
}

impl Position {
    fn print(&self) {
        println!("  {} {} {} {} {} {}  ", self.slots[12], self.slots[11], self.slots[10], self.slots[9], self.slots[8], self.slots[7]);
        println!("{}             {}", self.slots[13], self.slots[6]);
        println!("  {} {} {} {} {} {}  ", self.slots[0], self.slots[1], self.slots[2], self.slots[3], self.slots[4], self.slots[5]);
        println!("Turn: Player {:?}", self.turn);
        println!("Evaluation at depth 0: {}", self.evaluate());
        println!("");
    }

    fn evaluate(&self) -> i32 {
        let mut value = self.slots[6] as i32 - self.slots[13] as i32;
        if self.game_over {
            if value > 0 {
                value += 50;
            }
            else {
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

fn move_stones(child_position: &mut Position, move_index: usize, opponent_goal: usize) -> usize {
    // take stones out of slot
    let mut number_stones = child_position.slots[move_index];
    child_position.slots[move_index] = 0;

    // start here...
    let mut slot_index = move_index;
    // ...and add stones 1 by 1 to slots
    while number_stones > 0 {
        number_stones -= 1;
        slot_index += 1;
        // but skip the opponent's goal
        if slot_index == opponent_goal {
            slot_index += 1;
        }
        slot_index %= 14;
        child_position.slots[slot_index] += 1;
    }

    return slot_index;
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

fn get_move_order_player_one(position: &Position, move_order: &mut [usize; 6]) {
    let mut priority_index = 0;

    for i in 0..6 {
        // if playing this move gives another turn
        if i as u32 + position.slots[i] == 6 {
            move_order[priority_index] = i;
            priority_index += 1;
        }
        else {
            move_order[5 + priority_index - i] = i;
        }
    }

}

fn get_move_order_player_two(position: &Position, move_order: &mut [usize; 6]) {
    let mut priority_index = 0;
    for i in 7..13 {
        // if playing this move gives another turn
        if i as u32 + position.slots[i] == 13 {
            move_order[priority_index] = i;
            priority_index += 1;
        }
        else {
            move_order[12 + priority_index - i] = i;
        }
    }
}

fn minimax(
    position: &Position, 
    depth: u32, 
    mut alpha: i32, 
    mut beta: i32) -> i32 {

    //position.print();

    if depth == 0 || position.game_over {
        return position.evaluate();
    }

    // default initialisation
    let mut child_position = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
        game_over: false,
    };

    match position.turn {
        Player::One => {
            let mut value = -1 * INF;
            
            let mut move_order: [usize; 6] = [5, 4, 3, 2, 1, 0];
            get_move_order_player_one(position, &mut move_order);
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
                value = max(minimax(&child_position, depth - 1, alpha, beta), value);

                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }

            return value;
        }
        Player::Two => {
            let mut value = INF;

            let mut move_order: [usize; 6] = [5, 4, 3, 2, 1, 0];
            get_move_order_player_two(position, &mut move_order);
            for move_index in move_order {
                //let move_index = 12 - i;
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_two_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                value = min(minimax(&child_position, depth - 1, alpha, beta), value);

                beta = min(beta, value);

                if value <= alpha {
                    break;
                }
            }

            return value;
        }
    }
}

fn analyse(depth: u32) {
    println!("Depth: {}", depth);
    let now = Instant::now();

    let position = Position {
        slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
        turn: Player::One,
        game_over: false,
    };

    let evaluation = minimax(&position, depth, -1 * INF, INF);
    let time = now.elapsed().as_millis();
    println!("Evaluation: {}", evaluation);
    // it prints '2'
    println!("{}ms\n", time);
}

fn mtdf_test(max_depth: u32) {
    let mut depth = 2;
    while depth <= max_depth {
        let now = Instant::now();

        let position = Position {
            slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
            turn: Player::One,
            game_over: false,
        };

        let evaluation = minimax(&position, depth, -1 * INF, INF);
        let time = now.elapsed().as_millis();
        println!("Evaluation: {}, depth: {}, time: {}ms", evaluation, depth, time);
        depth += 2;
    }
}

// fn test_all_moves() {
//     let mut child_position_1 = Position {
//         slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//         turn: Player::One,
//         game_over: false,
//     };
//     let mut child_position_2 = Position {
//         slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//         turn: Player::One,
//         game_over: false,
//     };

//     println!("Testing...");
//     let now = Instant::now();
    
//     // test player 1 moves
//     for number_stones in 1..49 {
//         for move_index in 0..6 {
//             child_position_1.slots[move_index] = number_stones;
//             child_position_2.slots[move_index] = number_stones;
//             let final_slot_index_1 = move_stones(&mut child_position_1, move_index, 13);
//             let final_slot_index_2 = player_one_move_stones(&mut child_position_2, move_index);

//             let mut diff = false;
//             for i in 0..14 {
//                 if child_position_1.slots[i] != child_position_2.slots[i] {
//                     diff = true;
//                     break;
//                 }
//             }

//             if diff ||
//                final_slot_index_1 != final_slot_index_2 ||
//                child_position_1.turn != child_position_2.turn ||
//                child_position_1.game_over != child_position_2.game_over {
//                 println!("Move {}, stones {}\n", move_index, number_stones);
//                 println!("final_slot_index_1 {}, final_slot_index_2 {}\n", final_slot_index_1, final_slot_index_2);
//                 println!("turn 1 {:?}, turn 2 {:?}\n", child_position_1.turn, child_position_2.turn);
//                 println!("game_over 1 {}, game_over 2 {}\n", child_position_1.game_over, child_position_2.game_over);
//                 child_position_1.print();
//                 child_position_2.print();
//                 panic!();
//             }

//             for i in 0..14 {
//                 child_position_1.slots[i] = 0;
//                 child_position_2.slots[i] = 0;
//             }
            
//         }

//         child_position_1.turn = Player::Two;
//         child_position_2.turn = Player::Two;

//         for move_index in 7..13 {
//             child_position_1.slots[move_index] = number_stones;
//             child_position_2.slots[move_index] = number_stones;
//             let final_slot_index_1 = move_stones(&mut child_position_1, move_index, 6);
//             let final_slot_index_2 = player_two_move_stones(&mut child_position_2, move_index);

//             let mut diff = false;
//             for i in 0..14 {
//                 if child_position_1.slots[i] != child_position_2.slots[i] {
//                     diff = true;
//                     break;
//                 }
//             }

//             if diff ||
//                final_slot_index_1 != final_slot_index_2 ||
//                child_position_1.turn != child_position_2.turn ||
//                child_position_1.game_over != child_position_2.game_over {
//                 println!("Move {}, stones {}\n", move_index, number_stones);
//                 println!("final_slot_index_1 {}, final_slot_index_2 {}\n", final_slot_index_1, final_slot_index_2);
//                 println!("turn 1 {:?}, turn 2 {:?}\n", child_position_1.turn, child_position_2.turn);
//                 println!("game_over 1 {}, game_over 2 {}\n", child_position_1.game_over, child_position_2.game_over);
//                 child_position_1.print();
//                 child_position_2.print();
//                 panic!();
//             }

//             for i in 0..14 {
//                 child_position_1.slots[i] = 0;
//                 child_position_2.slots[i] = 0;
//             }
//         }
//     }

//     let time = now.elapsed().as_millis();
//     println!("{}ms\n", time);
// }

fn main() {
    let depth: u32 = 30;
    let mut start_game_state = mancala::MancalaGameState {
        slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
        turn: minimax::Player::One,
        game_over: false,
    };
    let mut solver = minimax::Solver {
        start_game_state: &mut start_game_state,
        depth: depth,
    };
    solver.solve();

    mtdf_test(depth);
}
