#![allow(dead_code)]

use crate::minimax::GameState;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MancalaGameState {
    // number of stones in each slot
    // u8 keeps the whole state in 16 bytes (there are only 48 stones)
    pub pits: [u8; Self::TOTAL_PITS],
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
        let (players_store, opponents_store) = self.stores();
        let (moves, num_moves) = self.ordered_moves();

        for &player_move in &moves[..num_moves] {
            // build the child in place rather than copying it in afterwards:
            // copying a freshly written state stalls on store-to-load forwarding
            children_cache.push(*self);
            let child = children_cache.last_mut().unwrap();
            child.make_move(player_move, players_store, opponents_store);
        }

        children_cache
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

    const DEFAULT_STARTING_POSITION: [u8; Self::TOTAL_PITS] =
        [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0];

    pub const TOTAL_STONES: u32 = Self::STONES_PER_PIT * Self::PITS_PER_SIDE as u32 * 2;
    const WIN_VALUE: i32 = Self::TOTAL_STONES as i32 + 2;

    // (players_store, opponents_store) for the player to move
    #[inline]
    fn stores(&self) -> (usize, usize) {
        if self.turn == Self::PLAYER_1 {
            (Self::PLAYER_1_STORE, Self::PLAYER_2_STORE)
        } else {
            (Self::PLAYER_2_STORE, Self::PLAYER_1_STORE)
        }
    }

    // valid moves for the player to move, in search order:
    //  1. moves which end in the player's store (another turn), from the far end towards the store
    //  2. moves which capture, from the store side backwards
    //  3. the rest, from the store side backwards
    pub fn ordered_moves(&self) -> ([usize; Self::PITS_PER_SIDE], usize) {
        let (players_store, _) = self.stores();
        let pit_offset = players_store - Self::PITS_PER_SIDE;
        // every pit except the opponent's store
        const LAP: usize = MancalaGameState::TOTAL_PITS - 1;

        let mut extra_turns = [0; Self::PITS_PER_SIDE];
        let mut num_extra_turns = 0;
        let mut captures = [0; Self::PITS_PER_SIDE];
        let mut num_captures = 0;
        let mut others = [0; Self::PITS_PER_SIDE];
        let mut num_others = 0;

        for i in 0..Self::PITS_PER_SIDE {
            let stones = self.pits[pit_offset + i] as usize;
            if stones == 0 {
                continue;
            }
            // where the last stone lands, counting the player's own pits as 0..6 and their store as 6
            let landing = (i + stones) % LAP;
            if landing == Self::PITS_PER_SIDE {
                extra_turns[num_extra_turns] = pit_offset + i;
                num_extra_turns += 1;
                continue;
            }
            let is_capture = landing < Self::PITS_PER_SIDE && {
                let landing_pit = pit_offset + landing;
                let opposite_pit = Self::TOTAL_PITS - 2 - landing_pit;
                // landing pit empty before the last stone (13 stones lands back in the emptied start pit)
                // and the opposite pit has stones (a full lap adds one)
                let landing_empty = if stones < LAP {
                    self.pits[landing_pit] == 0
                } else {
                    stones == LAP
                };
                landing_empty && (self.pits[opposite_pit] > 0 || stones >= LAP)
            };
            if is_capture {
                captures[num_captures] = pit_offset + i;
                num_captures += 1;
            } else {
                others[num_others] = pit_offset + i;
                num_others += 1;
            }
        }

        let mut ordered_moves = [0; Self::PITS_PER_SIDE];
        let mut n = 0;
        for &m in &extra_turns[..num_extra_turns] {
            ordered_moves[n] = m;
            n += 1;
        }
        for &m in captures[..num_captures].iter().rev() {
            ordered_moves[n] = m;
            n += 1;
        }
        for &m in others[..num_others].iter().rev() {
            ordered_moves[n] = m;
            n += 1;
        }

        (ordered_moves, n)
    }

    // non-zero if player 1 has any stones in their pits
    // reads the 6 pits as one 64 bit word (masking off the two bytes beyond them)
    #[inline]
    fn player_1_side(&self) -> u64 {
        u64::from_le_bytes(self.pits[0..8].try_into().unwrap()) & 0x0000_FFFF_FFFF_FFFF
    }

    // non-zero if player 2 has any stones in their pits (bytes 7..13, masking off both stores)
    #[inline]
    fn player_2_side(&self) -> u64 {
        u64::from_le_bytes(self.pits[6..14].try_into().unwrap()) & 0x00FF_FFFF_FFFF_FF00
    }

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
        // only one side can be empty in valid game play, but if both are this still ends the game
        if self.player_1_side() == 0 {
            let mut player_two_stones = 0;
            for i in Self::PLAYER_2_PITS {
                player_two_stones += self.pits[i];
                self.pits[i] = 0;
            }
            self.pits[Self::PLAYER_2_STORE] += player_two_stones;
            self.game_over = true;
        } else if self.player_2_side() == 0 {
            let mut player_one_stones = 0;
            for i in Self::PLAYER_1_PITS {
                player_one_stones += self.pits[i];
                self.pits[i] = 0;
            }
            self.pits[Self::PLAYER_1_STORE] += player_one_stones;
            self.game_over = true;
        }
    }

    // moves stones and returns the final pit played in to
    fn move_stones(&mut self, player_move: usize, opponents_store: usize) -> usize {
        // take stones out of the chosen pit
        let number_of_stones = self.pits[player_move];
        self.pits[player_move] = 0;

        // add the stones to each following pit (except the opponent's store)
        // wrap with a compare rather than a modulo per stone
        let mut current_pit = player_move;
        for _ in 0..number_of_stones {
            current_pit += 1;
            if current_pit == Self::TOTAL_PITS {
                current_pit = 0;
            }
            // skip the opponent's store
            if current_pit == opponents_store {
                current_pit += 1;
                if current_pit == Self::TOTAL_PITS {
                    current_pit = 0;
                }
            }
            self.pits[current_pit] += 1;
        }

        // return the final pit played to
        current_pit
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
                let opposite_pit = Self::TOTAL_PITS - 2 - final_pit;
                let stones_to_capture = self.pits[opposite_pit];
                if stones_to_capture > 0 {
                    capture_occurred = true;
                    // move the capturing stone and the opposite stones to the player's store
                    self.pits[opposite_pit] = 0;
                    self.pits[final_pit] = 0;
                    self.pits[players_store] += stones_to_capture + 1;
                }
            }
        }

        // handle game over
        // game can only be over if a capture occurred or the final stone was moved
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

