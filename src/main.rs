use std::env;
use std::process;

use martian_chess::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    
    if let Err(e) = martian_chess::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

