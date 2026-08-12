use crate::{logger::Logger, table::TranspositionTable};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use watermelon::prelude::*;

/// Caches useful information across moves of a game.
pub struct SearchCache {
    pub(crate) table: TranspositionTable,
}

impl SearchCache {
    pub fn new(megabytes: usize) -> Self {
        SearchCache {
            table: TranspositionTable::with_megabytes(megabytes),
        }
    }

    pub fn clear(&mut self) {
        self.table.reset();
    }
}

/// Timing parameters.
pub struct SearchTiming {
    /// The search lasts approximately this long.
    pub movetime: Option<Duration>,
    /// The amount of time white has on the clock.
    pub wtime: Option<Duration>,
    /// The amount of time black has on the clock.
    pub btime: Option<Duration>,
    /// Increment white gets per move.
    pub winc: Option<Duration>,
    /// Increment black gets per move.
    pub binc: Option<Duration>,
}

impl SearchTiming {
    pub fn new() -> Self {
        SearchTiming {
            movetime: None,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
        }
    }
}

/// Parameters that control how the relevant search is done.
pub struct SearchParams<'a> {
    pub timing: SearchTiming,
    pub game: Game,
    pub stop: Arc<AtomicBool>,
    pub logger: &'a dyn Logger,
}

pub struct SearchResults {
    pub game: Game,
    pub best_move: Move,
}
