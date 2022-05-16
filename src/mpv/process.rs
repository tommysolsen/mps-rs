
use std::{fs};
use std::io::Error;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use tokio::net::UnixStream;
use tokio::process::{Child, Command};


#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct MpvInitializer {
    mpv_args: Option<Vec<String>>
}

impl MpvInitializer {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.mpv_args = Some(args);
        self
    }
}

#[async_trait]
pub trait MpvProcess {
    async fn create_connection(&self) -> Result<UnixStream, Error>;
    fn kill_program(&self) -> Result<(), Error>;
}


pub struct ExistingProcess {
    socket_location: String,
}

impl ExistingProcess {

    #[allow(dead_code)]
    pub fn new(ipc_path: &str) -> Self {
        Self {
            socket_location: ipc_path.to_string(),
        }
    }
}

#[async_trait]
impl MpvProcess for ExistingProcess {
    async fn create_connection(&self) -> Result<UnixStream, Error> {
        return UnixStream::connect(&self.socket_location).await;
    }

    fn kill_program(&self) -> Result<(), Error> {
        Ok(())
    }
}


pub struct ManagedProcess {
    sock_path: String,
    child: Arc<Mutex<Child>>,
}

/// Attempt to kill the mpv instance if this process is ever dropped from the program.
/// It should really only happen if the program panics!

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        self.kill_program().unwrap();

        let file = Path::new(&self.sock_path);
        if file.exists() {
            fs::remove_file(file).unwrap();
        }
    }
}

impl ManagedProcess {
    pub fn new(args: MpvInitializer) -> Result<Self, Error> {
        let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let path = format!("/tmp/mpv-{}", epoch);

        let extra_args = args.mpv_args.unwrap_or_default();
        let command = Command::new("mpv").arg(format!("--input-ipc-server={}", path)).arg("--idle").arg("--really-quiet").args(extra_args).stdout(Stdio::null()).spawn()?;

        Ok(Self {
            child: Arc::new(Mutex::new(command)),
            sock_path: path,
        })
    }
}


#[async_trait]
impl MpvProcess for ManagedProcess {
    async fn create_connection(&self) -> Result<UnixStream, Error> {
        return UnixStream::connect(&self.sock_path).await;
    }

    fn kill_program(&self) -> Result<(), Error> {
        let value = self.child.lock().unwrap();

        if let Some(value) = &value.id() {
            let pid_string = value.to_string();
            Command::new("kill").arg(pid_string).spawn()?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn existing_client(path: &str) -> ExistingProcess {
    ExistingProcess {
        socket_location: path.to_string(),
    }
}
