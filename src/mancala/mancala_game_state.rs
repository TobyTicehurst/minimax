#![allow(dead_code)]

use crate::minimax::GameState;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MancalaGameState {
    // number of stones in each slot
    pub pits: [u32; Self::TOTAL_PITS],
    // who's turn is it next
    pub turn: bool,
    pub game_over: bool,
}

impl GameState<MancalaGameState> for MancalaGameState {
    // fn heuristic(&self) -> i32 {
    //     let mut value =
    //         self.pits[Self::PLAYER_1_STORE] as i32 - self.pits[Self::PLAYER_2_STORE] as i32;
    //     if self.game_over {
    //         if value > 0 {
    //             value += Self::WIN_VALUE * 2;
    //         } else {
    //             value -= Self::WIN_VALUE * 2;
    //         }
    //     } else if self.pits[Self::PLAYER_1_STORE] > (Self::TOTAL_STONES / 2) {
    //         return Self::WIN_VALUE
    //             + (self.pits[Self::PLAYER_1_STORE] as i32 - (Self::TOTAL_STONES as i32 / 2)) * 2;
    //     } else if self.pits[Self::PLAYER_2_STORE] > (Self::TOTAL_STONES / 2) {
    //         return Self::WIN_VALUE
    //             + (self.pits[Self::PLAYER_2_STORE] as i32 - (Self::TOTAL_STONES as i32 / 2)) * 2;
    //     }

    //     value
    // }

    fn heuristic(&self) -> i32 {
        self.pits[Self::PLAYER_1_STORE] as i32 - self.pits[Self::PLAYER_2_STORE] as i32
    }

    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn is_maximising_player(&self) -> bool {
        self.turn == Self::PLAYER_1
    }

    fn get_children<'a>(
        &self,
        children_cache: &'a mut Vec<MancalaGameState>,
    ) -> &'a Vec<MancalaGameState> {
        children_cache.clear();
        let children: &mut Vec<MancalaGameState> = children_cache;
        let pit_offset;
        let players_store;
        let opponents_store;

        if self.turn == Self::PLAYER_1 {
            pit_offset = 0;
            players_store = Self::PLAYER_1_STORE;
            opponents_store = Self::PLAYER_2_STORE;
        } else {
            pit_offset = Self::PLAYER_1_STORE + 1;
            players_store = Self::PLAYER_2_STORE;
            opponents_store = Self::PLAYER_1_STORE;
        }

        // loop through each possible move, add it to the list is valid
        let mut moves = [0; Self::PITS_PER_SIDE];
        let mut move_index = 0;
        for player_move in 0..Self::PITS_PER_SIDE {
            if self.pits[pit_offset + player_move] > 0 {
                moves[move_index] = pit_offset + player_move;
                move_index += 1;
            }
        }

        // loop through each valid move, if we would get another turn, put it to the start of the list
        let mut ordered_moves = [0; Self::PITS_PER_SIDE];
        let mut start_index = 0;
        let mut end_index = move_index;
        for i in 0..move_index {
            let player_move = moves[i];
            if self.pits[player_move] as usize == players_store - player_move {
                ordered_moves[start_index] = player_move;
                start_index += 1;
            } else {
                ordered_moves[end_index - 1] = player_move;
                end_index -= 1;
            }
        }

        for i in 0..move_index {
            let player_move = ordered_moves[i];
            let mut child = *self;
            child.make_move(player_move, players_store, opponents_store);
            children.push(child);
        }

        children
    }

    fn get_children_cache(&self) -> Vec<MancalaGameState> {
        Vec::with_capacity(Self::PITS_PER_SIDE)
    }
}

impl MancalaGameState {
    pub const PITS_PER_SIDE: usize = 6;
    pub const PITS_NO_STORES: usize = Self::PITS_PER_SIDE * 2;
    const TOTAL_PITS: usize = Self::PITS_PER_SIDE * 2 + 2;
    const STONES_PER_PIT: u32 = 4;
    pub const PLAYER_1_STORE: usize = Self::PITS_PER_SIDE;
    pub const PLAYER_2_STORE: usize = Self::PITS_PER_SIDE * 2 + 1;

    const PLAYER_1: bool = true;
    //const PLAYER_2: bool = !PLAYER_1;

    const PLAYER_1_PITS: std::ops::Range<usize> = 0..Self::PLAYER_1_STORE;
    const PLAYER_2_PITS: std::ops::Range<usize> = (Self::PLAYER_1_STORE + 1)..Self::PLAYER_2_STORE;

    const DEFAULT_STARTING_POSITION: [u32; Self::TOTAL_PITS] =
        [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0];

    pub const TOTAL_STONES: u32 = Self::STONES_PER_PIT * Self::PITS_PER_SIDE as u32 * 2;
    const WIN_VALUE: i32 = Self::TOTAL_STONES as i32 + 2;

    pub fn new() -> MancalaGameState {
        MancalaGameState {
            pits: [0; Self::TOTAL_PITS],
            turn: Self::PLAYER_1,
            game_over: false,
        }
    }

    pub fn default() -> MancalaGameState {
        MancalaGameState {
            pits: Self::DEFAULT_STARTING_POSITION,
            turn: Self::PLAYER_1,
            game_over: false,
        }
    }

