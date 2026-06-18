use vdf::{VDFParams, VDF, WesolowskiVDFParams};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: verify \"<message>\" <T> <proof_hex>");
        std::process::exit(1);
    }
    let message = &args[1];
    let t: u64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("T must be a positive integer");
        std::process::exit(1);
    });
    let proof = hex::decode(&args[3]).unwrap_or_else(|_| {
        eprintln!("Invalid hex in proof");
        std::process::exit(1);
    });

    let vdf = WesolowskiVDFParams(512).new();
    let valid = vdf.verify(message.as_bytes(), t, &proof).is_ok();

    if valid {
        println!("VALID — proof is authentic for \"{}\" at T={}", message, t);
    } else {
        println!("INVALID — proof does not verify for \"{}\" at T={}", message, t);
        std::process::exit(1);
    }
}
