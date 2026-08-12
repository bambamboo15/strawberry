//! I've been inspired by a bunch of chess engines!
//!
/// [https://github.com/AndyGrant/Ethereal]
/// [https://github.com/SebLague/Chess-Coding-Adventure]
use crate::{logger::*, structs::*, table::*};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use watermelon::prelude::*;

pub fn search(mut params: SearchParams<'_>, cache: &mut SearchCache) -> SearchResults {
    // Determine the maximum amount of time we can spend based on the timing parameters.
    let timing = &params.timing;
    let time_limit = if let Some(movetime) = timing.movetime {
        let secs = movetime.as_secs_f32().min(1.0);
        Duration::from_secs_f32(secs)
    } else {
        let duration = match params.game.color() {
            Color::White => timing.wtime,
            Color::Black => timing.btime,
        };

        if let Some(duration) = duration {
            let duration = duration.as_secs_f32();
            let increment = match params.game.color() {
                Color::White => timing.winc,
                Color::Black => timing.binc,
            }
            .unwrap_or(Duration::ZERO)
            .as_secs_f32();

            // The engine uses a tiny slice of the duration, then adds it to the increment. However,
            // for extremely short time controls, this can lead to the engine happily exceeding the
            // duration because of increment. Thus we make sure this never takes more than 80% of
            // the total duration. Additionally, due to GUI lag and how often we check for time limit
            // during search, we subtract around 30ms from the allocated engine duration.
            let secs = ((duration * 0.04 + increment).min(duration * 0.8) - 0.03).clamp(0.0, 0.75);
            Duration::from_secs_f32(secs)
        } else {
            Duration::MAX /* no time specified, but also no infinite, so go on until depth limit */
        }
    };

    // This is a different search, so the entries that we will put into the transposition table
    // will be newer, and we want to replace older entries with them more often.
    cache.table.advance_generation();

    // Implement iterative deepening, where we perform searches that lower a depth each time.
    let mut context = SearchContext {
        stop: &*params.stop,
        logger: params.logger,
        cache,
        time_start: Instant::now(),
        time_limit,
        best_move: Move::NULL,
        nodes: 0,
        initial_ply: params.game.ply_count(),
    };

    for depth in 1..100 {
        let search = params.game.search_mut();
        let score = negamax(&mut context, search, depth, MIN, MAX);
        if score.abs() == STOP {
            break;
        }

        // Log various statistics about the best move found for this iteration.
        context.logger.log(
            &params.game,
            Log::DepthIteration {
                depth,
                nodes: context.nodes,
                nodes_per_second: {
                    let seconds = context.time_start.elapsed().as_secs_f32();
                    (context.nodes as f32 / seconds) as usize
                },
                best_move: context.best_move,
                score_value: if score.abs() >= MINMATE {
                    let plies = MATE - score.abs();
                    let moves = (plies + 1) / 2;
                    LogScoreValue::Mate(score.signum() * moves)
                } else {
                    LogScoreValue::Centipawns(score)
                },
                // Since we cannot recover the original information, we just keep it `Exact`
                // even though that is not accurate.
                score_bound: LogScoreBound::Exact,
                // Recover the principal variation (PV) information using the TT. This is not the
                // best method but is still very good.
                principal_variation: {
                    let mut pv = Vec::new();
                    let mut board = Game::from_branch(&params.game);
                    for _ in 0..depth {
                        if let Some(entry) = context.cache.table.probe(board.zobrist_hash()) {
                            if board.try_play(entry.best_move) {
                                pv.push(entry.best_move);
                                continue;
                            }
                        }
                        break;
                    }
                    pv
                },
            },
        );
    }

    // When the search is over, pick a random move if there was none, and output the results.
    SearchResults {
        best_move: match context.best_move {
            Move::NULL => {
                use rand::seq::IndexedRandom;
                *(params.game)
                    .legal_moves()
                    .moves()
                    .choose(&mut rand::rng())
                    .unwrap()
            }
            x => x,
        },
        game: params.game,
    }
}

struct SearchContext<'a> {
    stop: &'a AtomicBool,
    logger: &'a dyn Logger,
    cache: &'a mut SearchCache,
    time_start: Instant,
    time_limit: Duration,
    best_move: Move,
    nodes: usize,
    initial_ply: usize,
}

const STOP: i16 = 32001;
const MIN: i16 = -32000;
const MAX: i16 = 32000;
const MATE: i16 = 31000;
const MINMATE: i16 = 30000;