    pub fn pretty_print(&self) {
        let mut pretty = String::new();

        // print boarder
        pretty.push_str("┼─────────────────────────────────────────────────┼\n");

        // print player 2 move indices
        pretty.push_str("│         F     E     D     C     B     A         │\n");

        // print player 2 pits
        pretty.push_str("│       ");
        for i in 0..Self::PITS_PER_SIDE {
            let pit_index = Self::PLAYER_2_STORE - i - 1;
            pretty.push_str(format!("[{:>2} ] ", self.pits[pit_index]).as_str());
        }
        pretty.push_str("      │\n");

        // print stores
        pretty.push_str(format!("│ [{:>2} ] ", self.pits[Self::PLAYER_2_STORE]).as_str());
        for _i in 0..Self::PITS_PER_SIDE {
            pretty.push_str("      ");
        }
        pretty.push_str(format!("[{:>2} ] ", self.pits[Self::PLAYER_1_STORE]).as_str());
        pretty.push_str("│\n");

        // print player 1 pits
        pretty.push_str("│       ");
        for i in 0..Self::PITS_PER_SIDE {
            pretty.push_str(format!("[{:>2} ] ", self.pits[i]).as_str());
        }
        pretty.push_str("      │\n");

        // print player 1 move indices
        pretty.push_str("│         A     B     C     D     E     F         │\n");

        // print boarder
        pretty.push_str("┼─────────────────────────────────────────────────┼\n");

        println!("{}", pretty);
    }

    pub fn handle_game_over(&mut self) {
        // handle game over
        let mut player_one_stones = 0;
        for i in Self::PLAYER_1_PITS {
            player_one_stones += self.pits[i];
        }

        let mut player_two_stones = 0;
        for i in Self::PLAYER_2_PITS {
            player_two_stones += self.pits[i];
        }

        // assume only player one or player two can have 0 stones (which is true in valid game play)
        if player_one_stones == 0 {
            self.pits[Self::PLAYER_2_STORE] += player_two_stones;
            for i in Self::PLAYER_2_PITS {
                self.pits[i] = 0;
            }
            self.game_over = true;
        } else if player_two_stones == 0 {
            self.pits[Self::PLAYER_1_STORE] += player_one_stones;
            for i in Self::PLAYER_1_PITS {
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
            if current_pit % Self::TOTAL_PITS == opponents_store {
                current_pit += 1;
            }
            self.pits[current_pit % Self::TOTAL_PITS] += 1;
            current_pit += 1;
        }

        // return the final pit played to
        (current_pit - 1) % Self::TOTAL_PITS
    }

    pub fn make_move(&mut self, player_move: usize, players_store: usize, opponents_store: usize) {
        let final_pit = self.move_stones(player_move, opponents_store);

        let mut capture_occurred = false;

        // if another turn is not granted (see rules)
        if final_pit != players_store {
            self.turn = !self.turn;

            // if a capture occurs (see capturing rules)
            if final_pit < players_store
                && final_pit >= players_store - MancalaGameState::PITS_PER_SIDE
                && self.pits[final_pit] == 1
            {
                capture_occurred = true;
                // capture stones in the opposite pit
                let opposite_pit = Self::TOTAL_PITS - 2 - final_pit;
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
            || player_move == Self::PLAYER_1_STORE - 1
            || player_move == Self::PLAYER_2_STORE - 1
        {
            self.handle_game_over();
        }
    }

    pub fn generate_children_memory(max_depth: u32) -> Vec<Vec<MancalaGameState>> {
        vec![Vec::with_capacity(Self::PITS_PER_SIDE); (max_depth + 1) as usize]
    }

    pub fn get_children_naive(&self) -> Vec<MancalaGameState> {
        let mut children = Vec::with_capacity(Self::PITS_PER_SIDE);

        let pit_offset;
        let players_store;
        let opponents_store;

        if self.turn == Self::PLAYER_1 {
            pit_offset = 0;
            players_store = Self::PLAYER_1_STORE;
            opponents_store = Self::PLAYER_2_STORE;
        } else {
            pit_offset = Self::PLAYER_1_STORE + 1;
            players_store = Self::PLAYER_2_STORE;
            opponents_store = Self::PLAYER_1_STORE;
        }

        for i in 0..Self::PITS_PER_SIDE {
            let player_move = pit_offset + i;
            if self.pits[player_move] > 0 {
                let mut child = *self;
                child.make_move(player_move, players_store, opponents_store);
                children.push(child);
            }
        }

        children
    }

    pub fn get_valid_moves(&self) -> Vec<usize> {
        let mut valid_moves = Vec::with_capacity(Self::PITS_PER_SIDE);

        let pit_offset;

        if self.turn == Self::PLAYER_1 {
            pit_offset = 0;
        } else {
            pit_offset = Self::PLAYER_1_STORE + 1;
        }

        for i in 0..Self::PITS_PER_SIDE {
            let player_move = pit_offset + i;
            if self.pits[player_move] > 0 {
                valid_moves.push(player_move);
            }
        }

        valid_moves
    }
}
