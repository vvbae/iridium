use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    thread,
};

use crate::{
    error::{IridiumError, Result},
    repl::{self, REPL},
};

pub struct Client {
    repl: repl::REPL,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    stream: TcpStream,
}

impl Client {
    /// Create new client with writer and reader from TcpStream
    pub fn new(stream: TcpStream) -> Result<Self> {
        let tcp_reader = stream.try_clone()?;
        let tcp_writer = stream.try_clone()?;
        Ok(Self {
            reader: BufReader::new(tcp_reader),
            writer: BufWriter::new(tcp_writer),
            repl: REPL::new(),
            stream,
        })
    }

    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(e) => {
                    println!("Error flushing to client: {}", e);
                    false
                }
            },
            Err(e) => {
                println!("Error writing to client: {}", e);
                false
            }
        }
    }

    /// Write ">>>"
    fn write_prompt(&mut self) {
        self.w(repl::PROMPT);
    }

    /// Listen for input and send to client
    fn recv_loop(&mut self) -> Result<()> {
        let rx = self.repl.rx_pipe.take();
        let writer = self.stream.try_clone()?;
        thread::spawn(move || -> Result<()> {
            let chan = rx.unwrap();
            let mut writer = BufWriter::new(writer);
            loop {
                match chan.recv() {
                    Ok(msg) => {
                        writer.write_all(msg.as_bytes())?;
                        writer.flush()?;
                        Ok(())
                    }
                    Err(e) => Err(IridiumError::Recv(e)),
                }?;
            }
        });

        Ok(())
    }

    /// Set up REPL for client
    pub fn run(&mut self) -> Result<()> {
        self.recv_loop()?;
        let mut buf = String::new();
        let banner = repl::REMOTE_BANNER.to_owned() + "\n" + repl::PROMPT;
        self.w(&banner);
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_end();
                    self.repl.run_single(&buf)?;
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}
