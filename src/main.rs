mod mancala;
mod minimax;

use mancala::MancalaGameState;
use minimax::MinimaxSolver;
use std::time::{Duration, SystemTime};
use tracing::info;
use tracing::level_filters::LevelFilter;

fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_ansi(false)
        .without_time()
        .init();

    for depth in 0..15 {
        let solver = MinimaxSolver::<MancalaGameState>::new(MancalaGameState::default());
        let now = SystemTime::now();
        let value = solver.solve(depth);
        let time = now.elapsed().unwrap();
        info!(
            "Value: {}, Depth: {}, Time: {}ms",
            value,
            depth,
            time.as_millis()
        );
    }
}
