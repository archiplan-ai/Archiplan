//! The textual interface as a REPL: statements apply one at a time.
//!
//! ```sh
//! cargo run --example repl
//! ```

use std::io::{self, BufRead, Write};

use modeling_lang::Session;

fn main() {
    let mut session = Session::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        let scope = session.scope_path();
        if scope.is_empty() {
            print!("> ");
        } else {
            print!("{scope}> ");
        }
        stdout.flush().expect("flush stdout");
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {e}");
                break;
            }
        }
        for r in session.execute_interactive(&line) {
            match r.result {
                Ok(outcome) => println!("{outcome}"),
                Err(e) => println!("{e}"),
            }
        }
    }
}
