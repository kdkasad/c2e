use std::{
    io::{stdin, IsTerminal},
    process::ExitCode,
};

use c_explainer::{explainer::explain_declaration, parser::parser};
use chumsky::Parser;
use rustyline::{error::ReadlineError, Config, DefaultEditor};

fn main() -> ExitCode {
    let rl_config = Config::builder().auto_add_history(true).build();
    let mut rl = DefaultEditor::with_config(rl_config).unwrap();
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                match parser().parse(&line).into_result() {
                    Ok(decl) => {
                        let explanation = explain_declaration(&decl);
                        println!("{explanation}");
                    }
                    Err(errs) => {
                        eprintln!("Error(s) parsing declaration:");
                        for err in errs {
                            eprintln!("{err}");
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if stdin().is_terminal() {
                    println!("Interrupted; exiting...");
                }
                return ExitCode::SUCCESS;
            }
            Err(ReadlineError::Eof) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error reading line: {err}");
                return ExitCode::FAILURE;
            }
        };
    }
}
