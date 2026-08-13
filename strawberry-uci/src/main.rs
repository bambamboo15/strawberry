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
    let mut game =
        Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq").unwrap();

    let start = std::time::Instant::now();
    let iterations = 1;
    let depth = 5;
    let mut count = 0;
    for _ in 0..iterations {
        count += perft(game.search_mut(), depth);
    }
    let elapsed = start.elapsed();

    println!(
        "number of legal moves from starting position in {} plies: {}",
        depth,
        count / iterations
    );
    println!(
        "time taken: {:.2}us",
        elapsed.as_micros() as f64 / iterations as f64
    );
    println!(
        "{} Mnps",
        count as f64 / (1000000.0 * elapsed.as_secs_f64())
    );
}