#[cfg(test)]
mod tests {
    use super::*;

    const P1_STORE: usize = MancalaGameState::PLAYER_1_STORE;
    const P2_STORE: usize = MancalaGameState::PLAYER_2_STORE;

    fn state(pits: [u8; MancalaGameState::TOTAL_PITS], turn: bool) -> MancalaGameState {
        MancalaGameState {
            pits,
            turn,
            game_over: false,
        }
    }

    fn total(s: &MancalaGameState) -> u32 {
        s.pits.iter().map(|&p| p as u32).sum()
    }

    #[test]
    fn capture_takes_last_stone_and_opposite_stones() {
        // P1 sows 1 stone from A into empty B; opposite B is pit 11 with 5 stones
        let mut s = state([1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 5, 0, 0], true);
        s.make_move(0, P1_STORE, P2_STORE);
        assert_eq!(s.pits, [0, 0, 0, 2, 0, 0, 6, 0, 3, 0, 0, 0, 0, 0]);
        assert!(!s.turn);
        assert!(!s.game_over);
    }

    #[test]
    fn player_2_capture() {
        // P2 sows 1 stone from pit 7 into empty pit 8; opposite pit 4 has 3 stones
        let mut s = state([2, 0, 0, 0, 3, 0, 0, 1, 0, 0, 4, 0, 0, 0], false);
        s.make_move(7, P2_STORE, P1_STORE);
        assert_eq!(s.pits, [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 4]);
        assert!(s.turn);
    }

    #[test]
    fn no_capture_when_opposite_pit_empty() {
        let mut s = state([1, 0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0], true);
        s.make_move(0, P1_STORE, P2_STORE);
        assert_eq!(s.pits, [0, 1, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0]);
        assert!(!s.turn);
        assert!(!s.game_over);
    }

    #[test]
    fn no_capture_on_opponents_side() {
        // P1 sows 2 stones from F: store, then pit 7 (empty, opponent's side)
        let mut s = state([0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 4, 0, 0], true);
        s.make_move(5, P1_STORE, P2_STORE);
        assert_eq!(s.pits, [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 4, 0, 0]);
        assert!(!s.turn);
    }

