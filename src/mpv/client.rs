use std::io;
use std::cmp::max;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::str::from_utf8;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::unix::{OwnedWriteHalf};
use tokio::sync::{Mutex, oneshot};
use tokio::sync::oneshot::channel;
use crate::mpv::commands::MpvCommandResponse;
use crate::MpvProcess;

pub struct Client {
    pending_futures: Arc<Mutex<HashMap<u64, oneshot::Sender<MpvCommandResponse>>>>,
    writer: BufWriter<OwnedWriteHalf>
}

impl Client {
    pub async fn new<T: MpvProcess>(process: T) -> Result<Self, io::Error> {
        let socket = process.create_connection().await?;

        let (reader, writer) = socket.into_split();
        let writer = BufWriter::new(writer);
        let mut reader = BufReader::new(reader);

        let pending_futures: Arc<Mutex<HashMap<u64, oneshot::Sender<MpvCommandResponse>>>> = Arc::new(Mutex::new(HashMap::new()));

        let futures_buffer = pending_futures.clone();
        tokio::spawn(async move {
            loop {
                let mut buffer: Vec<u8> = Vec::new();
                let _results = reader.read_until(0x0a, &mut buffer).await;
                let message = serde_json::from_slice::<MpvCommandResponse>(buffer.as_slice());

                match message {
                    Ok(parsed) => {
                        let mut futures = futures_buffer.lock().await;
                        if futures.contains_key(&parsed.request_id) {
                            let response_channel = futures.remove(&parsed.request_id).unwrap();
                            response_channel.send(parsed).unwrap();
                        }
                    }
                    _ => {
                        let str_msg = from_utf8(buffer.as_slice()).unwrap();
                        println!("Got event {}", str_msg);
                    }
                }

                ()
            }
        });

        Ok(Self {
            writer,
            pending_futures,
        })
    }

    pub async fn load_file(&mut self, path: &str) -> Result<MpvCommandResponse, io::Error> {
        let cmd = format!("[\"loadfile\", \"{}\"]", path);
        return self._perform_command(cmd.as_str()).await;
    }

    pub async fn get_property(&mut self, property: &str) -> Result<MpvCommandResponse, io::Error> {
        let cmd = format!("[\"get_property\", \"{}\"]", property);
        return self._perform_command(cmd.as_str()).await;
    }

    pub async fn observe(&mut self, property: &str, observer_id: u16) -> Result<MpvCommandResponse, io::Error> {
        let cmd = format!("[\"observe_property\", {}, \"{}\"]", observer_id, property);
        return self._perform_command(cmd.as_str()).await;
    }

    async fn _perform_command(&mut self, command: &str) -> Result<MpvCommandResponse, io::Error> {
        println!("Registering response");
        let (req_id, response) = self._register_future().await?;
        let cmd = format!("{{\"command\":{}, \"request_id\": {}, \"async\": true}}\n", command, req_id);

        println!("-> {}", cmd);
        self.writer.write(cmd.as_bytes()).await?;
        self.writer.flush().await?;
        let response = response.await.map_err(|err| io::Error::new(ErrorKind::InvalidData, err));
        println!("<- {:?}", response);
        return response;
    }

    async fn _register_future(&self) -> Result<(u64, oneshot::Receiver<MpvCommandResponse>), io::Error> {
        let mut lock = self.pending_futures.lock().await;
        let highest_value = lock.keys().fold(0_u64, |p, c| max(p, *c)) + 1;
        let (tx, rx) = channel::<MpvCommandResponse>();
        lock.insert(highest_value, tx);
        return Ok((highest_value, rx));
    }

    async fn _unregister_future(&self, req_id: u64) {
        let mut lock = self.pending_futures.lock().await;
        if lock.contains_key(&req_id) {
            lock.remove(&req_id);
        }
    }
}


