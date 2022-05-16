
mod mpv;

use std::{io};
use std::thread::sleep;
use std::time::Duration;
use crate::mpv::client::Client;
use crate::mpv::process::{ManagedProcess, MpvInitializer, MpvProcess};

#[tokio::main]
async fn main() -> Result<(), io::Error> {

    println!("Creating client");
    let process = ManagedProcess::new(
        MpvInitializer::new().with_args(vec!["--no-video".to_string()])
    ).unwrap();
    sleep(Duration::from_secs(1));
    let mut client = Client::new(process).await?;

    client.observe("volume", 1).await?;
    client.observe("time-pos", 2).await?;
    client.observe("duration", 3).await?;
    client.observe("media-title", 4).await?;

    client.load_file("https://www.youtube.com/watch?v=Mxa4EhLWUL4").await?;
    client.get_property("playlist").await?;
    client.unpause().await?;

    loop {
        sleep(Duration::from_secs(1));
        println!(".");
    }
}
