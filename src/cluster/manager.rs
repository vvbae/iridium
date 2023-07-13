use std::collections::HashMap;

use log::error;

use super::{cluster_client::ClusterClient, NodeAlias};

#[derive(Default)]
pub struct Manager {
    clients: HashMap<String, ClusterClient>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            clients: HashMap::new(),
        }
    }

    /// Adds a client as cluster member
    pub fn add_client(&mut self, alias: NodeAlias, client: ClusterClient) -> bool {
        if self.clients.contains_key(&alias) {
            error!("Tried to add a client that already existed");
            return false;
        }
        self.clients.insert(alias, client);
        true
    }

    /// Delete a client by alias
    pub fn del_client(&mut self, alias: NodeAlias) -> bool {
        if !self.clients.contains_key(&alias) {
            error!("Tried to delete a client that doesn't exist");
            return false;
        }
        self.clients.remove(&alias);
        true
    }

    /// Get client names
    pub fn get_client_names(&self) -> Vec<String> {
        self.clients.keys().map(|k| k.to_owned()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::Manager;

    fn test_create_manager() {
        let test_manager = Manager::new();
    }
}
