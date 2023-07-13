use log::{error, info};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::thread;

use crate::error::Result;

use super::cluster_client::ClusterClient;
use super::manager::Manager;

/// Run the server listening on the given address
pub fn listen<A: ToSocketAddrs>(
    alias: String,
    addr: A,
    conn_manager: Arc<RwLock<Manager>>,
) -> Result<()> {
    info!("Initializing Cluster server...");
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        info!("New Node connected!");
        let _alias = alias.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || -> Result<()> {
                    let mut client = ClusterClient::new(stream, _alias)?;
                    client.run()?;

                    Ok(())
                });
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
    Ok(())
}
