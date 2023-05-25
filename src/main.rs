use std::cmp::max;
use std::cmp::min;
use std::time::Instant;


const INF: i32 = 100;

#[derive(Debug, Clone)]
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

fn player_one_make_move(position: &Position, child_position: &mut Position, move_index: usize) {
    // deep copy position
    child_position.slots = position.slots.clone();
    child_position.game_over = position.game_over;

    // take stones out of slot
    let mut number_stones = child_position.slots[move_index];
    child_position.slots[move_index] = 0;

    // get the goal indices for later
    let player_goal = 6;
    let opponent_goal = 13;

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

    // where did the final stone land?
    let final_slot_index = slot_index;
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

    // these are the only ways the game can end on player one's turn
    if move_index == 5 || capture_ocurred {
        handle_game_over(child_position);
    }
}

fn player_two_make_move(position: &Position, child_position: &mut Position, move_index: usize) {
    // deep copy position
    child_position.slots = position.slots.clone();
    child_position.game_over = position.game_over;

    // take stones out of slot
    let mut number_stones = child_position.slots[move_index];
    child_position.slots[move_index] = 0;

    // get the goal indices for later
    let player_goal = 13;
    let opponent_goal = 6;

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

    // where did the final stone land?
    let final_slot_index = slot_index;
    let mut capture_ocurred = false;

    // if it landed in the player's goal, give them another turn,
    // but if not, test if they capture the enemy stones
    if final_slot_index != player_goal {
        child_position.turn = Player::One;
        // test if the final slot is on the player's side
        if final_slot_index < player_goal && final_slot_index > opponent_goal {
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

fn minimax(position: &Position, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {

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

fn get_best_move(position: &Position, depth: u32) -> usize {

    // default initialisation
    let mut child_position = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
        game_over: false,
    };

    match position.turn {
        Player::One => {
            let mut value = -1 * INF;
            let mut best_move = 0;
            
            for move_index in 0..6 {
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_one_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = minimax(&child_position, depth - 1, -1 * INF, INF);
                if evaluation > value {
                    value = evaluation;
                    best_move = move_index;
                }
            }

            return best_move;
        }
        Player::Two => {
            let mut value = INF;
            let mut best_move = 0;

            for move_index in 7..13 {
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_two_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = minimax(&child_position, depth - 1, -1 * INF, INF);
                if evaluation < value {
                    value = evaluation;
                    best_move = move_index;
                }
            }

            return best_move;
        }
    }
}

fn play_game(depth: u32) {
    let mut position = Position {
        slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
        turn: Player::One,
        game_over: false,
    };
    let mut child_position = Position {
        slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
        turn: Player::One,
        game_over: false,
    };
    position.print();
    println!("Depth: {}", depth);

    loop {
        let now = Instant::now();
        
        let evaluation = minimax(&position, depth, -1 * INF, INF);
        let best_move = get_best_move(&position, depth);

        println!("Best move: {}", best_move);
        println!("Evaluation at depth {}: {}", depth, evaluation);
        // it prints '2'
        println!("{}ms\n", now.elapsed().as_millis());

        match position.turn {
            Player::One => {
                player_one_make_move(&position, &mut child_position, best_move);
            }
            Player::Two => {
                player_two_make_move(&position, &mut child_position, best_move);
            }
        }

        child_position.print();

        if child_position.game_over {
            return;
        }
        position.slots = child_position.slots.clone();
        position.turn = child_position.turn.clone();
    }
}

fn analyse() {
    let mut depth = 20;
    while depth < 21 {
        let now = Instant::now();
   
        depth += 1;
        println!("Depth: {}", depth);
        let position = Position {
            slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
            turn: Player::One,
            game_over: false,
        };
        let evaluation = minimax(&position, depth, -1 * INF, INF);
        let best_move = get_best_move(&position, depth);
        println!("Best move: {}", best_move);
        println!("Evaluation: {}", evaluation);
        // it prints '2'
        println!("{}ms\n", now.elapsed().as_millis());
    }
}

fn main() {
    let choice = 2;
    if choice == 1 {
        play_game(20);
    }
    else if choice == 2 {
        analyse();
    }

    
}
