use serde::{Deserialize, Serialize};

use super::NodeAlias;

#[derive(Debug, Serialize, Deserialize)]
pub enum IridiumMessage {
    Hello {
        alias: NodeAlias, // node alias of the node that wants to join the cluster
    },
    HelloAck {
        alias: NodeAlias,                        // Receiver alias
        nodes: Vec<(NodeAlias, String, String)>, // list of nodes (alias, IP, port)
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HelloResponse {
    Ok(String),
    Err(String),
}
