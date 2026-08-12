use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};
use strawberry::{
    logger::{Log, LogScoreBound, LogScoreValue, Logger},
    structs::{SearchCache, SearchParams, SearchTiming},
};
use watermelon::prelude::*;

struct StrawberryLogger;
impl Logger for StrawberryLogger {
    #[inline(never)]
    fn log(&self, game: &Game, log: Log) {
        match log {
            Log::DepthIteration {
                depth,
                nodes,
                nodes_per_second,
                best_move,
                score_value,
                score_bound,
                principal_variation,
            } => {
                let currmove = game.move_to_intent(best_move).unwrap();
                println!(
                    "info depth {depth} score{}{} nodes {nodes} nps {nodes_per_second} currmove {currmove} pv {}",
                    match score_bound {
                        LogScoreBound::Exact => "",
                        LogScoreBound::Lower => " lowerbound",
                        LogScoreBound::Upper => " upperbound",
                    },
                    match score_value {
                        LogScoreValue::Centipawns(value) => format!(" cp {value}"),
                        LogScoreValue::Mate(value) => format!(" mate {value}"),
                    },
                    {
                        let mut board = Game::from_branch(game);
                        let mut str = String::new();
                        for mv in principal_variation {
                            str.push_str(&board.move_to_intent(mv).unwrap().to_string());
                            str.push(' ');
                            board.try_play(mv);
                        }
                        str
                    }
                );
            }
            _ => {}
        }
    }
}

pub struct StrawberryGame {
    game: Game,
    cache: SearchCache,
}

/// Resposible for handling searches from the Strawberry engine.
pub struct Strawberry {
    game: Option<StrawberryGame>,
    handle: Option<JoinHandle<StrawberryGame>>,
    stop: Arc<AtomicBool>,
}

impl Strawberry {
    pub fn new() -> Self {
        Strawberry {
            game: Some(StrawberryGame {
                game: Game::start_position(),
                cache: SearchCache::new(512),
            }),
            handle: None,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_search(&mut self, timing: SearchTiming) {
        self.stop_search();
        if let Some(mut game) = self.game.take() {
            let stop = self.stop.clone();
            self.handle = Some(std::thread::spawn(move || {
                let params = SearchParams {
                    timing,
                    game: game.game,
                    stop,
                    logger: &StrawberryLogger,
                };

                let result = strawberry::search(params, &mut game.cache);
                let intent = result.game.move_to_intent(result.best_move).unwrap();
                println!("bestmove {}", intent);

                StrawberryGame {
                    game: result.game,
                    cache: game.cache,
                }
            }));
        }
    }

    pub fn stop_search(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Relaxed);
            self.game = Some(handle.join().unwrap());
            self.stop.store(false, Ordering::Relaxed);
        }
    }

    pub fn game_mut(&mut self) -> Option<&mut Game> {
        self.stop_search();
        match self.game.as_mut() {
            Some(game) => Some(&mut game.game),
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.stop_search();
        if let Some(game) = self.game.as_mut().take() {
            game.game = Game::start_position();
            game.cache.clear();
        }
    }
}
