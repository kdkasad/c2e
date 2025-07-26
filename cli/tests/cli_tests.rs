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

use rexpect::session::{PtySession, spawn_command};

use pretty_assertions::assert_eq;

fn spawn() -> PtySession {
    let path = env!("CARGO_BIN_EXE_c2e");
    spawn_command(
        Command::new(path),
        Some(Duration::from_secs(10).as_millis() as u64),
    )
    .unwrap()
}

fn kill(mut c: PtySession) {
    c.send_control('d').unwrap();
    c.exp_eof().unwrap();
}

#[test]
fn test_send_eof() {
    let mut c = spawn();
    c.exp_string("> ").unwrap();
    c.send_line("int").unwrap();
    c.exp_string("an int").unwrap();
    c.exp_string("> ").unwrap();
    c.send_control('d').unwrap(); // Send EOF
    c.exp_eof().unwrap();
}

#[test]
fn test_interrupt() {
    let mut c = spawn();
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
    let mut c = spawn();
    c.exp_string("> ").unwrap();
    c.send_line("").unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}

#[test]
fn test_parse_error() {
    let mut c = spawn();
    c.exp_string("> ").unwrap();
    c.send_line("int x = 5;").unwrap();
    c.exp_string("Error(s) parsing declaration:\r\nat 6..7: expected '[', '(', or end of input, but found '='")
        .unwrap();
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
    let mut c = spawn();
    c.exp_string("> ").unwrap();
    c.send_line("@license").unwrap();
    let output = c.exp_string("> ").unwrap();
    kill(c);
    assert!(output.contains("GNU General Public License"));
    assert!(output.contains(env!("CARGO_PKG_REPOSITORY")));
}

#[test]
fn test_interactive_license_header() {
    let mut c = spawn();
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
