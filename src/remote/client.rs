use std::{
    io::{BufRead, BufReader, BufWriter},
    net::TcpStream,
    thread,
};

use crate::{
    common::w,
    error::{IridiumError, Result},
    repl::{self, REPL},
    vm::VM,
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
            repl: REPL::new(VM::new()),
            stream,
        })
    }

    /// Write ">>>"
    fn write_prompt(&mut self) -> Result<()> {
        w(&mut self.writer, repl::PROMPT)?;
        Ok(())
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
                    Ok(msg) => w(&mut writer, &msg),
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
        w(&mut self.writer, &banner)?;
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
