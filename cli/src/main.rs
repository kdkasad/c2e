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
    io::{IsTerminal, Write, stderr, stdin, stdout},
    process::ExitCode,
};

use c2e::{
    explainer::explain_declaration,
    parser::{State, parser},
};
use chumsky::Parser;
use fmt::{CliFormatter, ColorMap};
use rustyline::{Config, DefaultEditor, error::ReadlineError};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

mod fmt;

// Must be a macro so it expands to a string literal
macro_rules! copyright_header {
    () => {
        concat!(
            env!("CARGO_BIN_NAME"),
            " ",
            env!("CARGO_PKG_VERSION"),
            "\n",
            "Copyright (C) 2025  ",
            env!("CARGO_PKG_AUTHORS"),
            "\n",
        )
    };
}

const COLOR_MAP: ColorMap = ColorMap {
    qualifier: Color::Cyan,
    primitive_type: Color::Yellow,
    user_defined_type: Color::Magenta,
    identifier: Color::Red,
    number: Color::Blue,
    quasi_keyword: Color::Green,
};

fn main() -> ExitCode {
    let rl_config = Config::builder().auto_add_history(true).build();
    let mut rl = DefaultEditor::with_config(rl_config).unwrap();

    // Print license information if interactive
    if stdin().is_terminal() {
        eprintln!(indoc::concatdoc! {
            copyright_header!(), r"
            This program comes with ABSOLUTELY NO WARRANTY.
            This is free software, and you are welcome to redistribute it
            under certain conditions; type `@license' for details.
            "
        });
    }

    // Use color if the output is a terminal, otherwise disable it
    let formatter = CliFormatter::new(COLOR_MAP);
    let mut stdout = StandardStream::stdout(if stdout().is_terminal() {
        termcolor::ColorChoice::Auto
    } else {
        termcolor::ColorChoice::Never
    });
    let mut stderr = StandardStream::stderr(if stderr().is_terminal() {
        termcolor::ColorChoice::Auto
    } else {
        termcolor::ColorChoice::Never
    });

    // Persist state input lines
    let mut parser_state = State::default();

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                if line == "@license" {
                    eprintln!(indoc::concatdoc! {
                        copyright_header!(), "
                        This program is free software: you can redistribute it and/or modify
                        it under the terms of the GNU General Public License as published by
                        the Free Software Foundation, either version 3 of the License, or
                        (at your option) any later version.

                        This program is distributed in the hope that it will be useful,
                        but WITHOUT ANY WARRANTY; without even the implied warranty of
                        MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
                        GNU General Public License for more details.

                        You should have received a copy of the GNU General Public License
                        along with this program.  If not, see <https://www.gnu.org/licenses/>.

                        ---

                        Source code is available at ", env!("CARGO_PKG_REPOSITORY")
                    });
                    continue;
                }

                match parser()
                    .parse_with_state(&line, &mut parser_state)
                    .into_result()
                {
                    Ok(decls) => match &decls[..] {
                        [decl] => {
                            let explanation = explain_declaration(decl);
                            formatter.format(&mut stdout, explanation).unwrap();
                            writeln!(&mut stdout).unwrap();
                        }
                        decls => {
                            for decl in decls {
                                let explanation = explain_declaration(decl);
                                formatter.format(&mut stdout, explanation).unwrap();
                                writeln!(&mut stdout, ";").unwrap();
                            }
                        }
                    },
                    Err(errs) => {
                        stderr
                            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                            .unwrap();
                        eprintln!("Error(s) parsing declaration:");
                        for err in errs {
                            eprintln!("{err}");
                        }
                        stderr.reset().unwrap();
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
