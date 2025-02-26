use crate::minimax::game_state::GameState;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TranspositionTable<T: GameState<T>> {
    data: HashMap<T, TranspositionTableElement>,
    pub max_depth: u32,
}

impl<T: GameState<T>> TranspositionTable<T> {
    pub fn new(max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::new(),
            max_depth,
        }
    }

    pub fn with_capacity(capacity: usize, max_depth: u32) -> TranspositionTable<T> {
        TranspositionTable::<T> {
            data: HashMap::with_capacity(capacity),
            max_depth,
        }
    }

    pub fn lookup(&self, game_state: &T) -> Option<&TranspositionTableElement> {
        self.data.get(game_state)
    }

    pub fn get(&mut self, game_state: &T) -> &mut TranspositionTableElement {
        // TODO avoid cloning here if possible
        self.data
            .entry(game_state.clone())
            .or_insert(TranspositionTableElement::default())
    }
}

#[derive(Clone, Debug)]
pub struct TranspositionTableElement {
    pub lower_bound: Bound,
    pub upper_bound: Bound,
}

#[derive(Clone, Debug)]
pub struct Bound {
    pub bound: i32,
    pub depth: u32,
}

impl TranspositionTableElement {
    fn default() -> Self {
        TranspositionTableElement {
            lower_bound: Bound {
                bound: i32::MIN,
                depth: 0,
            },
            upper_bound: Bound {
                bound: i32::MAX,
                depth: 0,
            },
        }
    }
}
