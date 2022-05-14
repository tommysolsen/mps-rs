use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::str::{from_utf8, Utf8Error};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use serde_json::Value::String as ValueString;
use crate::mpv::ReadCommandErrors::Utf8Error as mUtf8Error;

#[derive(Debug)]
pub enum ReadCommandErrors {
    IOError(io::Error),
    Utf8Error(Utf8Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpvCommand {
    command: Vec<Value>,
}

pub struct MpvClient {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>
}

impl MpvClient {
    pub fn new(stream: UnixStream) -> Result<Self, io::Error> {
        stream.set_nonblocking(true)?;
        let res = stream.try_clone()?;
        Ok(Self {
            reader: BufReader::new(res),
            writer: BufWriter::new(stream),
        })
    }

    pub fn loadFile(&mut self, path: &str) -> Result<usize, io::Error> {
        return self.command(vec![
            ValueString("loadfile".to_string()),
            ValueString(path.to_string())
        ]);
    }

    pub fn read_next_command(&mut self) -> Result<String, ReadCommandErrors> {
        let mut data = Vec::new();
        self.reader.read_until(0x0a, &mut data)
            .map_err(|err| crate::mpv::ReadCommandErrors::IOError(err))?;

        return match from_utf8(&data) {
            Ok(val) => Ok(val.to_string()),
            Err(err) => Err(mUtf8Error(err)),
        };
    }

    fn command(&mut self, cmd: Vec<Value>) -> Result<usize, io::Error> {
        let mut command = serde_json::to_string(&MpvCommand {
            command: cmd,
        })?.as_bytes().to_vec();

        command.push(0x0A);
        return match self.writer.write(command.as_slice()) {
            Ok(size) => {
                println!("Wrote command {}, total {} bytes", from_utf8(command.as_slice()).unwrap(), size);
                return Ok(size);
            }
            Err(e) => Err(e),
        }
    }
}