    #[test]
    fn capture_with_last_own_stone_ends_game() {
        // P1's only stone captures, leaving P1's side empty: P2 sweeps their remaining stones
        let mut s = state([1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 5, 0, 0], true);
        s.make_move(0, P1_STORE, P2_STORE);
        assert!(s.game_over);
        assert_eq!(s.pits[P1_STORE], 6);
        assert_eq!(s.pits[P2_STORE], 3);
        assert_eq!(total(&s), 9);
    }

    #[test]
    fn capture_of_opponents_last_stones_ends_game() {
        let mut s = state([1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0], true);
        s.make_move(0, P1_STORE, P2_STORE);
        assert!(s.game_over);
        assert_eq!(s.pits[P1_STORE], 8);
        assert_eq!(s.pits[P2_STORE], 0);
    }

    #[test]
    fn thirteen_stone_lap_captures_into_starting_pit() {
        // 13 stones from A: one lap (skipping P2's store) ending back in the now-empty A
        let mut s = state([13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0], true);
        s.make_move(0, P1_STORE, P2_STORE);
        // opposite of A is pit 12: 2 + 1 sown = 3, plus the capturing stone
        assert_eq!(s.pits[0], 0);
        assert_eq!(s.pits[12], 0);
        assert_eq!(s.pits[P1_STORE], 1 + 3 + 1);
        assert_eq!(s.pits[P2_STORE], 0);
        assert_eq!(total(&s), 15);
    }

    #[test]
    fn last_stone_in_store_gives_extra_turn() {
        let mut s = MancalaGameState::default();
        // C has 4 stones: D, E, F, store
        s.make_move(2, P1_STORE, P2_STORE);
        assert!(s.turn);
        assert_eq!(s.pits, [4, 4, 0, 5, 5, 5, 1, 4, 4, 4, 4, 4, 4, 0]);
    }

    #[test]
    fn sowing_skips_opponents_store() {
        // 9 stones from F: P1 store, 7..12, skip P2 store, then A and B (B non-empty, so no capture)
        let mut s = state([0, 1, 1, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0], true);
        s.make_move(5, P1_STORE, P2_STORE);
        assert_eq!(s.pits, [1, 2, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0]);
        assert!(!s.turn);
    }

    #[test]
    fn full_lap_extra_turn_moves_are_ordered_first() {
        // F with 14 stones goes round once and ends in P1's store; A with 1 stone does not
        let s = state([1, 0, 0, 0, 0, 14, 0, 1, 0, 0, 0, 0, 0, 0], true);
        let mut cache = s.get_children_cache();
        let children = s.get_children(&mut cache);
        assert_eq!(children.len(), 2);
        assert!(children[0].turn, "extra-turn move should be searched first");
        assert!(!children[1].turn);
    }

    #[test]
    fn moves_are_ordered_extra_turns_then_captures_then_the_rest() {
        // A (1 stone) lands in empty B opposite 3 stones: capture
        // C (4 stones) ends in the store: extra turn
        // D (1 stone) lands in E which has stones, E (3 stones) lands on the opponent's side: neither
        // F (13 stones) laps back into the emptied F, opposite gets a stone: capture
        let s = state([1, 0, 4, 1, 3, 13, 0, 0, 0, 0, 0, 3, 0, 0], true);
        let (moves, n) = s.ordered_moves();
        assert_eq!(&moves[..n], &[2, 5, 0, 4, 3]);

        // the same position from player 2's side
        let s = state([0, 0, 0, 0, 3, 0, 0, 1, 0, 4, 1, 3, 13, 0], false);
        let (moves, n) = s.ordered_moves();
        assert_eq!(&moves[..n], &[9, 12, 7, 11, 10]);
    }

    #[test]
    fn stones_are_conserved_from_start() {
        let s = MancalaGameState::default();
        let mut cache = s.get_children_cache();
        for child in s.get_children(&mut cache).clone() {
            assert_eq!(total(&child), MancalaGameState::TOTAL_STONES);
            let mut cache2 = child.get_children_cache();
            for grandchild in child.get_children(&mut cache2) {
                assert_eq!(total(grandchild), MancalaGameState::TOTAL_STONES);
            }
        }
    }
}
