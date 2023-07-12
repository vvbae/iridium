use std::net::{TcpListener, ToSocketAddrs};
use std::thread;

use log::error;

use crate::error::Result;
use crate::remote::client::Client;

pub struct Server {}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    /// Run the server listening on the given address
    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| -> Result<()> {
                        let mut client = Client::new(stream)?;
                        client.run()?;

                        Ok(())
                    });
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}
