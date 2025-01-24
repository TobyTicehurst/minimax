//TODO delete me
#![allow(warnings)]

// mod mancala;
// mod endgame;

use num_integer::binomial;
use std::sync::Mutex;

const MAX: usize = 64; 
static mut lookup: [u64; MAX*MAX] = [0; MAX*MAX]; 

fn bin(n: u64, k: u64) -> u64 {
    if n > 64 || k > n {
        panic!("{}, {}", n, k);
    }
    unsafe {
        return lookup[(n as usize * MAX) + k as usize];
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
pub struct DatabaseEntry {
    pub player_1_evaluation: u32,
    pub player_2_evaluation: i8,
}

pub struct EndgameDatabase {
    // database for each stone
    database: Vec<DatabaseEntry>,
    // indices into the above database, see 'get' function
    indices: Vec<usize>,
}

impl EndgameDatabase {
    pub fn new(max_stones: u32) -> EndgameDatabase {
        // TODO make num_slots a compile time constant
        let mut sum = 0;
        let mut indices = vec![0; max_stones as usize + 2];
        for i in 2..(max_stones + 2) {
            indices[i as usize] = sum;
            sum += get_number_positions_partial(i, 12) as usize;;
        }

        EndgameDatabase {
            database: vec![DatabaseEntry {
                player_1_evaluation: 0,
                player_2_evaluation: 0,
            }; sum],
            indices: indices.clone(),
        }
    }

    pub fn get(&self, num_stones: u32, index: usize) -> &DatabaseEntry {
        if self.indices[num_stones as usize] + index < self.indices[num_stones as usize + 1] {
            return &self.database[self.indices[num_stones as usize] + index];
        }
        else {
            panic!("range error")
        }
    }

    pub fn calculate_endgames(&mut self, num_stones: u32, num_threads: u32) {
        let slots = 12;
        let batch_size = 2;

        // for each stone
        for i in 2..(num_stones + 1) {
            println!("Number of stones: {i}");

            let num_positions = get_number_positions_partial(i, slots) as usize;
            println!("Number of positions: {num_positions}");

            // create the new database (could assign max memory once but would lose bounds checking)
            let mut new_database = vec![DatabaseEntry {
                player_1_evaluation: 0,
                player_2_evaluation: 0,
            }; num_positions];
            // this mutex is a good point to optimise but going to leave it for now in favour of getting something working
            // could only lock when a batch finishes or just accept some unsafe code
            let new_database_arc = Arc::new(Mutex::new(new_database));

            // atomic for counting how many positions have been handled
            let arc_atomic = Arc::new(AtomicUsize::new(0));

            // self is immutably borrowed for the duration of this scope
            thread::scope(|scope| {
                println!("Spawning {num_threads} threads");
                for _ in 0..num_threads {
                    scope.spawn(|| {
                        // get next position and add 1 to mark as processing
                        let mut position_index = arc_atomic.fetch_add(batch_size, Ordering::SeqCst);
                        while position_index < num_positions as usize {
                            
                            self.analyse_batch(position_index, batch_size as u32, i, &new_database_arc);
                            //println!("{position_index}");
                            position_index = arc_atomic.fetch_add(batch_size, Ordering::SeqCst);
                        }
                    });
                }
            });
            println!("Threads joined");
            // we can now mutate self

            // consume the lock
            let lock = Arc::into_inner(new_database_arc).unwrap();
            let database = Mutex::into_inner(lock).unwrap();

            // copy the database over
            let offset = self.indices[i as usize];
            for i in 0..database.len() {
                self.database[offset + i] = database[i].clone();
            }
        }
    }

    fn analyse_batch(&self, start_index: usize, num_positions: u32, max_stones: u32, database: &Arc<Mutex<Vec<DatabaseEntry>>>) {
        let mut position = vec![0; 12];
        position_from_index(start_index as u32, &mut position, max_stones);
        let mut index = start_index;
        let final_index = start_index + num_positions as usize;
        if index < final_index {
            loop  {
                //println!("{:?}", position);
                let mut data = database.lock().unwrap();
                (*data)[index].player_1_evaluation = index_from_position(&position, max_stones);
                (*data)[index].player_2_evaluation = max_stones as i8;
                if index == 364 {
                    println!("test: {:?}", position);
                }
                index += 1;
                if !(index < final_index && get_next_position(&mut position)) {
                    break;
                }
            }
        }
    }
    
}

// get the number of positions which have this number of stones and slots (for table with n or fewer stones)
fn get_number_positions_full(stones: u32, slots: u32) -> u32 {
    // TODO
    let num_positions = binomial(stones + slots, slots);
    println!("num positions: {}", num_positions);
    return num_positions;
}

// get the number of positions which have this number of stones and slots (for table with exactly n stones)
fn get_number_positions_partial(stones: u32, slots: u32) -> u32 {
    // TODO
    binomial(stones + slots - 1, slots - 1)
}


fn position_from_index(index: u32, position: &mut Vec<u32>, max_stones: u32) {
    // clear position
    let length = position.len();
    position.clear();
    position.resize(length, 0);

    //println!("index: {}", index);

    let mut position_index = 0;
    let mut min_guess = 0;
    let mut max_guess = max_stones + 1;
    let mut guess = 0;
    let mut remaining_stones = max_stones;

    // search through all but the final index
    while position_index < length - 1 {
        // binary search
        guess = (max_guess + min_guess) / 2;
        position[position_index] = guess;

        // put any remaining stones in the last slot
        position[length - 1] = remaining_stones - guess;

        //println!("guess: {guess}, ({min_guess}-{max_guess})");
        //println!("{:?}", position);

        // get the index of this guessed position
        let index_guess = index_from_position(&position, max_stones);
        //println!("index_guess: {index_guess} (index: {index})");

        
        if index_guess == index {
            // index is correct
            //println!("index: {}", index);
            return;
        }
        else if index_guess > index {
            // index_guess is too large, need to increase the guess
            min_guess = guess;
        }
        else {
            // index_guess is too small, need to reduce the guess
            max_guess = guess;
        }

        if max_guess - min_guess == 1 {
            // if we have narrowed our search to here then the answer is between min and max, 
            // set this position to min and move to the next index
            //println!("Setting position index {position_index} to {min_guess}");
            position[position_index] = min_guess;
            position_index += 1;
            remaining_stones -= min_guess;
            max_guess = remaining_stones + 1;
            min_guess = 0;
        }
    }

    // put any remaining stones in the final slot
    position[length - 1] = remaining_stones;

    // double check the result
    if index_from_position(&position, max_stones) != index {
        println!("{:?}", position);
        panic!("index != guess");
    }
}

fn index_from_position(position: &Vec<u32>, max_stones: u32) -> u32 {
    //println!("index_from_position");
    let mut remaining_n: u32 = max_stones;
    let mut remaining_depth = position.len() as u32;

    let mut index: u32 = 0;

    for i in position {
        if remaining_n == *i {
            break;
        }
        remaining_depth -= 1;
        remaining_n -= i;
        let n = (remaining_depth + remaining_n - 1) as u64;
        let k = (remaining_depth) as u64;
        index += bin(n, k) as u32;
        
        //println!("n: {n}, k: {k}, remaining_depth: {remaining_depth}, remaining_n: {remaining_n}, index: {index}");
    }

    return index;
}

// returns false if there was no change
fn get_next_position(position: &mut Vec<u32>) -> bool {
    let len = position.len();
    let mut changed = false;

    // work backwards through list starting from list[-2]
    for j in 2..(len + 1) {
        // if this element is non-zero
        if position[len - j] != 0 {
            changed = true;
            // sub one from this element and move it to the next
            position[len - j] -= 1;
            position[len - j + 1] += 1;

            // last element should be made 0, unless we just added to it (ie if j = 2)
            let end = position[len - 1];
            position[len - j + 1] += end;
            position[len - 1] -= end;

            break;
        }
    }

    return changed;
}

fn threading_test(mut data: Vec<usize>) -> Vec<usize> {
    // want the vec of vecs to be an array of slices
    // take some mutable data
    //let mut data = vec![0; 1];
    let mut data_2: Vec<Vec<u32>> = vec![vec![0; 1]; 1];
    // let mut data = vec![0;3];
    for i in 0..2 {
        // move ownership of the data in to an Arc
        let mut arc_data = Arc::new(data);
        let mut arc_data_2 = Arc::new(data_2);
        let mut handles = vec![];

        // spawn and run all the threads with read only access to the data
        for _ in 0..3 {
            let arc_data = Arc::clone(&arc_data);
            let arc_data_2 = Arc::clone(&arc_data_2);
            let handle = thread::spawn(move || {
                // data isn't mutable here
                //arc_data[i] = 0;
                if arc_data[i] == i {
                    println!("yay");
                }

                if arc_data_2[i][0] == 0 {
                    println!("yay 2");
                }
            });
            handles.push(handle);
        }

        // join all the threads
        for handle in handles {
            handle.join().unwrap();
        }

        // data would not be mutable yet
        //data.push(i + 1);

        // move ownership of the data out of the Arc
        data = Arc::try_unwrap(arc_data).unwrap();
        // data is now mutable again
        data.push(i + 1);

        data_2 = Arc::try_unwrap(arc_data_2).unwrap();
        data_2.push(vec![0;1]);
    }

    println!("Result: {:?}", data);
    println!("Result: {:?}", data_2);

    return data;
}

fn all_positions(position: &mut Vec<u32>, depth: usize, remaining: u32, position_table: &mut Vec<[u32;12]>, ref_table: &mut Vec<u32>) {
    // if we have reached the last slot
    if depth == position.len() - 1 {
        position[depth] = remaining;
        // get index
        let max_stones = position.iter().sum();
        let index = index_from_position(&position, max_stones) as usize;
        // add to table
        ref_table[index] += 1;
        for i in 0..position.len() {
            position_table[index][i] = position[i];
        }
    }
    else {
        // for stones 0 to 'remaining' (inclusive)
        for i in 0..(remaining + 1) {
            position[depth] = i;
            all_positions(position, depth + 1, remaining - i, position_table, ref_table);
        }
        
    }

    // reset this slot (may not be needed)
    position[depth] = 0;
}

// for each position:
//  generate an index
//  store the position in an array at that index
// for each index
//  generate a position
//  get the position from the array and check it matches the generated one
fn check_indexing() {
    // go through each position and add it to the table
    for max_stones in 0..20 {
        println!("Max stones: {max_stones}");
        let num_slots = 12;
        let num_positions = get_number_positions_partial(max_stones, num_slots);

        let mut position = vec![0 as u32; num_slots as usize];
        let mut position_table = vec![[0 as u32; 12]; num_positions as usize];
        let mut ref_table = vec![0 as u32; num_positions as usize];
        all_positions(&mut position, 0 as usize, max_stones, &mut position_table, &mut ref_table);

        for i in 0..num_positions {
            position_from_index(i, &mut position, max_stones);
            let test_position = position_table[i as usize];
            let test_ref = ref_table[i as usize];

            // test every position is seen once
            if test_ref != 1 {
                println!("{i}: {test_ref}");
            }

            // test every index contains the correct position
            for i in 0..test_position.len() {
                if test_position[i] != position[i] {
                    println!("{i}: {:?}", test_position);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ThreadedData<'a> {
    pub data: Vec<u32>,
    pub slices: Vec<&'a [u32]>,
}

impl<'a> ThreadedData<'a> {
    pub fn threading_test_3(&mut self) {
        
        for i in 0..2 {
            // self is immutably borrowed for the duration of this scope
            thread::scope(|scope| {
                for _ in 0..3 {
                    scope.spawn(|| {
                        if self.slices[0][i] == 0 {
                            println!("yay 2");
                        }
                    });
                }
            });

            // we can now mutate self
            self.data.push(i as u32 + 1);
        }
    }
}

fn main() {

    for i in 0..MAX {
        for j in 0..MAX {
            unsafe {
                lookup[i * MAX + j] = binomial(i as u64, j as u64);
            }
        }
    }

    let num_threads = 1;
    let num_stones = 4;
    let num_slots = 12;
    let mut endgame_database = EndgameDatabase::new(num_stones);
    endgame_database.calculate_endgames(num_stones, num_threads);

    let mut offset = 0;
    for i in 0..(endgame_database.indices.len() - 1) {
        //println!("{}", endgame_database.indices[i]);
        for j in 0..endgame_database.indices[i] {
            let entry = &endgame_database.database[offset + j];
            if entry.player_1_evaluation != j as u32 || entry.player_2_evaluation != 1 {
                println!("{i}, {j}, {:?}", entry);
            }
        }
        offset = endgame_database.indices[i];
    }

    for e in endgame_database.database {
        //println!("{:?}", e);
    }

}
