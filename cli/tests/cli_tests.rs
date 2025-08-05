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
    io::Write,
    process::{Command, Stdio},
    time::Duration,
};

use rexpect::{
    session::{Options, PtySession},
    spawn_with_options,
};

use pretty_assertions::assert_eq;

fn spawn(color: bool) -> PtySession {
    let path = env!("CARGO_BIN_EXE_c2e");
    let mut cmd = Command::new(path);
    if color {
        cmd.env_clear().env("TERM", "xterm-256color");
    }
    spawn_with_options(
        cmd,
        Options {
            timeout_ms: Some(Duration::from_secs(10).as_millis() as u64),
            strip_ansi_escape_codes: !color,
        },
    )
    .unwrap()
}

fn kill(mut c: PtySession) {
    c.send_control('d').unwrap();
    c.exp_eof().unwrap();
}

#[test]
fn test_send_eof() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("int").unwrap();
    c.exp_string("an int").unwrap();
    c.exp_string("> ").unwrap();
    c.send_control('d').unwrap(); // Send EOF
    c.exp_eof().unwrap();
}

#[test]
fn test_interrupt() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("int").unwrap();
    c.exp_string("an int").unwrap();
    c.exp_string("> ").unwrap();
    c.send_control('c').unwrap(); // Send EOF
    c.exp_string("Interrupted; exiting...").unwrap();
    c.exp_eof().unwrap();
}

#[test]
fn test_send_empty_line() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("").unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}

#[test]
fn test_parse_error() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("int x = 5;").unwrap();
    c.exp_string("Error(s) parsing declaration:\r\n").unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}

#[test]
fn test_read_error() {
    let mut c = Command::new(env!("CARGO_BIN_EXE_c2e"))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.as_mut().unwrap().write_all(&[200, 200]).unwrap();
    let output = c.wait_with_output().unwrap();
    let out_str = str::from_utf8(&output.stderr).unwrap();
    println!("\"{out_str}\"");
    assert!(out_str.contains("Error reading line: stream did not contain valid UTF-8\n"));
}

#[test]
fn test_print_license() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("@license").unwrap();
    let output = c.exp_string("> ").unwrap();
    kill(c);
    assert!(output.contains("GNU General Public License"));
    assert!(output.contains(env!("CARGO_PKG_REPOSITORY")));
}

#[test]
fn test_interactive_license_header() {
    let mut c = spawn(false);
    let header = c.exp_string("> ").unwrap();
    kill(c);
    assert!(header.contains("This program comes with ABSOLUTELY NO WARRANTY."));
}

#[test]
fn test_non_interactive_no_license() {
    let mut c = Command::new(env!("CARGO_BIN_EXE_c2e"))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.as_mut().unwrap().write_all(b"int foo\n").unwrap();
    let output = c.wait_with_output().unwrap();
    let out_str = str::from_utf8(&output.stdout).unwrap();
    assert_eq!(out_str, "an int named foo\n", "wrong output on stdout");
    assert!(output.stderr.is_empty(), "expected stderr to be empty");
}

#[test]
fn test_multiple_declarations() {
    let mut c = spawn(false);
    c.exp_string("> ").unwrap();
    c.send_line("int x; float y;").unwrap();
    c.exp_string("an int named x;\r\na float named y;").unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}

#[test]
fn test_colors() {
    let mut c = spawn(true);
    c.exp_string("> ").unwrap();
    c.send_line("const struct foo *func(int[10]);").unwrap();
    c.exp_string("a ").unwrap();
    c.exp_string("\x1b[32m").unwrap();
    c.exp_string("function").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" named ").unwrap();
    c.exp_string("\x1b[31m").unwrap();
    c.exp_string("func").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" that takes (").unwrap();
    c.exp_string("an ").unwrap();
    c.exp_string("\x1b[32m").unwrap();
    c.exp_string("array").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" of").unwrap();
    c.exp_string("\x1b[34m").unwrap();
    c.exp_string("10").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" ").unwrap();
    c.exp_string("\x1b[33m").unwrap();
    c.exp_string("int").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string("s) and returns a ").unwrap();
    c.exp_string("\x1b[32m").unwrap();
    c.exp_string("pointer").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" to a ").unwrap();
    c.exp_string("\x1b[36m").unwrap();
    c.exp_string("const").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string(" ").unwrap();
    c.exp_string("\x1b[35m").unwrap();
    c.exp_string("struct foo").unwrap();
    c.exp_string("\x1b[0m").unwrap();
    c.exp_string("\r\n").unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}

#[test]
fn test_error_color() {
    let mut c = spawn(true);
    c.exp_string("> ").unwrap();
    c.send_line("int x = 5;").unwrap();
    c.exp_string("\x1b[31m").unwrap(); // Error color
    c.exp_string("Error(s) parsing declaration:\r\n").unwrap();
    c.exp_string("\r\n").unwrap();
    c.exp_string("\x1b[0m").unwrap(); // Reset color
    c.exp_string("> ").unwrap();
    kill(c);
}
