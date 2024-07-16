use std::net::SocketAddrV4;

use crate::{info, warn};
use discv5::{enr::{self, CombinedKey, NodeId}, Discv5};

pub struct NodeInfo{
    pub node_id: NodeId,
    pub udp4: Option<SocketAddrV4>
}

pub async fn start_discv5_service(
    enr: enr::Enr<CombinedKey>,
    enr_key: CombinedKey,
    config: discv5::Config,
) -> Discv5 {
    let discv5 = Discv5::new(enr, enr_key, config).unwrap();
    discv5
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

pub fn derive_info(enr: &enr::Enr<CombinedKey>) -> NodeInfo {
    let node_id = enr.node_id();
    let udp_port = enr.udp4_socket();

    NodeInfo {
        node_id: node_id,
        udp4: udp_port
    }
}
