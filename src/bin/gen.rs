use vdf::{VDFParams, VDF, WesolowskiVDFParams};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: gen \"<message>\" <T>");
        eprintln!("Example: gen \"Behold\" 1000000");
        std::process::exit(1);
    }
    let message = &args[1];
    let t: u64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("T must be a positive integer");
        std::process::exit(1);
    });

    let vdf = WesolowskiVDFParams(512).new();
    eprintln!("Generating VDF proof for \"{}\" with T={}...", message, t);
    eprintln!("(this is sequential and cannot be parallelised)");

    let proof = vdf
        .solve(message.as_bytes(), t)
        .unwrap_or_else(|e| {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        });

    println!("{}", hex::encode(&proof));
    eprintln!("Done. Copy the hex line above and submit it at the VDP board.");
}
