use std::cmp::max;
use std::cmp::min;
use std::time::Instant;


const inf: i32 = 100;

#[derive(Debug)]
enum Player {
    One,
    Two,
}

#[derive(Debug)]
struct Position {
    // number of stones in each slot
    slots: [i32; 14],
    // who's turn is it next
    turn: Player,
}

impl Position {
    fn print(&self) {
        println!("  {} {} {} {} {} {}  ", self.slots[12], self.slots[11], self.slots[10], self.slots[9], self.slots[8], self.slots[7]);
        println!("{}             {}", self.slots[13], self.slots[6]);
        println!("  {} {} {} {} {} {}  ", self.slots[0], self.slots[1], self.slots[2], self.slots[3], self.slots[4], self.slots[5]);
        println!("Turn: Player {:?}", self.turn);
        println!("Evaluation: {}", self.evaluate());
        println!("");
    }

    fn evaluate(&self) -> i32 {
        return self.slots[6] - self.slots[13];
    }
}

fn makeMove(position: &Position, childPosition: &mut Position, moveIndex: usize) {
    // deep copy position
    childPosition.slots = position.slots.clone();

    // take stones out of slot
    let mut numberStones = childPosition.slots[moveIndex];
    childPosition.slots[moveIndex] = 0;

    // get the opponent's goal for later
    let opponentGoal = match position.turn {
        Player::One => 13,
        Player::Two => 6,
    };

    // start here...
    let mut slotIndex = moveIndex;
    // ...and add stones 1 by 1 to slots
    while numberStones > 0 {
        numberStones -= 1;
        slotIndex += 1;
        // but skip the opponent's goal
        if slotIndex == opponentGoal {
            slotIndex += 1;
        }
        slotIndex %= 14;
        childPosition.slots[slotIndex] += 1;
    }

    // where did the final stone land?
    let finalSlotIndex = slotIndex;

    // get the player's goal for later
    match position.turn {
        Player::One => {
            if finalSlotIndex == 6 {
                childPosition.turn = Player::One;
            }
            else {
                childPosition.turn = Player::Two;
            }
        },
        Player::Two => {
            if finalSlotIndex == 13 {
                childPosition.turn = Player::Two;
            }
            else {
                childPosition.turn = Player::One;
            }
        },
    };
}

fn minimax(position: &Position, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {

    //position.print();

    if depth == 0 {
        return position.evaluate();
    }

    // default initialisation
    let mut childPosition = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
    };

    match position.turn {
        Player::One => {
            let mut value = -1 * inf;
            
            for moveIndex in 0..6 {
                // make each move and store each result in childPosition
                makeMove(&position, &mut childPosition, 5 - moveIndex);
                // recursively evaluate the child position
                value = max(minimax(&childPosition, depth - 1, alpha, beta), value);

                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }

            return value;
        }
        Player::Two => {
            let mut value = inf;
            for moveIndex in 0..6 {
                // make each move and store each result in childPosition
                makeMove(&position, &mut childPosition, 12 - moveIndex);
                // recursively evaluate the child position
                value = min(minimax(&childPosition, depth - 1, alpha, beta), value);

                beta = min(beta, value);

                if value <= alpha {
                    break;
                }
            }

            return value;
        }
    }
}

fn main() {
    println!("Hello, world!");
    let mut depth = 1;
    loop {
        let now = Instant::now();
   
        depth += 1;
        println!("Depth: {}", depth);
        let position = Position {
            slots: [4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4, 0],
            turn: Player::One,
        };
        let evaluation = minimax(&position, depth, -1 * inf, inf);
        println!("Evaluation: {}", evaluation);
        // it prints '2'
        println!("{}ms\n", now.elapsed().as_millis());
    }
    
}
