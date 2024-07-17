use std::{fs::File, io::BufReader, str::FromStr};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use tracing::info;
use discv5::{Enr, Discv5};
use std::env;
use crate::dht::node::Node;

use super::service::derive_info;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BootstrapStore {
    /// The list of bootstrap nodes.
    pub data: Vec<BootstrapNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BootstrapNode {
    pub enr: String,
}

pub async fn boostrap(discv5: &mut Discv5, file: Option<String>) -> eyre::Result<()> {
    if let Some(f) = file {
        // Read the JSON bootstrap file
        println!("Current directory: {:?}", env::current_dir()?);
        info!("File : {}", f);
        let file = File::open(f)?;
        let reader = BufReader::new(file);
        let bootstrap_store: BootstrapStore = serde_json::from_reader(reader)?;

        // For each bootstrap node, try to connect to it.
        for node in bootstrap_store.data {
            // Skip over invalid enrs
            if let Ok(enr) = Enr::from_str(&node.enr) {
                let node_id = enr.node_id();
                match discv5.add_enr(enr) {
                    Err(_) => { /* log::warn!("Failed to bootstrap node with id: {node_id}") */ }
                    Ok(_) => {
                        info!("Bootstrapped node: {node_id}");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Checks if a bootstrap file exists and returns the ENR of the first node if available.
pub fn get_bootstrap_if_exists(file: Option<String>) -> Option<Node> {
    if let Some(f) = file {
        println!("Current directory: {:?}", env::current_dir());
        info!("File : {}", f);

        // Handling file open and read errors safely
        let file = match File::open(&f) {
            Ok(file) => file,
            Err(_) => {
                info!("Failed to open file: {}", f);
                return None;
            }
        };
        let reader = BufReader::new(file);
        let bootstrap_store: BootstrapStore = match serde_json::from_reader(reader) {
            Ok(store) => store,
            Err(_) => {
                info!("Failed to deserialize bootstrap data from file.");
                return None;
            }
        };

        // Process the first node if available
        if let Some(first_node) = bootstrap_store.data.first() {
            let enr = Enr::from_str(&first_node.enr);
            let node_info = derive_info(&enr.unwrap());
            if let Some(ip) = node_info.udp4{
                let node = Node::new(ip.ip().to_string(), ip.port());
                return Some(node)
            }else{
               return None
            }
        }
    }
    None
}