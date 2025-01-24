
use std::fs;
use std::time::Instant;
use std::io::Write;
use std::io::Read;
use std::thread;
//use std::sync::{Arc, Mutex};

use crate::mancala;
use crate::mancala::Position;

const MAX_STONES: usize = 10;

#[derive(Clone)]
pub struct DatabaseEntry {
    pub player_1_evaluation: i8,
    pub player_2_evaluation: i8,
}
pub struct Endgame {
    pub cache: [[usize; MAX_STONES]; 12],
    pub database: Vec<DatabaseEntry>,
    pub max_stones: u32,
    pub current_stones: u32,
}

fn fact(n: usize) -> u128 {
    return (1..((n as u128)+1)).product();
}

fn test2(database: &mut Vec<DatabaseEntry>, position: &mut mancala::Position) {
    // //position.print();
    // let index = self.get_index(position);
    // //println!("Index: {}", index);

    // let mut player_one_stones = 0;
    // for i in 0..6 {
    //     player_one_stones += position.slots[i];
    // }

    // let mut player_two_stones = 0;
    // for i in 7..13 {
    //     player_two_stones += position.slots[i];
    // }

    // if player_one_stones == 0 || player_two_stones == 0 {
    //     let evaluation: i32 = position.slots[6]  as i32 + player_one_stones as i32 
    //                         - position.slots[13] as i32 - player_two_stones as i32;
    //     self.database[index].player_1_evaluation = evaluation as i8;
    //     self.database[index].player_2_evaluation = evaluation as i8;
    // }
    // else {
    //     position.turn = mancala::Player::One;
    //     self.database[index].player_1_evaluation = mancala::minimax_no_depth(position, self, -100, 100) as i8;

    //     position.turn = mancala::Player::Two;
    //     self.database[index].player_2_evaluation = mancala::minimax_no_depth(position, self, -100, 100) as i8;
    // }
}

impl Endgame {

    pub fn new() -> Endgame {
        let mut endgame = Endgame {
            cache: [[0; MAX_STONES]; 12],
            max_stones: 0,
            database: Vec::new(),
            current_stones: 0,
        };
        endgame.set_cache(MAX_STONES);
        let num = (fact(12+MAX_STONES) / (fact(12) * fact(MAX_STONES))) as usize;
        endgame.database.resize(num, DatabaseEntry { player_1_evaluation: 0, player_2_evaluation: 0 });

        return endgame;
    }

    fn set_cache(&mut self, max_stones: usize) {
        println!("Calculating database cache...");
        let now = Instant::now();
        
        for depth in 0..12 {
            for stones in 0..max_stones {
                self.cache[depth][stones] = (fact(depth + stones + 1) / (fact(depth + 1) * fact(stones))) as usize;
            }
        }

        let time = now.elapsed().as_millis();
        println!("Done calculating database cache: {}ms", time);
    }

    #[allow(dead_code)]
    fn print_cache(&self) {
        println!("Database cache:");
        for i in 0..12 {
            for j in 0..MAX_STONES {
                print!("{} ", self.cache[i][j])
            }
            println!("");
        }
    }

    fn get_index(&self, position: &mancala::Position) -> usize {
        let mut remaining = MAX_STONES;
        let mut count = 0;
        for i in 0..6 {
            let num_stones = position.slots[i] as usize;
            if num_stones == remaining {
                return count;
            }

            remaining -= num_stones;
            count += self.cache[11 - i][remaining - 1];
        }

        for i in 7..13 {
            let num_stones = position.slots[i] as usize;
            //println!("{}, {}", num_stones, remaining);
            if num_stones == remaining {
                return count;
            }

            remaining -= num_stones;
            count += self.cache[12 - i][remaining - 1];
        }

        return count;
    }

    pub fn get_value(&self, position: &mancala::Position) -> i32 {
        let index = self.get_index(position);
        match position.turn {
            mancala::Player::One => return self.database[index].player_1_evaluation as i32,
            mancala::Player::Two => return self.database[index].player_2_evaluation as i32,
        }
    }

