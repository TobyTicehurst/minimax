use crate::minimax::GameState;

pub trait EndgamesTable<T: GameState<T>> {
    fn calculate_endgames(&mut self);
    fn lookup(&self, game_state: &T) -> Option<i32>;
}
