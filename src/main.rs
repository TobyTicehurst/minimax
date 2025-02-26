#![allow(dead_code)]

mod mancala;
mod minimax;

use mancala::{interactive, interactive::SolverAlgorithm, MancalaEndgamesTable, MancalaGameState};
use minimax::EndgamesTable;
use std::rc::Rc;
use tracing::level_filters::LevelFilter;

enum Operation {
    PlayerVsBot,
    Analyse,
}

fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_ansi(false)
        .without_time()
        .init();

    let operation = Operation::Analyse;
    match operation {
        Operation::PlayerVsBot => interactive::player_vs_mancala_bot(),
        Operation::Analyse => {
            let depth = 26;
            //let endgames_table = MancalaEndgamesTable::read_from_file("endgames.bin");
            let mut endgames_table = MancalaEndgamesTable::new(0);
            endgames_table.calculate_endgames();
            interactive::analyse_position(
                MancalaGameState::default(),
                SolverAlgorithm::MtdfMemory,
                depth,
                &Rc::new(endgames_table),
            )
        }
    }
}
