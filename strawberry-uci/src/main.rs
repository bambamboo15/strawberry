use watermelon::prelude::*;

fn perft(mut search: SearchZero<'_, Exclusive>, depth: usize) -> usize {
    match depth {
        0 => 1,
        1 => search.legal_moves().moves().len(),
        _ => {
            let mut count = 0;
            let mut search = search.legal_moves();
            for index in 0..search.moves().len() {
                search
                    .as_one(index)
                    .test(|game| count += perft(game, depth - 1));
            }
            count
        }
    }
}

fn main() {
    for (name, fen) in [
        //("start", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
        //("tricky", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        //("complex", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        //("buggy", "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        //("position-6", "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w -"),
    ] {
        let mut game = Game::from_fen(fen).unwrap();

        let start = std::time::Instant::now();
        let depth = 5;
        let iterations = 1;
        let mut count = 0;
        for _ in 0..iterations {
            count += perft(game.search_mut(), depth);
        }
        let elapsed = start.elapsed();

        println!("\x1b[38;2;255;255;255m{} | {}\x1b[0m", name, fen);
        println!("depth: {} -> nodes: {}", depth, count / iterations);
        println!(
            "{:.2} us | {:.2} Mnps",
            elapsed.as_micros() as f64 / iterations as f64,
            count as f64 / (1000000.0 * elapsed.as_secs_f64())
        );
    }
}
