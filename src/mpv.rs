#[cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::{io, thread};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, exit};
use std::str::{from_utf8};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::time::Duration;

use serde_json::{Number, Value};
use serde::{Serialize, Deserialize};
use serde_json::Value::{String as JsonString, Number as JsonNumber};
use crate::mpv::MpvEvent::{PropertyChange};
use crate::mpv::MpvResponse::{CommandResponse, Event, UnknownResponse};


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MpvResponse {
    Event(MpvEvent),
    CommandResponse(MpvCommandResponse),
    UnknownResponse(String),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "kebab-case")]
pub enum MpvEvent {
    Pause,
    Unpause,
    PropertyChange { name: String, data: f64 },
    UnknownEvent(String),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "error")]
#[serde(rename_all = "kebab-case")]
pub enum MpvCommandResponse {
    Success { data: Value, request_id: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpvCommand {
    command: Vec<Value>,
}

#[allow(dead_code)]
pub struct MpvClient {
    cmd_sender: Sender<MpvCommand>,
    kill_channel: Option<Sender<()>>,
    event_bus_receiver: Receiver<MpvResponse>,
}

///
///
/// # Arguments
///
/// * `message`:
///
/// returns: MpvResponse
///
/// # Examples
///
/// ```
///
/// ```
fn _map_message_to_event(message: &str) -> MpvResponse {
    match serde_json::from_str::<MpvEvent>(&message) {
        Ok(event) => Event(event),
        Err(_) => {
            match serde_json::from_str::<MpvCommandResponse>(&message) {
                Ok(event) => CommandResponse(event),
                Err(_) => UnknownResponse(message.to_string()),
            }
        }
    }
}

impl MpvClient {
    pub fn new(stream: UnixStream, child: Option<Child>) -> Result<Self, io::Error> {
        let kill_channel = match child {
            None => None,
            Some(mut child2) => {
                let (tx, rx) = mpsc::channel::<()>();
                thread::spawn(move || {
                    if rx.recv().is_ok() {
                        child2.kill().unwrap();
                        child2.wait().unwrap();
                    }
                });
                Some(tx)
            }
        };
        let res = stream.try_clone()?;
        let mut writer = BufWriter::new(stream);
        let (cmd_sender, rx_cmd) = mpsc::channel::<MpvCommand>();

        thread::spawn(move || {
            loop {
                match rx_cmd.recv() {
                    Ok(msg) => {
                        let mut command = serde_json::to_string(&msg)?.as_bytes().to_vec();

                        command.push(0x0a);
                        return match writer.write(command.as_slice()) {
                            Ok(size) => {
                                writer.flush().unwrap();
                                return Ok(size);
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => {
                        println!("Got fatal error: {}", e);
                        exit(-1);
                    }
                }
            }
        });


        let (tx, rx) = mpsc::channel::<MpvResponse>();
        let tx_c = tx.clone();

        let mut reader = BufReader::new(res);

        thread::spawn(move || {
            loop {
                let mut data = Vec::new();
                match reader.read_until(0x0a, &mut data) {
                    Ok(_) => {
                        match from_utf8(data.as_slice()) {
                            Ok(str) => {
                                let message = _map_message_to_event(&str);

                                match message {
                                    Event(PropertyChange { ref name, data: _data }) if name == "time-pos" => {
                                        tx_c.send(message).unwrap_or(())
                                    },
                                    message => tx_c.send(message).unwrap_or(())
                                };
                            }
                            _ => {}
                        }
                    },
                    Err(err) => println!("Unable to read line: {}", err),
                }
            }
        });


        thread::sleep(Duration::from_secs(1));

        return Ok(Self {
            cmd_sender,
            kill_channel,
            event_bus_receiver: rx,
        });
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), SendError<MpvCommand>> {
        return self.command(vec![
            JsonString("loadfile".to_string()),
            JsonString(path.to_string())
        ]);
    }

    pub fn events(&self) -> &Receiver<MpvResponse> {
        return &self.event_bus_receiver;
    }

    pub fn kill(&self) -> Result<bool, SendError<()>> {
        return match &self.kill_channel {
            None => Ok(false),
            Some(process) => process.send(()).and(Ok(true)),
        }
    }

    pub fn make_killer(&self) -> Option<Sender<()>> {
        return match &self.kill_channel {
            None => None,
            Some(process) => Some(process.clone()),
        }
    }


    fn command(&mut self, cmd: Vec<Value>) -> Result<(), SendError<MpvCommand>> {
        let cmd_struct = MpvCommand {
            command: cmd
        };

        return self.cmd_sender.send(cmd_struct);
    }
}
