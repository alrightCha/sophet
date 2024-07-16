//! Demonstrates how to run a basic Discovery v5 Service.
//!
//! This example creates a discv5 service which searches for peers every 30 seconds. On creation,
//! the local ENR created for this service is displayed in base64. This can be used to allow other
//! instances to connect and join the network. The service can be stopped by pressing Ctrl-C.
//!
//! To add peers to the network, create multiple instances of this service adding the ENR of a
//! participating node in the command line. The nodes should discover each other over a period of
//! time. (It is probabilistic that nodes to find each other on any given query).
//!
//! See the example's help with
//! ```
//! sh cargo run --example find_nodes -- --help
//! ```
//!
//! For a simple CLI discovery service see [discv5-cli](https://github.com/AgeManning/discv5-cli)

mod discovery;
use discovery::args::parse_args;
use discovery::enr_builder::build_enr;
use discovery::service::{lookup_nodes, start_discv5_service, derive_info};
use discovery::SocketKind;
use discovery::bootstrap::boostrap;
use discv5::{
    enr,
    enr::{k256, CombinedKey},
    ConfigBuilder, Discv5, Event, ListenConfig,
};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use tokio::io::{self, stdin, AsyncBufReadExt, BufReader};
use tracing::{info, instrument::WithSubscriber, warn};


#[tokio::main]
async fn main() {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .try_init();

    let args = parse_args();
    let port = args
        .port
        .unwrap_or_else(|| (rand::random::<u16>() % 1000) + 9000);
    let port6 = args.port.unwrap_or_else(|| loop {
        let port6 = (rand::random::<u16>() % 1000) + 9000;
        if port6 != port {
            return port6;
        }
    });

    let enr_key = CombinedKey::generate_secp256k1();

    let enr = build_enr(&args, &enr_key, port, port6);
    // the address to listen on.
    let listen_config = match args.socket_kind {
        SocketKind::Ip4 => ListenConfig::from_ip(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
        SocketKind::Ip6 => ListenConfig::from_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port6),
        SocketKind::Ds => ListenConfig::default()
            .with_ipv4(Ipv4Addr::UNSPECIFIED, port)
            .with_ipv6(Ipv6Addr::UNSPECIFIED, port6),
    };

    // default configuration with packet filtering
    let config = ConfigBuilder::new(listen_config)
        .enable_packet_filter()
        .build();

    info!("Node Id: {}", enr.node_id());
    if args.enr_ip6.is_some() || args.enr_ip4.is_some() {
        // if the ENR is useful print it
        info!(
            base64_enr = &enr.to_base64(),
            ipv6_socket = ?enr.udp6_socket(),
            ipv4_socket = ?enr.udp4_socket(),
            "Local ENR",
        );
    }

    let mut discv5 = start_discv5_service(enr, enr_key, config).await;
    // construct the discv5 server
    let bootstrap_file = Some("bootstrap.json".to_string());
    // if we know of another peer's ENR, add it known peers
    boostrap(&mut discv5, bootstrap_file.clone()).await.unwrap();

    // start the discv5 service
    discv5.start().await.unwrap();
    let mut event_stream = discv5.event_stream().await.unwrap();
    let check_evs = args.events;

    // construct a 30 second interval to search for new peers.
    let mut query_interval = tokio::time::interval(Duration::from_secs(30));

    //Implement logic to lookup votes and update accordingly 
    //Implement logic to 
    loop {
        tokio::select! {
            Some(discv5_ev) = event_stream.recv() => {
                // consume the events from disc
                if !check_evs {
                    continue;
                }
                match discv5_ev {
                    Event::Discovered(enr) => {
                        //Derive ip address and port from enr as well as storage
                        info!(%enr, "Enr discovered");
                        let enr_info = derive_info(&enr);
                        if let Some(multi_addr) = enr_info.udp4{
                            info!("Info derived: {} {}", enr_info.node_id , multi_addr);
                        }
                    }, 
                    Event::NodeInserted { node_id, replaced: _ } => info!(%node_id, "Node inserted"), //derive 
                    Event::SessionEstablished(enr, _) => info!(%enr, "Session established"),
                    Event::SocketUpdated(addr) => info!(%addr, "Socket updated"), //Find key in dht and update addr + port 
                    Event::TalkRequest(_) => info!("Talk request received"), // Check if dht still has storage
                    _ => {}
                };
            }
            _ = query_interval.tick() => {
                // execute a FINDNODE query every 30 seconds to register new nodes and update routing table
                lookup_nodes(&discv5).await;
            }
        }
    }
}