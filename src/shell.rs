use std::{fs, io};
use std::fs::File;
use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::os::unix::net::{UnixStream};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value};
use serde_json::Value::{Number as ValueNumber, String};
use crate::mpv::MpvClient;


struct MpvResponse {
    error: std::string::String,
    data: Value,
}

#[derive(Debug, Deserialize)]
enum MpvResponseData {
    Float(f32),
    Int(i64),
    String(std::string::String)
}

pub fn initialize() -> Result<(Child, MpvClient), io::Error> {
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let path = format!("/tmp/mpv-{}", epoch);
    let child = start_mpv(path.as_str())?;
    sleep(Duration::from_secs(1));
    let socket = UnixStream::connect(path)?;

    let client = crate::mpv::MpvClient::new(socket)?;
    return Ok((child, client));
}


/// Returns a new mpv instance child with a open ipc server bound on a unix sock file.
/// If the sock file does not exist it will be created
///
/// # Arguments
///
/// * `path`:
///
/// returns: Result<Child, Error>
///
/// # Examples
///
/// ```
///
/// ```
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn start_mpv(path: &str) -> Result<Child, std::io::Error> {
    // match fs::metadata(path) {
    //     Ok(_) => Ok(()),
    //     Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
    //         match File::create(path).err() {
    //             None => Ok(()),
    //             Some(e) => Err(e),
    //         }
    //     },
    //     Err(e) => Err(e),
    // }?;

    let command = Command::new("mpv")
        .arg(format!("--input-ipc-server={}", path))
        .arg("--idle")
        .arg("--force-window")
        .arg("--really-quiet")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;

    return Ok(command);
}
