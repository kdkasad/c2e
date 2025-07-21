use std::{process::Command, time::Duration};

use rexpect::session::{PtySession, spawn_command};

fn spawn() -> PtySession {
    let path = env!("CARGO_BIN_EXE_c-explainer-cli");
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
    c.exp_string("Error(s) parsing declaration:\r\nfound '=' expected '[', '(', or end of input")
        .unwrap();
    c.exp_string("> ").unwrap();
    kill(c);
}