    fn test1(&mut self, position: &mut mancala::Position, remaining_stones: u32, slot: usize, job_queue: &mut Vec<Position>) {
        
        if slot == 12 {
            position.slots[slot] = remaining_stones;
            //position.print();
            //self.test2(position);
            job_queue.push(*position);
        }
        else {
            let new_slot: usize;
            if slot == 5 {
                new_slot = 7;
            }
            else {
                new_slot = slot + 1;
            }

            for i in 0..(remaining_stones + 1) {
                position.slots[slot] = i;
                //position.print();
                self.test1(position, remaining_stones - i, new_slot, job_queue);
            }
        }
    }

    pub fn test(&mut self) {
        println!("Calculating endgames...");
        let now = Instant::now();

        for i in 0..(MAX_STONES as u32 + 1) {
            let loop_now = Instant::now();

            let mut job_queue: Vec<Position> = Vec::new();

            let mut position = mancala::Position {
                slots: [0; 14],
                turn: mancala::Player::One,
                game_over: false,
            };
            self.current_stones = i;
            self.test1(&mut position, i, 0, &mut job_queue);

            let num_jobs = job_queue.len();
            let num_threads = 3;
            let jobs_each = num_jobs / num_threads;

            let mut database: Vec<DatabaseEntry> = Vec::new();

            let mut handles = vec![];

            for i in 0..num_threads {
                let handle = thread::spawn( move || {
                        for j in (i * num_threads)..((i + 1) * num_threads) {
                            let mut position_clone = position.clone();
                            test2(&mut database, &mut position_clone);
                        }
                    }
                );
                handles.push(handle);
            }


            for handle in handles {
                handle.join().unwrap();
            }

            for position in job_queue {
                let mut position_clone = position.clone();
                test2(&mut self.database, &mut position_clone);
            }
            
            self.max_stones = i;

            let loop_time = loop_now.elapsed().as_millis();
            println!("max stones: {}, time: {}ms", self.max_stones, loop_time);
        }

        let time = now.elapsed().as_millis();
        println!("Done calculating endgames: {}ms", time);
    }

    pub fn to_file(&self, filename: &str) {
        println!("Writing endgame database to file...");
        let now = Instant::now();

        let mut file = fs::OpenOptions::new()
        .create(true) // To create a new file
        .write(true)
        .open(filename)
        .expect("");

        let mut buffer = Vec::new();
        buffer.resize(self.database.len() * 2 + 1, 0);
        buffer[0] = self.max_stones as u8;

        for i in 0..self.database.len() {
            buffer[i * 2 + 1] = (self.database[i].player_1_evaluation + 64) as u8;
            buffer[i * 2 + 2] = (self.database[i].player_2_evaluation + 64) as u8;
        }

        file.write(&buffer).expect("");

        let time = now.elapsed().as_millis();
        println!("Done writing endgame database to file: {}ms", time);
    }

    pub fn from_file(&mut self, filename: &str) {
        println!("Reading endgame database from file...");
        let now = Instant::now();

        let mut file = fs::OpenOptions::new()
        .create(false) // To create a new file
        .read(true)
        .open(filename)
        .expect("");

        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).expect("");

        self.max_stones = buffer[0] as u32;
        let max_stones = self.max_stones as usize;
        let num = (fact(12+max_stones) / (fact(12) * fact(max_stones))) as usize;
        if num * 2 + 1 != buffer.len() {
            panic!("");
        }
        self.set_cache(max_stones);
        self.database.resize(num, DatabaseEntry { player_1_evaluation: 0, player_2_evaluation: 0 });

        for i in 0..num {
            self.database[i].player_1_evaluation = buffer[i * 2 + 1] as i8 - 64;
            self.database[i].player_2_evaluation = buffer[i * 2 + 2] as i8 - 64;
        }

        let time = now.elapsed().as_millis();
        println!("Done reading endgame database from file: {}ms", time);
    }
}
