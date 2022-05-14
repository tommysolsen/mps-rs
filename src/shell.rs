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
use serde_json::Value::{Number as ValueNumber, String as ValueString};
use crate::mpv::MpvClient;

#[derive(Default)]
pub struct InitializerSettings {
    existing_socket: Option<String>,
}

impl InitializerSettings {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn set_existing_socket(mut self, socket_path: &str) -> Self {
        self.existing_socket = Some(socket_path.to_string());
        return self;
    }

}

pub fn initialize(settings: InitializerSettings) -> Result<MpvClient, io::Error> {
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    return if settings.existing_socket.is_none() {
        let path = format!("/tmp/mpv-{}", epoch);
        let mpv = start_mpv(&path)?;
        sleep(Duration::from_secs(1));
        let socket = UnixStream::connect(path).unwrap();
        MpvClient::new(socket, Some(mpv))
    } else {
        let path = settings.existing_socket.unwrap();
        let socket = UnixStream::connect(path)?;
        MpvClient::new(socket, None)
    }
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
