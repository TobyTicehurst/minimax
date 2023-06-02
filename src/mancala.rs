//mod minimax;
use crate::minimax::Player;
use crate::minimax::GameState;

const SLOTS_PER_SIDE: usize = 6;
const TOTAL_SLOTS: usize = SLOTS_PER_SIDE * 2 + 2;
//const STONES_PER_SLOT: u32 = 4;

pub struct MancalaGameState {
    // number of stones in each slot
    pub slots: [u32; TOTAL_SLOTS],
    // who's turn is it next
    pub turn: Player,
    pub game_over: bool,
}

impl GameState for MancalaGameState {
    fn new() -> MancalaGameState {
        MancalaGameState {
            slots: [0; TOTAL_SLOTS],
            turn: Player::One,
            game_over: false,
        }
    }

    fn make_move(&self, next_state: &mut MancalaGameState, move_index: usize) {
        match self.get_turn() {
            Player::One => self.player_one_make_move(next_state, move_index),
            Player::Two => self.player_two_make_move(next_state, move_index),
        }
    }

    fn get_move_order(&self) -> Vec<usize> {
        let mut move_order: [usize; SLOTS_PER_SIDE] = [0; SLOTS_PER_SIDE];
        match self.get_turn() {
            Player::One => self.get_move_order_player_one(&mut move_order),
            Player::Two => self.get_move_order_player_two(&mut move_order),
        }

        return move_order.to_vec();
    }

    fn heuristic(&self) -> i32 {
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

    fn get_turn(&self) -> Player {
        return self.turn;
    }

    fn is_game_over(&self) -> bool {
        return self.game_over;
    }

    fn is_move_valid(&self, move_index: usize) -> bool {
        return self.slots[move_index] != 0;
    }

    fn print(&self) {
        println!("  {} {} {} {} {} {}  ", self.slots[12], self.slots[11], self.slots[10], self.slots[9], self.slots[8], self.slots[7]);
        println!("{}             {}", self.slots[13], self.slots[6]);
        println!("  {} {} {} {} {} {}  ", self.slots[0], self.slots[1], self.slots[2], self.slots[3], self.slots[4], self.slots[5]);
        println!("Turn: Player {:?}", self.turn);
        println!("Evaluation at depth 0: {}", self.heuristic());
        println!("");
    }
}

impl MancalaGameState {
    // TODO turn this into Copy()
    fn half_clone_position(&self, child_game_state: &mut MancalaGameState) {
        child_game_state.slots = self.slots.clone();
        child_game_state.game_over = self.game_over;
    }

    fn handle_game_over(&mut self) {
        // handle game over
        let mut player_one_stones = 0;
        for i in 0..6 {
            player_one_stones += self.slots[i];
        }

        let mut player_two_stones = 0;
        for i in 7..13 {
            player_two_stones += self.slots[i];
        }

        if player_one_stones == 0 {
            self.slots[13] += player_two_stones;
            for i in 7..13 {
                self.slots[i] = 0;
            }
            self.game_over = true;
        }
        else if player_two_stones == 0 {
            self.slots[6] += player_one_stones;
            for i in 0..6 {
                self.slots[i] = 0;
            }
            self.game_over = true;
        }
    }

    // move index will be between 0 and 5 inclusive
    fn player_one_move_stones(&mut self, move_index: usize) -> usize {
        // take stones out of slot
        let number_stones = self.slots[move_index];
        self.slots[move_index] = 0;

        for i in 0..13 {
            // add stones for each loop around the board (usually 0)
            self.slots[i] += number_stones / 13;
            // then possibly add a stone for the remainder
            // could make this branchless
            if (i + 12 - move_index) % 13 < number_stones as usize % 13 {
                self.slots[i] += 1;
            }
        }

        return (move_index + number_stones as usize) % 13;
    }

    // move index will be between 7 and 12 inclusive
    fn player_two_move_stones(&mut self, move_index: usize) -> usize {
        // take stones out of slot
        let number_stones = self.slots[move_index];
        self.slots[move_index] = 0;

        // need to skip slot 6 so do this in 2 loops
        for i in 0..6 {
            // add stones for each loop around the board (usually 0)
            self.slots[i] += number_stones / 13;
            // then possibly add a stone for the remainder
            if (i + 13 - move_index) % 13 < number_stones as usize % 13 {
                self.slots[i] += 1;
            }
        }

        // skip slot 6 and include slot 13
        for i in 7..14 {
            // add stones for each loop around the board (usually 0)
            self.slots[i] += number_stones / 13;
            // then possibly add a stone for the remainder
            if (i + 12 - move_index) % 13 < number_stones as usize % 13 {
                self.slots[i] += 1;
            }
        }

        // this is a way to find the final slot while skipping slot 6
        return (7 + (move_index - 7 + number_stones as usize) % 13) % 14;
    }

