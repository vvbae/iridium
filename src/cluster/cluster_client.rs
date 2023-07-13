use std::{
    io::{BufRead, BufReader, BufWriter},
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

use super::message::IridiumMessage;

#[derive(Debug)]
pub struct ClusterClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    rx: Option<Arc<Mutex<Receiver<String>>>>, // add for Arc + Mutex for thread-safety
    tx: Option<Arc<Mutex<Sender<String>>>>, //If something wants to send something to this client, they can clone the `tx` channel.
    stream: TcpStream,
    alias: String,
}

impl ClusterClient {
    /// Create new client with writer and reader from TcpStream
    pub fn new(
        stream: TcpStream,
        // manager: Arc<RwLock<Manager>>,
        // bind_port: String,
        alias: String,
    ) -> Result<Self> {
        let tcp_reader = stream.try_clone()?;
        let tcp_writer = stream.try_clone()?;
        let (tx, rx) = channel();
        Ok(Self {
            reader: BufReader::new(tcp_reader),
            writer: BufWriter::new(tcp_writer),
            stream,
            tx: Some(Arc::new(Mutex::new(tx))),
            rx: Some(Arc::new(Mutex::new(rx))),
            alias,
        })
    }

    /// Sets the alias of the ClusterClient and returns it
    // pub fn with_alias(mut self, alias: String) -> Self {
    //     self.alias = Some(alias);
    //     self
    // }

    pub fn send_hello(&mut self) {
        let msg = IridiumMessage::Hello {
            alias: self.alias.to_owned(),
        };
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
        let mut buf = String::new();
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_end();
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}
