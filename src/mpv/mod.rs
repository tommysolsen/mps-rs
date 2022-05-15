pub mod events;
pub mod commands;
pub mod client;
pub mod errors;
pub mod process;

#[cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::{io, thread};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, exit};
use std::str::{from_utf8};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, SendError};
use std::time::Duration;

use serde_json::{Number, Value};
use serde::{Serialize, Deserialize};
use serde_json::Value::{String as JsonString, Number as JsonNumber};
use crate::mpv::MpvResponse::{CommandResponse, Event, UnknownResponse};
use crate::mpv::commands::{CommandError, MpvCommandResponse};


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
    PlaybackRestart,
    FileLoaded,
    TracksChanged,
    ChapterChange,
    PropertyChange { name: String, data: f64 },
    UnknownEvent(String),
}


#[derive(Debug, Serialize, Deserialize)]
pub struct MpvCommand {
    command: Vec<Value>,
    request_id: u64,
}

#[allow(dead_code)]
pub struct MpvClient {
    observer_id: u16,
    mpv_pid: Option<u32>,
    cmd_sender: Sender<MpvCommand>,
    kill_channel: Option<Sender<()>>,
    event_bus_receiver: Receiver<MpvResponse>,
    commands: Arc<Mutex<HashMap<u64, Sender<MpvCommandResponse>>>>,
    cmd_id: Arc<Mutex<u64>>,
}

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
        let pid = child.as_ref().map(|f| f.id());

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
        let commands: Arc<Mutex<HashMap<u64, Sender<MpvCommandResponse>>>> = Arc::new(Mutex::new(Default::default()));
        let cmds = commands.clone();
        thread::spawn(move || {
            loop {
                let mut data = Vec::new();
                match reader.read_until(0x0a, &mut data) {
                    Ok(_) => {
                        match from_utf8(data.as_slice()) {
                            Ok(str) => {
                                let message = _map_message_to_event(&str);

                                match message {
                                    CommandResponse(command) => {
                                        let mut command_buffer = cmds.lock().unwrap();
                                        if (*command_buffer).contains_key(&command.request_id) {
                                            let sender = (*command_buffer).get(&command.request_id).unwrap();

                                            let rid = command.request_id;
                                            sender.send(command);
                                            (*command_buffer).remove(&rid);
                                            ()
                                        }
                                    }
                                    x => {
                                        tx_c.send(x).unwrap_or(())
                                    }
                                };
                            },
                            _ => {}
                        }
                    }
                    Err(err) => println!("Unable to read line: {}", err),
                }
            }
        });


        thread::sleep(Duration::from_secs(1));

        return Ok(Self {
            observer_id: 0,
            mpv_pid: pid,
            cmd_sender,
            kill_channel,
            event_bus_receiver: rx,
            commands,
            cmd_id: Arc::new(Mutex::new(0))
        });
    }

    pub fn load_file(&mut self, path: &str) -> Result<MpvCommandResponse, CommandError<SendError<MpvCommand>, RecvError>> {
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

    pub fn observe(&mut self, param: &str) -> Result<MpvCommandResponse, CommandError<SendError<MpvCommand>, RecvError>> {
        let cmd_struct = vec![
            JsonString("observe_property".to_string()),
            JsonNumber(Number::from(self.observer_id)),
            JsonString(param.to_string()),
        ];
        return self.command(cmd_struct);
    }

    pub fn pid(&self) -> Option<u32> {
        return self.mpv_pid;
    }

    fn command(&mut self, cmd: Vec<Value>) -> Result<MpvCommandResponse, CommandError<SendError<MpvCommand>, RecvError>> {
        let (request_id, response) = self.register_command();

        let cmd_struct = MpvCommand {
            command: cmd,
            request_id,
        };

        self.cmd_sender.send(cmd_struct).map_err(|e| CommandError::SendError(e))?;
        return response.recv().map_err(|e| CommandError::ReceiveError(e));
    }


    fn register_command(&self) -> (u64, Receiver<MpvCommandResponse>) {
        let mut cmd_nr = self.cmd_id.lock().unwrap();
        let mut commands = self.commands.lock().unwrap();

        *cmd_nr = (*cmd_nr + 1) & 0x0fffffffffffffff;

        let (tx, rx) = channel::<MpvCommandResponse>();

        commands.insert(*cmd_nr, tx);

        return (*cmd_nr, rx);
    }
}
