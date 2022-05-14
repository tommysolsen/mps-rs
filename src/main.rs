use std::io;

mod shell;
mod mpv;


use std::process::exit;

use crate::mpv::MpvResponse::UnknownResponse;
use crate::shell::InitializerSettings;

fn main() -> Result<(), io::Error> {
    println!("Hello, world!");

    let settings = InitializerSettings::new(); //.set_existing_socket("/tmp/server");

    let mut mpv = shell::initialize(settings)?;
    let _ = mpv.load_file("https://www.youtube.com/watch?v=lhmKby7c-Jg");

    // Handles CTRL+C, kills the mpv instance if it was spawned by the initiate method, leaves it
    // if it was spawned manually
    let killer = mpv.make_killer();
    ctrlc::set_handler(move || {
        match &killer {
            Some(killer) => killer.send(()).unwrap(),
            _ => {},
        }
        exit(0);
    }).expect("Error setting Ctrl-C handler");

    let mut stop = false;
    loop {
        match mpv.events().recv().unwrap_or(UnknownResponse("UNPARSEABLE EVENT".to_string())) {
            UnknownResponse(x) if x == "" => stop = true,
            cmd => println!("{:?}", cmd),
        }

    }
}
