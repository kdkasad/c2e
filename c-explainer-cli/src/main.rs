/*
 * This program is free software: you can redistribute it and/or modify it under the terms of
 * the GNU General Public License as published by the Free Software Foundation, either version
 * 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
 * without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program. If
 * not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    io::{IsTerminal, stdin},
    process::ExitCode,
};

use c_explainer::{explainer::explain_declaration, parser::parser};
use chumsky::Parser;
use rustyline::{Config, DefaultEditor, error::ReadlineError};

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
        }
    }
}