    // fn move_stones(&mut self, move_index: usize, opponent_goal: usize) -> usize {
    //     // take stones out of slot
    //     let mut number_stones = self.slots[move_index];
    //     self.slots[move_index] = 0;

    //     // start here...
    //     let mut slot_index = move_index;
    //     // ...and add stones 1 by 1 to slots
    //     while number_stones > 0 {
    //         number_stones -= 1;
    //         slot_index += 1;
    //         // but skip the opponent's goal
    //         if slot_index == opponent_goal {
    //             slot_index += 1;
    //         }
    //         slot_index %= 14;
    //         self.slots[slot_index] += 1;
    //     }

    //     return slot_index;
    // }

    fn player_one_handle_final_slot(&mut self, final_slot_index: usize, player_goal: usize) -> bool {
        let mut capture_ocurred = false;

        // if it landed in the player's goal, give them another turn,
        // but if not, test if they capture the enemy stones
        if final_slot_index != player_goal {
            self.turn = Player::Two;
            // test if the final slot is on the player's side
            if final_slot_index < player_goal {
                // test if the final slot is empty
                if self.slots[final_slot_index] == 0 {
                    // if so, capture those stones
                    let opposite_slot = 12 - final_slot_index;
                    self.slots[player_goal] += self.slots[opposite_slot];
                    self.slots[opposite_slot] = 0;
                    capture_ocurred = true;
                }
            }
        }
        else {
            self.turn = Player::One;
        }

        return capture_ocurred;
    }

    fn player_two_handle_final_slot(&mut self, final_slot_index: usize, player_goal: usize) -> bool {
        let mut capture_ocurred = false;

        // if it landed in the player's goal, give them another turn,
        // but if not, test if they capture the enemy stones
        if final_slot_index != player_goal {
            self.turn = Player::One;
            // test if the final slot is on the player's side
            if final_slot_index < player_goal {
                // test if the final slot is empty
                if self.slots[final_slot_index] == 0 {
                    // if so, capture those stones
                    let opposite_slot = 12 - final_slot_index;
                    self.slots[player_goal] += self.slots[opposite_slot];
                    self.slots[opposite_slot] = 0;
                    capture_ocurred = true;
                }
            }
        }
        else {
            self.turn = Player::Two;
        }

        return capture_ocurred;
    }

    fn player_one_make_move(&self, child_position: &mut MancalaGameState, move_index: usize) {
        // deep copy position
        self.half_clone_position(child_position);
    
        // get the goal indices for later
        let player_goal = 6;
    
        // where did the final stone land?
        let final_slot_index = child_position.player_one_move_stones(move_index);
        
        let capture_ocurred = child_position.player_one_handle_final_slot(final_slot_index, player_goal);
    
        // these are the only ways the game can end on player one's turn
        if move_index == 5 || capture_ocurred {
            child_position.handle_game_over();
        }
    }
    
    fn player_two_make_move(&self, child_position: &mut MancalaGameState, move_index: usize) {
        // deep copy position
        self.half_clone_position(child_position);
    
        // get the goal indices for later
        let player_goal = 13;
    
        // where did the final stone land?
        let final_slot_index = child_position.player_two_move_stones(move_index);
        let capture_ocurred = child_position.player_two_handle_final_slot(final_slot_index, player_goal);
    
        // these are the only ways the game can end on player two's turn
        if move_index == 12 || capture_ocurred {
            child_position.handle_game_over();
        }
    }

    fn get_move_order_player_one(&self, move_order: &mut [usize; 6]) {
        let mut priority_index = 0;
    
        for i in 0..6 {
            // if playing this move gives another turn
            if i as u32 + self.slots[i] == 6 {
                move_order[priority_index] = i;
                priority_index += 1;
            }
            else {
                move_order[5 + priority_index - i] = i;
            }
        }
    }
    
    fn get_move_order_player_two(&self, move_order: &mut [usize; 6]) {
        let mut priority_index = 0;
        for i in 7..13 {
            // if playing this move gives another turn
            if i as u32 + self.slots[i] == 13 {
                move_order[priority_index] = i;
                priority_index += 1;
            }
            else {
                move_order[12 + priority_index - i] = i;
            }
        }
    }
}

