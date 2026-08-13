mod engine;
use crate::engine::Strawberry;
use std::{str::FromStr, time::Duration};
use strawberry::structs::SearchTiming;
use watermelon::prelude::*;

fn main() {
    println!("\x1b[38;2;255;100;100mstrawberry - by bambamboo15 on github ^w^\x1b[0m");

    let mut strawberry = Strawberry::new();

    'outer: loop {
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);

        let mut commands = line.split_whitespace();
        let Some(command) = commands.next() else {
            continue;
        };

        match command {
            "uci" => {
                println!("id name strawberry 0.1");
                println!("id author bambamboo15");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                strawberry.clear();
            }
            "setoption" => { /* ignore all setoption commands for now */ }
            "stop" => {
                strawberry.stop_search();
            }
            "quit" => {
                break 'outer;
            }
            "position" => {
                let mut game = match commands.next() {
                    Some("startpos") => Game::start_position(),
                    Some("fen") => {
                        // Collect up to 6 fields, leaving 'moves' completely untouched.
                        let mut fen_fields = Vec::new();
                        for _ in 0..6 {
                            if commands.clone().next() != Some("moves")
                                && let Some(field) = commands.next()
                            {
                                fen_fields.push(field);
                            } else {
                                break;
                            }
                        }

                        let fen_string = fen_fields.join(" ");
                        Game::from_fen(&fen_string).expect("invalid FEN provided")
                    }
                    _ => continue 'outer,
                };

                if commands.next() == Some("moves") {
                    while let Some(str) = commands.next()
                        && let Ok(intent) = MoveIntent::from_str(str)
                        && let Some(mv) = game.intent_to_move(intent)
                        && game.try_play(mv)
                    {}
                }

                if let Some(global_game) = strawberry.game_mut() {
                    *global_game = game;
                }
            }
            "go" => {
                let mut timing = SearchTiming::new();
                while let Some(command) = commands.next() {
                    match command {
                        "movetime" | "wtime" | "btime" | "winc" | "binc" => {
                            let millis = commands.next().unwrap().parse::<u64>().unwrap();
                            let duration = Duration::from_millis(millis);
                            match command {
                                "movetime" => timing.movetime = Some(duration),
                                "wtime" => timing.wtime = Some(duration),
                                "btime" => timing.btime = Some(duration),
                                "winc" => timing.winc = Some(duration),
                                "binc" => timing.binc = Some(duration),
                                _ => unreachable!(),
                            }
                        }
                        // Non-standard command just to test the Lasker position.
                        "lasker" => {
                            strawberry.clear();
                            if let Some(global_game) = strawberry.game_mut() {
                                *global_game =
                                    Game::from_fen("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - -").unwrap();
                            }
                        }
                        _ => unimplemented!(),
                    }
                }

                strawberry.start_search(timing);
            }
            _ => println!("unknown command '{}'", command),
        }
    }
}
