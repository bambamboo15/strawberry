use watermelon::prelude::*;

pub enum LogScoreValue {
    Centipawns(i16),
    Mate(i16),
}

pub enum LogScoreBound {
    Exact,
    Lower,
    Upper,
}

/// Determines the kind of message the engine sends during a search.
#[non_exhaustive]
pub enum Log {
    DepthIteration {
        depth: u8,
        nodes: usize,
        nodes_per_second: usize,
        best_move: Move,
        score_value: LogScoreValue,
        score_bound: LogScoreBound,
        principal_variation: Vec<Move>,
    },
}

/// Trait for multithreaded logging of informational messages during a search.
pub trait Logger: Send + Sync {
    fn log(&self, game: &Game, log: Log);
}
