use std::fmt::Debug;
use std::hash::Hash;

// GameState should encode all the specifics of a game and none of the specifics of the solver algorithm
pub trait GameState<T>: PartialEq + Eq + Hash + Clone + Debug {
    fn get_children<'a>(&self, children_cache: &'a mut Vec<T>) -> &'a Vec<T>;
    fn get_children_cache(&self) -> Vec<T>;
    fn is_game_over(&self) -> bool;
    fn heuristic(&self) -> i32;
    fn is_maximising_player(&self) -> bool;
}
