
/*
enum TranspositionTableEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}
struct TranspositionTableEntry {
    evaluation: i32,
    flag: TranspositionTableEntryFlag,
    bestMove: Option<usize>,
    depth: u32,
}

fn generate_hash(game_state: &Position) -> u64 {
    let mut bits: u64 = 1;
    for i in 0..12 {
        bits <<= (game_state.slots[i] + 1);
        bits += 1;
    }

    bits <<= game_state.slots[12] + 1;
    match game_state.turn {
        Player::One => (),
        Player::Two => bits += 1,
    }

    return bits;
}

fn alphaBetaWithMemory(
    transpositionTable: &mut HashMap<u64, TranspositionTableEntry>,
    position: &Position, 
    depth: u32, 
    mut alpha: i32, 
    mut beta: i32) -> i32 {
    //position.print();

    let mut bestMove: Option<usize>;
    let alphaOrig = alpha;
    let betaOrig = beta;
    let key = generate_hash(position);

    match transpositionTable.get(&key) {
        Some(&ref entry) => {
            if entry.depth > depth {
                match entry.flag {
                    TranspositionTableEntryFlag::Exact => {
                        //println!("Exact TT entry: {}", entry.evaluation);
                        return entry.evaluation;
                    },
                    TranspositionTableEntryFlag::LowerBound => {
                        //println!("LowerBound TT entry: {}, {}", alpha, entry.evaluation);
                        alpha = max(alpha, entry.evaluation);
                    },
                    TranspositionTableEntryFlag::UpperBound => {
                        //println!("UpperBound TT entry: {}, {}", beta, entry.evaluation);
                        beta = min(beta, entry.evaluation);
                    },
                }
                // could alpha ever be greater than beta?
                if alpha >= beta {
                    // println!("alpha >= beta: {} >= {}", alpha, beta);
                    return entry.evaluation
                }
            }
            bestMove = entry.bestMove;
        },
        None => bestMove = None,
    }
    // match bestMove {
    //     Some(m) => println!("Best move: {}", m),
    //     None => println!("Best move: None"),
    // }

    if depth == 0 || position.game_over {
        let evaluation = position.evaluate();
        //println!("Leaf node. Evaluation: {}", evaluation);
        return evaluation;
    }

    // default initialisation
    let mut child_position = Position {
        slots: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        turn: Player::One,
        game_over: false,
    };

    let mut value: i32;

    match position.turn {
        Player::One => {
            value = -1 * INF;
            
            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_one(position, &mut move_order, bestMove);
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
                let evaluation = alphaBetaWithMemory(transpositionTable, &child_position, depth - 1, alpha, beta);
                if evaluation > value {
                    value = evaluation;
                    bestMove = Some(move_index);
                }

                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }
        }
        Player::Two => {
            value = INF;

            let mut move_order: [usize; 6] = [0; 6];
            get_move_order_player_two(position, &mut move_order, bestMove);
            for move_index in move_order {
                if position.slots[move_index] == 0 {
                    continue
                }
                // make each move and store each result in child_position
                player_two_make_move(&position, &mut child_position, move_index);
                // recursively evaluate the child position
                let evaluation = alphaBetaWithMemory(transpositionTable, &child_position, depth - 1, alpha, beta);
                if evaluation < value {
                    value = evaluation;
                    bestMove = Some(move_index);
                }

                beta = min(beta, value);

                if value <= alpha {
                    break;
                }
            }
        }
    }

    // if we set alpha and beta before breaking then we need to save the original alpha and beta
    // if we break before setting them then don't need to save
    // a value less than alpha is either:
    // Player1: has explored every possibility and this is the best there is
    // Player2: breaks after finding a value a value less than alpha, could be something better (lower)
    // vice-versa for beta
    // how can we trust that "this is the best there is" if values returned are bounds rather than exact
    // we can't trust exact values, but a lower bound that could be lower will only ever make the lower bound not as low as it could be and vice vera for the upper bound
    // a lower bound returning that isn't low enough will never affect the upper bound
    let flag: TranspositionTableEntryFlag;
    if value <= alphaOrig {
        flag = TranspositionTableEntryFlag::UpperBound
    }
    else if value >= betaOrig {
        flag = TranspositionTableEntryFlag::LowerBound
    }
    else {
        flag = TranspositionTableEntryFlag::Exact
    };

    transpositionTable.insert(key, TranspositionTableEntry { evaluation: value, flag: flag, bestMove: bestMove, depth: depth });

    return value;
}
*/