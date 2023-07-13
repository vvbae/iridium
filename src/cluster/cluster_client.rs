use serde::Deserialize;
use serde_json::{de::IoRead, Deserializer};
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    common::w,
    error::{IridiumError, Result},
    repl::{self},
};

use super::message::{HelloResponse, IridiumMessage};

pub struct ClusterClient {
    pub reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
    rx: Option<Arc<Mutex<Receiver<String>>>>, // add for Arc + Mutex for thread-safety
    tx: Option<Arc<Mutex<Sender<String>>>>, //If something wants to send something to this client, they can clone the `tx` channel.
    stream: TcpStream,
    alias: Option<String>,
}

impl ClusterClient {
    /// Create new client with writer and reader from TcpStream
    pub fn new(
        stream: TcpStream,
        // manager: Arc<RwLock<Manager>>,
        // bind_port: String,
        // alias: String,
    ) -> Result<Self> {
        let tcp_reader = stream.try_clone()?;
        let tcp_writer = stream.try_clone()?;
        let (tx, rx) = channel();
        Ok(Self {
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
            writer: BufWriter::new(tcp_writer),
            stream,
            tx: Some(Arc::new(Mutex::new(tx))),
            rx: Some(Arc::new(Mutex::new(rx))),
            alias: None,
        })
    }

    /// Sets the alias of the ClusterClient and returns it
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Send alias to the cluster just joined
    pub fn send_hello(&mut self) -> Result<()> {
        let msg = IridiumMessage::Hello {
            alias: self.alias.as_ref().unwrap().to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &msg)?;
        self.writer.flush()?;

        Ok(())
    }

    /// Read from server response
    pub fn read(&mut self) -> Result<String> {
        let resp = HelloResponse::deserialize(&mut self.reader)?;
        match resp {
            HelloResponse::Ok(value) => Ok(value),
            HelloResponse::Err(msg) => Err(IridiumError::StringError(msg)),
        }
    }

    /// Write ">>>"
    fn write_prompt(&mut self) -> Result<()> {
        w(&mut self.writer, repl::PROMPT)?;
        Ok(())
    }

    /// Listen for input and send to client
    fn recv_loop(&mut self) -> Result<()> {
        let chan = self.rx.take().unwrap();
        let mut writer = BufWriter::new(self.stream.try_clone()?);

        thread::spawn(move || -> Result<()> {
            if let Ok(locked_rx) = chan.lock() {
                match locked_rx.recv() {
                    Ok(msg) => w(&mut writer, &msg),
                    Err(_e) => Err(IridiumError::Recv(_e)),
                }?;
            }

            Ok(())
        });

        Ok(())
    }

    /// Set up REPL for client
    pub fn run(&mut self) -> Result<()> {
        self.recv_loop()?;
        loop {
            let server_res = self.read()?;
            println!("{}", server_res);
        }
    }
}