fn negamax(
    context: &mut SearchContext,
    mut search: SearchZero<'_, Exclusive>,
    depth: u8,
    mut alpha: i16,
    mut beta: i16,
) -> i16 {
    // Periodically check for stopping conditions.
    // This must be done every 2-4ms so that we do not lose on time in extremely fast time controls!
    context.nodes += 1;
    if context.nodes & 1023 == 0 {
        std::hint::cold_path();
        if context.stop.load(Ordering::Relaxed) || context.time_start.elapsed() > context.time_limit
        {
            return STOP;
        }
    }

    // Check for draws before doing anything.
    let game = search.game();
    let ply_in_search = game.ply_count() - context.initial_ply;
    if game.is_fifty_move_draw() || drawn_by_repetition(game, ply_in_search) {
        std::hint::cold_path();
        return 0;
    }

    // Mate distance pruning. Chess engines really feel like building from a set of predefined
    // heuristics these days, but from a young age I've always wanted to build one that at
    // least plays a little well! I think this project does just that.
    alpha = std::cmp::max(alpha, -MATE + ply_in_search as i16);
    beta = std::cmp::min(beta, MATE - (ply_in_search as i16 + 1));
    if alpha >= beta {
        return alpha;
    }

    // Perform a transposition table lookup.
    let old_alpha = alpha;
    let zobrist_hash = game.zobrist_hash();
    let tt_entry = context.cache.table.probe(zobrist_hash);
    if let Some(entry) = tt_entry
        && entry.depth >= depth
    {
        // When the stored score is a checkmate (example: -M3 or M4), the score will be in
        // the form of how many plies until a checkmate happens. However, our search deals
        // with how many plies happen from the search root until the checkmate.
        //    (TT: Checkmated in 3 plies) -MATE + 3 -> -MATE + (ply_in_search + 3)
        //    (TT: Checkmating in 3 plies) MATE - 3 -> MATE - (ply_in_search + 3)
        let score = entry.score
            + match entry.score {
                v if v < -MINMATE => ply_in_search as i16,
                v if v > MINMATE => -(ply_in_search as i16),
                _ => 0,
            };

        if match entry.bound() {
            TranspositionBound::None => false,
            TranspositionBound::Exact => true,
            TranspositionBound::Lower => score >= beta,
            TranspositionBound::Upper => score <= alpha,
        } {
            // If a root node hits the transposition table, we must adjust the best move.
            if ply_in_search == 0 {
                std::hint::cold_path();
                context.best_move = entry.best_move;
            }

            return score;
        }
    }

    // Check for any game-ending results.
    let mut legal_moves = search.legal_moves();
    if legal_moves.moves().is_empty() {
        std::hint::cold_path();
        return if legal_moves.game().is_in_check() {
            // This is the score for **our side** being checkmated. The farther the checkmate
            // from the search root, the better the score is. When this bubbles up to the top,
            // we can also take note of how many plies the checkmate occurs in.
            -MATE + ply_in_search as i16
        } else {
            0
        };
    }

    // If a terminal node, perform static evaluation.
    if depth == 0 {
        return eval(search.game());
    }

    // Perform a greatest-to-least sort of the legal moves using some heuristics.
    // We always want the TT move to be in the front as it is usually the best move, so we do
    // this early and swap the rest to optimize sorting a little.
    let tt_move_hit = if let Some(entry) = tt_entry
        && let Some(index) = legal_moves
            .moves()
            .iter()
            .position(|&x| x == entry.best_move)
    {
        legal_moves.swap(0, index);
        true
    } else {
        false
    };

    legal_moves
        .as_mut_sliced((tt_move_hit as usize)..)
        .sort_unstable_by_key(|search| {
            if let Some(captured_piece) = search.captured_piece() {
                let moved_piece = search.moved_piece();
                (
                    std::cmp::Reverse(100000 + captured_piece as usize),
                    moved_piece as usize,
                )
            } else {
                (std::cmp::Reverse(0), 0)
            }
        });

    // Perform negamax search on all moves.
    let mut best_move = Move::NULL;
    let mut best_score = MIN;
    for index in 0..legal_moves.moves().len() {
        // For some reason (probably aliasing) LLVM cannot prove the length is invariant across
        // loop iterations, so a bounds check is here. That's fine.
        let mut one = legal_moves.as_one(index);
        let score = one.test(|zero| -negamax(context, zero, depth - 1, -beta, -alpha));

        if score == -STOP {
            std::hint::cold_path();
            return STOP;
        }

        if score > best_score {
            best_score = score;
            best_move = one.mv();

            if score >= beta {
                break;
            }
            alpha = std::cmp::max(alpha, score);
        }
    }

    // Transposition table store. See the comment in the transposition table lookup for why the mate
    // scores have to be adjusted.
    let score = best_score
        + if best_score.abs() > MINMATE {
            best_score.signum() * ply_in_search as i16
        } else {
            0
        };
    let bound = match best_score {
        v if v >= beta => TranspositionBound::Lower,
        v if v <= old_alpha => TranspositionBound::Upper,
        _ => TranspositionBound::Exact,
    };
    (context.cache.table).store(zobrist_hash, depth, score, bound, best_move);

    // Overwrite the best move if everything is okay with the search.
    if ply_in_search == 0 {
        std::hint::cold_path();
        context.best_move = best_move;
    }

    best_score
}

fn eval(game: &Game) -> i16 {
    let position = game.position();

    let mut score = 0;
    for (piece, value) in [
        (Piece::Pawn, 100),
        (Piece::Knight, 300),
        (Piece::Bishop, 300),
        (Piece::Rook, 500),
        (Piece::Queen, 900),
    ] {
        score += value
            * (position.bitboard(piece, Color::White).count_ones() as i16
                - position.bitboard(piece, Color::Black).count_ones() as i16);
    }

    std::hint::select_unpredictable(game.color() == Color::White, score, -score)
}

/// Checks if the position is drawn by repetition from the engine's standpoint.
fn drawn_by_repetition(game: &Game, height: usize) -> bool {
    let mut reps = 0;

    // Look through hash histories, from the current move down to the last move that
    // reset the half-move counter, as no repetitions could have occured before then.
    let mut index = 2;
    while let Some(hash) = game
        .zobrist_hash_history()
        .get_historical_after_reset(index)
    {
        if hash == game.zobrist_hash() {
            reps += 1;
            if index < height || reps == 2 {
                return true;
            }
        }
        index += 2;
    }

    false
}
