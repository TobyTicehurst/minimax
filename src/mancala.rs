#![allow(dead_code)]

use crate::minimax::GameState;

const PITS_PER_SIDE: usize = 6;
const TOTAL_PITS: usize = PITS_PER_SIDE * 2 + 2;
//const STONES_PER_PIT: u32 = 4;
const PLAYER_1_STORE: usize = PITS_PER_SIDE;
const PLAYER_2_STORE: usize = PITS_PER_SIDE * 2 + 1;

const PLAYER_1: bool = true;
//const PLAYER_2: bool = !PLAYER_1;

const PLAYER_1_PITS: std::ops::Range<usize> = 0..PLAYER_1_STORE;
const PLAYER_2_PITS: std::ops::Range<usize> = (PLAYER_1_STORE + 1)..PLAYER_2_STORE;

const DEFAULT_STARTING_POSITION: [u32; TOTAL_PITS] = [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0];

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MancalaGameState {
    // number of stones in each slot
    pub pits: [u32; TOTAL_PITS],
    // who's turn is it next
    pub turn: bool,
    pub game_over: bool,
}

impl GameState<MancalaGameState> for MancalaGameState {
    fn heuristic(&self) -> i32 {
        let mut value = self.pits[6] as i32 - self.pits[13] as i32;
        if self.game_over {
            if value > 0 {
                value += 50;
            } else {
                value -= 50;
            }
        }
        value
    }

    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn is_maximising_player(&self) -> bool {
        self.turn == PLAYER_1
    }

    fn get_children<'a>(
        &self,
        children_memory: &'a mut Vec<MancalaGameState>,
    ) -> &'a Vec<MancalaGameState> {
        children_memory.clear();
        let children: &mut Vec<MancalaGameState> = children_memory;
        let players_pits;
        let players_store;
        let opponents_store;

        if self.turn == PLAYER_1 {
            players_pits = PLAYER_1_PITS;
            players_store = PLAYER_1_STORE;
            opponents_store = PLAYER_2_STORE;
        } else {
            players_pits = PLAYER_2_PITS;
            players_store = PLAYER_2_STORE;
            opponents_store = PLAYER_1_STORE;
        }

        for player_move in players_pits {
            // if this is a valid move
            if self.pits[player_move] > 0 {
                // copy self TODO - does this need to be a copy?
                let mut child = *self;
                child.make_move(player_move, players_store, opponents_store);
                children.push(child);
            }
        }

        // smart_reorder(&mut children);
        children.reverse();
        children
    }
}

// fn smart_reorder(children: &mut Vec<MancalaGameState>) {
//     // first try any moves which result in a second turn
//     // next try any moves which result in a capture
//     // finally try any moves which
// }

impl MancalaGameState {
    pub fn new() -> MancalaGameState {
        MancalaGameState {
            pits: [0; TOTAL_PITS],
            turn: PLAYER_1,
            game_over: false,
        }
    }

    pub fn default() -> MancalaGameState {
        MancalaGameState {
            pits: DEFAULT_STARTING_POSITION,
            turn: PLAYER_1,
            game_over: false,
        }
    }

    fn handle_game_over(&mut self) {
        // handle game over
        let mut player_one_stones = 0;
        for i in PLAYER_1_PITS {
            player_one_stones += self.pits[i];
        }

        let mut player_two_stones = 0;
        for i in PLAYER_2_PITS {
            player_two_stones += self.pits[i];
        }

        if player_one_stones == 0 {
            self.pits[PLAYER_2_STORE] += player_two_stones;
            for i in PLAYER_2_PITS {
                self.pits[i] = 0;
            }
            self.game_over = true;
        } else if player_two_stones == 0 {
            self.pits[PLAYER_1_STORE] += player_one_stones;
            for i in PLAYER_1_PITS {
                self.pits[i] = 0;
            }
            self.game_over = true;
        }
    }

    // moves stones and returns the final pit played in to
    fn move_stones(&mut self, player_move: usize, opponents_store: usize) -> usize {
        // take stones out of the chosen pit
        let number_of_stones: u32 = self.pits[player_move];
        self.pits[player_move] = 0;

        // add the remaining stones to each pit (except the opponent's store)
        let mut current_pit = player_move + 1;
        for _ in 0..number_of_stones {
            // skip the opponent's store
            if current_pit % TOTAL_PITS == opponents_store {
                current_pit += 1;
            }
            self.pits[current_pit % TOTAL_PITS] += 1;
            current_pit += 1;
        }

        // return the final pit played to
        (current_pit - 1) % TOTAL_PITS
    }

    fn make_move(&mut self, player_move: usize, players_store: usize, opponents_store: usize) {
        let final_pit = self.move_stones(player_move, opponents_store);

        let mut capture_occurred = false;

        // if another turn is not granted (see rules)
        if final_pit != players_store {
            self.turn = !self.turn;

            // if a capture occurs (see capturing rules)
            if final_pit < players_store && self.pits[final_pit] == 1 {
                capture_occurred = true;
                // capture stones in the opposite pit
                let opposite_pit = TOTAL_PITS - 2 - final_pit;
                let stones_to_capture = self.pits[opposite_pit];
                self.pits[opposite_pit] = 0;

                // move those stones to the player's store
                self.pits[players_store] += stones_to_capture;
            }
        }

        // handle game over
        // game can only be over if:
        //      the current player has run out of stones:
        //          move was made from the rightmost pit (and has 7 or fewer stones)
        //          This was the only valid move
        //      The opposite player has had stones captured as a result of this move
        if capture_occurred
            || player_move == PLAYER_1_STORE - 1
            || player_move == PLAYER_2_STORE - 1
        {
            self.handle_game_over();
        }
    }

    pub fn generate_children_memory(max_depth: u32) -> Vec<Vec<MancalaGameState>> {
        vec![Vec::with_capacity(PITS_PER_SIDE); (max_depth + 1) as usize]
    }
}
