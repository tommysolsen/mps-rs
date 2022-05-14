
use std::io::{BufRead, BufReader, Read, stdout, Write};
use std::os::unix::net::{UnixStream};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Number, Value};
use serde_json::Value::{Number as ValueNumber, String};

#[derive(Debug, Serialize, Deserialize)]
pub struct MpvCommand {
    command: Vec<serde_json::Value>,
}

struct MpvResponse {
    error: std::string::String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
enum MpvResponseData {
    Float(f32),
    Int(i64),
    String(std::string::String)
}

#[cfg(target_os = "macos")]
pub fn start_mpv() {

    // File::create(Path::new("/tmp/mpvsocket")).expect("Unable to create socket file");
    // let command = Command::new("mpv")
    //     .args([
    //         "--input-ipc-server=/tmp/mpvsocket",
    //         "--for"
    //     ]).output().expect("Unable to run mpv");
    //
    // let mut x = stdout();
    // x.write(command.stderr.as_slice());
    // x.flush();


    let mut sock = UnixStream::connect("/tmp/mpvsocket").expect("Unable to connect to mpv server via ");

    let cmd_i = serde_json::Number::from(1_u16);

    let mut x = serde_json::to_string(&MpvCommand { command: vec![
        String("observe_property".to_string()),
        ValueNumber(cmd_i),
        String("time-pos".to_string())
        ], }).expect("Expected paylod to be serializable");

    writeln!(sock, "{}", x);


    let mut reader = BufReader::new(sock);
    loop {
        let mut st: Vec<u8> = Vec::new();
        let mut response: Vec<u8> = Vec::new();
        reader.read_until(0x0a, &mut st).expect("Could not read from socket");
        let x = std::string::String::from_utf8(st).unwrap();
        println!("Got {}", x);
    }





}
