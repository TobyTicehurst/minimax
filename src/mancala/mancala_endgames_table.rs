#![allow(dead_code)]

use num_integer::binomial;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::{
    mancala::MancalaGameState,
    minimax::{EndgamesTable, Solver},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableEntry {
    pub evaluation: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MancalaEndgamesTable {
    // cache is:
    //  1. indexed by the number of stones in play, then
    //  2. sub-indexed by the number of stones in each pit
    // the sum over the results from 2 will result in an index into table
    // see get_index
    pub cache: Vec<[usize; MancalaGameState::PITS_NO_STORES + 2]>,
    // first skip over the tables with fewer number of stones, given by num_game_states(num_stones-1, 11)
    // next, for each pit except the last, add on the index given by num_game_states(remaining, 10-n)
    pub table: Vec<TableEntry>,
    pub max_stones: u32,
    // when calculating endgames, slowly build up the number of stones and use previous results
    current_stones: u32,
    // when calculating endgames, not all 48 stones are in play
    stones_in_play: u32,
}

fn factorial(n: u32) -> u128 {
    return (1..((n as u128) + 1)).product();
}

impl MancalaEndgamesTable {
    pub fn write_to_file(&self, filepath: &str) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(filepath)
            .unwrap();

        let num_stones_data = self.max_stones.to_ne_bytes();
        file.write_all(&num_stones_data).unwrap();
        let formatted_table: Vec<u8> = self
            .table
            .iter()
            .map(|eval| eval.evaluation as u8)
            .collect();
        file.write_all(&formatted_table).unwrap();
    }

    pub fn read_from_file(filepath: &str) -> Self {
        let mut data: Vec<u8> = fs::read(filepath).unwrap();
        println!("Read {} MB from disk", data.len() as f32 / 1e6);
        let table_data = data.split_off(4);
        let max_stones_data = data;
        let max_stones = u32::from_ne_bytes(max_stones_data.try_into().unwrap());
        let mut endgames_table = Self::new(max_stones);
        endgames_table.table = table_data
            .into_iter()
            .map(|byte| TableEntry {
                evaluation: byte as i8,
            })
            .collect();
        endgames_table.current_stones = max_stones;
        endgames_table.stones_in_play = MancalaGameState::TOTAL_STONES;
        println!("Endgames table with {} stones", max_stones);
        endgames_table
    }
}

impl EndgamesTable<MancalaGameState> for MancalaEndgamesTable {
    fn calculate_endgames(&mut self) {
        let batch_size = 1000;
        let mut offset = 0;

        let rt = tokio::runtime::Runtime::new().unwrap();

        for num_stones in 0..(self.max_stones + 1) {
            // keep track of this so we can do lookups
            self.stones_in_play = num_stones;

            let num_games = Self::num_game_states_full_board(num_stones) as usize;

            // create memory
            let table = Self::get_table_memory(num_stones);

            // lock table and get slice
            let mut data = table.lock().unwrap();
            let mut subtable = &mut data[..];

            // creates a scope which won't exit until all spawned futures have exited
            // this allows the lifetime of table to not be static
            tokio_scoped::scoped(rt.handle()).scope(|scope| {
                // must reference self here to avoid copying self into each future
                let endgames_table = &self;
                let mut batch: &mut [TableEntry];

                // integer division trick to get the number of batches, including the possibly smaller final batch
                let num_batches = (num_games + batch_size - 1) / batch_size;

                for batch_number in 0..num_batches {
                    // handle final batch being possibly smaller
                    let current_batch_size;
                    if batch_number == num_batches - 1 && num_games % batch_size != 0 {
                        current_batch_size = num_games % batch_size;
                    } else {
                        current_batch_size = batch_size;
                    }

                    // split off a batch of games to analyse and memory to store the result
                    (batch, subtable) = subtable.split_at_mut(current_batch_size);

                    // Use the scope to spawn the future.
                    scope.spawn(async move {
                        let mut game_state;
                        // analyse each game state in the batch and store the result in the table
                        for batch_index in 0..current_batch_size {
                            let game_index = offset + batch_number * batch_size + batch_index;
                            game_state = endgames_table.get_game_state(game_index, num_stones);
                            game_state.handle_game_over();
                            batch[batch_index] = TableEntry {
                                evaluation: Solver::alphabeta_no_depth_limit(
                                    &game_state,
                                    i32::MIN,
                                    i32::MAX,
                                    *endgames_table,
                                ) as i8,
                            };
                            // this is used for debugging
                            //batch[batch_index].evaluation += 1;
                            // batch[batch_index] = TableEntry {
                            //     evaluation: game_index as i32,
                            // };
                        }
                    });
                }
            }); // all spawned futures await here

            offset += num_games;
            self.current_stones = num_stones;

            // clone data (need to unlock and relock)
            drop(data);
            let data_to_clone = table.lock().unwrap();
            let subtable_to_clone = &data_to_clone[..];
            self.table.extend_from_slice(subtable_to_clone);
            //println!("{:#?}", self.table);
        }

        // this is an invalid game state but I want to handle it properly
        // handle_game_over() doesn't handle this properly so need to set manually
        self.table[0].evaluation = 0;
        assert!(self.current_stones == self.max_stones);
        self.stones_in_play = MancalaGameState::TOTAL_STONES;
    }

    fn lookup(&self, game_state: &MancalaGameState) -> Option<i32> {
        self.get_value(game_state)
    }
}

impl MancalaEndgamesTable {
    pub fn new(max_stones: u32) -> MancalaEndgamesTable {
        let capacity = Self::total_num_game_states_full_board(max_stones) as usize;

        let mut endgames_table = MancalaEndgamesTable {
            cache: Vec::with_capacity(max_stones as usize + 1),
            table: Vec::with_capacity(capacity),
            max_stones: max_stones,
            current_stones: 0,
            stones_in_play: 0,
        };
        endgames_table.set_cache(max_stones as usize);

        endgames_table
    }

    fn set_cache(&mut self, max_stones: usize) {
        for num_stones in 0..(max_stones + 1) {
            self.cache.push([0; MancalaGameState::PITS_NO_STORES + 2]);

            for num_pits in 1..(MancalaGameState::PITS_NO_STORES + 2) {
                self.cache[num_stones][num_pits] =
                    Self::num_game_states(num_stones as u32, num_pits as u32) as usize;
            }
        }
    }

    fn print_cache(&self) {
        println!("print_cache");
        for i in 0..(self.max_stones + 1) {
            println!("{}: {:?}", i, self.cache[i as usize]);
        }
    }

    fn get_index(&self, game_state: &MancalaGameState, remaining_stones: u32) -> usize {
        if remaining_stones == 0 {
            return 0;
        }

        let mut remaining = remaining_stones as usize;
        let mut index = self.cache[remaining - 1][MancalaGameState::PITS_NO_STORES + 1];

        // if player 1
        if game_state.turn {
            for pit_index in 0..MancalaGameState::PITS_PER_SIDE {
                remaining -= game_state.pits[pit_index] as usize;
                if remaining == 0 {
                    return index;
                }

                let remaining_pits = MancalaGameState::PITS_NO_STORES - pit_index - 1;
                // need to skip past all combinations of (remaining - 1) *or fewer* so need to add 1 to remaining pits
                index += self.cache[remaining - 1][remaining_pits + 1];
            }

            // don't index the final pit given we know how many stones are in play
            for pit_index in 0..(MancalaGameState::PITS_PER_SIDE - 1) {
                remaining -=
                    game_state.pits[pit_index + MancalaGameState::PLAYER_1_STORE + 1] as usize;
                if remaining == 0 {
                    return index;
                }

                let remaining_pits = MancalaGameState::PITS_PER_SIDE - pit_index - 1;
                index += self.cache[remaining - 1][remaining_pits + 1];
            }
        }
        // if player 2
        else {
            for pit_index in 0..(MancalaGameState::PITS_PER_SIDE) {
                remaining -=
                    game_state.pits[pit_index + MancalaGameState::PLAYER_1_STORE + 1] as usize;
                if remaining == 0 {
                    return index;
                }

                let remaining_pits = MancalaGameState::PITS_NO_STORES - pit_index - 1;
                index += self.cache[remaining - 1][remaining_pits + 1];
            }

            for pit_index in 0..(MancalaGameState::PLAYER_1_STORE - 1) {
                remaining -= game_state.pits[pit_index] as usize;
                if remaining == 0 {
                    return index;
                }

                let remaining_pits = MancalaGameState::PITS_PER_SIDE - pit_index - 1;
                index += self.cache[remaining - 1][remaining_pits + 1];
            }
        }

        index
    }

    pub fn get_value(&self, game_state: &MancalaGameState) -> Option<i32> {
        let remaining_stones = self.stones_in_play
            - game_state.pits[MancalaGameState::PLAYER_1_STORE]
            - game_state.pits[MancalaGameState::PLAYER_2_STORE];
        //println!("{:?}", game_state);
        //println!("remaining_stones: {}, stones_in_play: {}, current_stones: {}", remaining_stones, self.stones_in_play, self.current_stones);
        let result;
        if self.current_stones < remaining_stones {
            result = None;
        } else {
            let index = self.get_index(game_state, remaining_stones);
            let eval = self.table[index].evaluation;
            let player_eval;
            if game_state.turn {
                player_eval = eval;
            } else {
                player_eval = -1 * eval;
            }
            result = Some(
                (player_eval + game_state.pits[MancalaGameState::PLAYER_1_STORE] as i8
                    - game_state.pits[MancalaGameState::PLAYER_2_STORE] as i8)
                    as i32,
            );
        }

        result
    }

    fn get_pit_index(&self, index: usize) -> usize {
        return index + index / MancalaGameState::PLAYER_1_STORE;
    }

    // this function can be optimised in many ways
    // could do binary search
    // could just call this function less by having a get_next_game_state function
    // using the cache less will speed up
    fn get_game_state(&self, index: usize, total_num_stones: u32) -> MancalaGameState {
        if total_num_stones == 0 {
            return MancalaGameState::new();
        }

        let mut remaining_stones = total_num_stones;
        let mut index_guess = Self::total_num_game_states_full_board(total_num_stones - 1) as usize;
        let mut game_state = MancalaGameState::new();

        // for each pit
        for pit_index in 0..MancalaGameState::PITS_NO_STORES {
            // get the number of stones which is only just shy of the required index
            // index increases as number of stones decreases so need to loop backwards
            // index is always 0 when the number of stones is the remaining stones
            let mut next_num_stones = 0;
            for num_stones in (0..(remaining_stones + 1)).rev() {
                // generate a new index guess
                let new_remaining = remaining_stones - num_stones;
                let new_index_guess;
                if new_remaining == 0 {
                    new_index_guess = index_guess;
                } else {
                    new_index_guess = index_guess
                        + self.cache[new_remaining as usize - 1]
                            [MancalaGameState::PITS_NO_STORES - pit_index];
                }

                // if the new index guess is correct, or has over corrected
                if new_index_guess >= index {
                    // if the guess is correct
                    if new_index_guess == index {
                        game_state.pits[self.get_pit_index(pit_index)] = num_stones;
                        if new_remaining != 0 {
                            game_state.pits[self.get_pit_index(pit_index + 1)] = new_remaining;
                        }

                        return game_state;
                    }
                    // if the guess over-corrected
                    else {
                        next_num_stones = num_stones + 1;
                        break;
                    }
                }
            }

            // can get to this point either via over correcting or if 0 is the correct number of stones
            game_state.pits[self.get_pit_index(pit_index)] = next_num_stones;
            remaining_stones -= next_num_stones;
            if remaining_stones == 0 {
                return game_state;
            }
            index_guess += self.cache[remaining_stones as usize - 1]
                [MancalaGameState::PITS_NO_STORES - pit_index];
        }

        game_state
    }

    // the number of game states for a fixed number of stones is a stars and bars combinatorics problem
    // the number of stars (n) is the number of stones
    // the number of bars (k) is the number of pits - 1 (a bar would be between each pit)
    // the formula for stars and bars is then: total_combinations = binomial(n + k - 1, k - 1)
    fn num_game_states(num_stones: u32, num_pits: u32) -> u128 {
        if num_stones == 0 {
            return 1;
        }
        let n = num_stones as u128;
        let k = num_pits as u128;
        //binomial(n + k - 1, k - 1)
        binomial(n + k - 1, k - 1)
    }

    fn num_game_states_full_board(num_stones: u32) -> u128 {
        let num_pits = MancalaGameState::PITS_NO_STORES as u32;
        Self::num_game_states(num_stones, num_pits)
    }

    // the number of game states for a number of stones *or fewer*
    // this also includes 0 and 1 stone games
    // this is the same as above but add one extra pit for out of play stones
    fn total_num_game_states_full_board(num_stones: u32) -> u128 {
        let num_pits = MancalaGameState::PITS_NO_STORES as u32 + 1;
        Self::num_game_states(num_stones, num_pits)
    }

    // helper function
    fn get_table_memory(num_stones: u32) -> Arc<Mutex<Vec<TableEntry>>> {
        let num_games = Self::num_game_states_full_board(num_stones) as usize;
        let reserved = std::mem::size_of::<TableEntry>() * num_games;
        let megabytes_int = reserved / 1000000;
        let megabytes_remain = (reserved % 1000000) / 1000;
        println!(
            "Reserving: {:>6}.{:<3} MB for {:>2} stones. Number of games: {}",
            megabytes_int, megabytes_remain, num_stones, num_games
        );

        Arc::new(Mutex::new(
            std::iter::repeat(TableEntry { evaluation: 0 })
                .take(num_games)
                .collect(),
        ))
    }

    ////////////////////// Test code /////////////////////////////////////////////

    pub fn all_positions(
        &mut self,
        game_state: &mut MancalaGameState,
        remaining_stones: u32,
        pit_index: usize,
        total_stones: u32,
    ) {
        if pit_index == 12 {
            game_state.pits[12] = remaining_stones;

            // println!("state: {:?}", game_state);

            let index = self.get_index(game_state, total_stones);

            let state_guess = self.get_game_state(index, total_stones);
            let index_guess = self.get_index(&state_guess, total_stones);
            self.table[index].evaluation += 1;

            if index_guess != index {
                println!("index: {}, state: {:?}", index, game_state);
                println!("guess: {}, guess: {:?}", index_guess, state_guess);
            }
        } else if pit_index == 6 {
            self.all_positions(game_state, remaining_stones, pit_index + 1, total_stones);
        } else {
            for i in 0..(remaining_stones + 1) {
                game_state.pits[pit_index] = i;
                self.all_positions(
                    game_state,
                    remaining_stones - i,
                    pit_index + 1,
                    total_stones,
                );
            }
        }
    }

    pub fn test_index_validity(max_stones: u32) {
        // create the table
        let mut endgames_table = MancalaEndgamesTable::new(max_stones);
        endgames_table.print_cache();

        let total_num_games = Self::total_num_game_states_full_board(max_stones) as usize;
        endgames_table.table = std::iter::repeat(TableEntry { evaluation: 0 })
            .take(total_num_games)
            .collect();

        for i in 0..(max_stones + 1) {
            let num_games = Self::total_num_game_states_full_board(i);
            let num_games_guess =
                endgames_table.cache[i as usize][MancalaGameState::PITS_NO_STORES + 1];
            println!(
                "num_games: {}, num_games_guess: {}",
                num_games, num_games_guess
            );
        }

        // test get_index and get_game_state are inverses of each other
        for i in (0..(max_stones + 1)).rev() {
            endgames_table.all_positions(&mut MancalaGameState::new(), i, 0, i);
        }

        // test each entry was reached exactly once (shows that get_index and get_game_state are 1 to 1 inverses of each other)
        for (i, entry) in endgames_table.table.iter().enumerate() {
            if entry.evaluation != 1 {
                println!("{}", i);
            }
        }
    }

    // change the calculate_endgames result to: batch[i].evaluation += 1;
    pub fn test_table_validity(max_stones: u32) {
        // create the table
        let mut endgames_table = MancalaEndgamesTable::new(max_stones);
        endgames_table.print_cache();

        endgames_table.calculate_endgames();

        println!("Endgame table size: {}", endgames_table.table.len());

        //let mut offset = 0;

        // this is for debugging
        // for num_stones in 0..=max_stones {
        //     let num_games = Self::num_game_states_full_board(num_stones) as usize;
        //     for index in 0..num_games {
        //         let game_state = endgames_table.get_game_state(offset + index, num_stones);
        //         let eval = endgames_table.table[index + offset].evaluation;
        //         println!("eval: {:>4}, state: {:?}", eval, game_state);
        //     }

        //     offset += num_games;
        // }

        // this is for testing (set eval to 1 in calculate_endgames)
        let total_num_games = Self::total_num_game_states_full_board(max_stones) as usize;
        for i in 0..total_num_games {
            if endgames_table.table[i].evaluation != 1 {
                // this may print 0 as I am handling that separately
                println!("Index: {}, eval: {}", i, endgames_table.table[i].evaluation);
            }
        }

        // this is for testing (set eval to index in calculate_endgames)
        // offset = 0;
        // for num_stones in 0..=max_stones {
        //     let num_games = Self::num_game_states_full_board(num_stones) as usize;
        //     for index in 0..num_games {

        //         let eval = endgames_table.table[index + offset].evaluation;
        //         if eval as usize != index + offset {
        //             let game_state = endgames_table.get_game_state(offset + index, num_stones);
        //             println!("eval: {}, total_index: {}, index: {}, offset: {}, state: {:?}", eval, index + offset, index, offset, game_state);
        //         }
        //     }

        //     offset += num_games;
        // }
    }

    pub fn test_table_accuracy(max_stones: u32) {
        // create the table
        let mut endgames_table = MancalaEndgamesTable::new(max_stones);
        endgames_table.calculate_endgames();

        let mut blank_table = MancalaEndgamesTable::new(0);
        blank_table.calculate_endgames();

        let mut offset = 0;

        // this is for debugging
        for num_stones in 0..=max_stones {
            let num_games = Self::num_game_states_full_board(num_stones) as usize;
            endgames_table.stones_in_play = num_stones;
            for index in 0..num_games {
                let mut game_state = endgames_table.get_game_state(offset + index, num_stones);
                game_state.turn = true;
                let table_eval = endgames_table.lookup(&game_state).expect(":O");
                game_state.handle_game_over();
                if !game_state.game_over {
                    let eval = Solver::alphabeta_no_depth_limit(
                        &game_state,
                        i32::MIN,
                        i32::MAX,
                        &blank_table,
                    );

                    if eval != table_eval {
                        println!("{:?}", game_state);
                        println!("eval: {}, table_eval: {}", eval, table_eval);
                    }
                }
            }

            offset += num_games;
        }

        //println!("{:#?}", endgames_table);
    }
}
