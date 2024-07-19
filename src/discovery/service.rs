use std::net::SocketAddrV4;

use crate::{dht::protocol::Protocol, info, warn};
use discv5::{
    enr::{self, CombinedKey, NodeId},
    Discv5,
};

pub struct NodeInfo {
    pub node_id: NodeId,
    pub udp4: Option<SocketAddrV4>,
}

pub async fn start_discv5_service(
    enr: enr::Enr<CombinedKey>,
    enr_key: CombinedKey,
    config: discv5::Config,
) -> Discv5 {
    let discv5 = Discv5::new(enr, enr_key, config).unwrap();
    discv5
}

pub fn derive_id_from_enr(enr: &enr::Enr<CombinedKey>) -> Option<String> {
    if let Some(node_id) = enr.id() {
        return Some(node_id);
    }
    None
}

pub async fn lookup_nodes(discv5: &Discv5) {
    // Initiate peer search
    let target_random_node_id = enr::NodeId::random();
    match discv5.find_node(target_random_node_id).await {
        Err(e) => {
            warn!(error = ?e, "Find Node result failed")
        }
        Ok(v) => {
            // found a list of ENR's print their NodeIds
            let node_ids = v.iter().map(|enr| enr.node_id()).collect::<Vec<_>>();
            info!(len = node_ids.len(), "Nodes found");
            for node_id in node_ids {
                info!(%node_id, "Node");
            }
        }
    }
    let metrics = discv5.metrics();
    let connected_peers = discv5.connected_peers();
    info!(
        connected_peers,
        active_sessions = metrics.active_sessions,
        unsolicited_requests_per_second =
            format_args!("{:.2}", metrics.unsolicited_requests_per_second),
        "Current connected peers: "
    );
}

pub async fn talk(
    discv5: &Discv5,
    interface: &Protocol, // Assuming Protocol is a trait or struct with necessary methods
    enr: &enr::Enr<CombinedKey>,
    protocol: &[u8], // "peer_size".as_bytes() would be passed here
) -> Result<(), String> {
    // Convert the local node's ID and peer size to a byte array
    let self_id = discv5.local_enr().node_id().to_string();
    let peer_count = discv5.connected_peers().to_string();
    let request_data = format!("{}|{}", self_id, peer_count).as_bytes().to_vec();

    // Send the talk request
    match discv5
        .talk_req(enr.clone(), protocol.to_vec(), request_data)
        .await
    {
        Ok(talk_response) => {
            // Process the talk response
            // Assuming the response is also in the form "node_id|peer_size"
            if let Ok(response_str) = std::str::from_utf8(&talk_response) {
                let parts: Vec<&str> = response_str.split('|').collect();
                if parts.len() == 2 {
                    let remote_node_id = parts[0];
                    let remote_peer_size = parts[1];

                    // Log or handle the received data
                    println!("Received peer size from {}: {}", remote_node_id, remote_peer_size);

                    // Optionally update interface or perform additional actions
                    interface.put(remote_node_id.to_string(), remote_peer_size.to_string());
                }
            }
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}


pub fn derive_info(enr: &enr::Enr<CombinedKey>) -> NodeInfo {
    let node_id = enr.node_id();
    let udp_port = enr.udp4_socket();

    NodeInfo {
        node_id: node_id,
        udp4: udp_port,
    }
}
