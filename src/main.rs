
mod mpv;

use std::{io};

use mpv::process::ExistingProcess;

use std::thread::sleep;
use std::time::Duration;

use crate::mpv::process::MpvProcess;

#[tokio::main]
async fn main() -> Result<(), io::Error> {

    println!("Creating client");
    let client = ExistingProcess::new("/tmp/server");

    println!("Connecting to socket");
    let connection = client.create_connection().await;
    println!("Connected");

    loop {
        sleep(Duration::from_secs(1));
        println!(".");
    }
}
