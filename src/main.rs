use std::io;

mod shell;
mod mpv;

use std::{thread, time::Duration};
use std::process::exit;
use std::sync::mpsc;
use crate::mpv::ReadCommandErrors;

fn main() -> Result<(), io::Error> {
    println!("Hello, world!");

    let (mut child, mut mpv) = shell::initialize()?;
    mpv.loadFile("https://www.youtube.com/watch?v=lhmKby7c-Jg")?;

    let (tx, rx) = mpsc::channel::<bool>();

    thread::spawn(move || {
        let x = rx.recv();
        if x.is_ok() {
            child.kill().expect("Expected to be able to kill mpv");
            child.wait().expect("Expected to be able to wait for it to die");
            exit(0);
        }
    });

    let tx_ctrlc = tx.clone();
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        tx_ctrlc.send(true).expect("Expected to be able to send kill signal");
    }).expect("Error setting Ctrl-C handler");

    println!("Reading commands");
    loop {
        thread::sleep(Duration::from_secs(1));
        let cmd = mpv.read_next_command().unwrap();

    }
    tx.send(true).expect("Expected to be able to send kill signal");
    return Ok(());
}
