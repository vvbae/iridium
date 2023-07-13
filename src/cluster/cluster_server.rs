use log::{debug, error, info};
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::thread;

use crate::cluster::message::{HelloResponse, IridiumMessage};
use crate::error::Result;

use super::manager::Manager;

pub struct ClusterServer {
    conn_manager: Arc<RwLock<Manager>>,
    alias: String,
}

impl ClusterServer {
    pub fn new(
        alias: String, // server alias
        conn_manager: Arc<RwLock<Manager>>,
    ) -> Self {
        Self {
            conn_manager,
            alias,
        }
    }

    /// Run the server listening on the given address
    pub fn listen<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        info!("Initializing Cluster server...");
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            info!("New Node connected!");
            match stream {
                Ok(stream) => {
                    thread::spawn(move || -> Result<()> {
                        Self::serve(stream)?;
                        Ok(())
                    });
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }

    /// Read messages and write response to the stream
    pub fn serve(tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<IridiumMessage>();

        macro_rules! send_resp {
            ($resp:expr) => {{
                let resp = $resp;
                serde_json::to_writer(&mut writer, &resp)?;
                writer.flush()?;
                debug!("Response sent to {}: {:?}", peer_addr, resp);
            }};
        }

        for req in req_reader {
            let req = req?;
            info!("Receive request from {}: {:?}", peer_addr, req);
            match req {
                IridiumMessage::Hello { alias } => send_resp!(HelloResponse::Ok(format!(
                    "Received hello from node {}",
                    alias
                ))),
                IridiumMessage::HelloAck { alias: _, nodes: _ } => todo!(),
            }
        }
        Ok(())
    }
}
