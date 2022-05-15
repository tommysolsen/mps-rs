
mod mpv;

use std::{io, thread};

use mpv::process::ExistingProcess;

use std::thread::sleep;
use std::time::Duration;
use crate::mpv::client::Client;

use crate::mpv::process::MpvProcess;

#[tokio::main]
async fn main() -> Result<(), io::Error> {

    println!("Creating client");
    let process = ExistingProcess::new("/tmp/server");

    let mut client = Client::new(process).await?;

    client.observe("volume", 1).await?;
    client.observe("time-pos", 2).await?;
    client.observe("duration", 3).await?;
    client.observe("media-title", 4).await?;
    thread::sleep(Duration::from_secs(1));
    client.load_file("https://www.youtube.com/watch?v=Mxa4EhLWUL4").await?;
    client.get_property("playlist").await?;

    loop {
        sleep(Duration::from_secs(1));
        println!(".");
    }
}
