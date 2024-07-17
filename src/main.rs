//cargo run -- --enr-ip4 [ip address here with no brackets] --port [port with no brackets]
//Make sure to add the enr of a bootsrap node. The ENR is printed at runtime 

//Discv5 packages
mod discovery;
use discovery::args::parse_args;
use discovery::bootstrap::{boostrap, get_bootstrap_if_exists};
use discovery::enr_builder::build_enr;
use discovery::service::{derive_info, lookup_nodes, start_discv5_service};
use discovery::{FindNodesArgs, SocketKind};
use discv5::{enr::CombinedKey, ConfigBuilder, Discv5, Event, ListenConfig};

//DHT packages
mod dht;
use dht::node::Node;
use dht::protocol::Protocol;
use dht::utils;

//Data
mod datatypes;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use datatypes::requests::{RetrieveRequest, StoreRequest};
use std::sync::Arc;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use tracing::{info, warn};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

//DHT functionalities endpoints 
#[post("/store")]
async fn store_data(
    data: web::Json<StoreRequest>,
    dht: web::Data<Arc<Protocol>>,
) -> impl Responder {
    let new_store = StoreRequest {
        key: data.key.clone(),
        value: data.value.clone()
    };
    info!("Received store request {} {}", new_store.key, new_store.value);
    dht.put(new_store.key, new_store.value);
    HttpResponse::Ok().json("Data stored successfully")
}

#[post("/retrieve")]
async fn retrieve_data(
    data: web::Json<RetrieveRequest>,
    dht: web::Data<Arc<Protocol>>,
) -> impl Responder {
    info!("Received get request");
    match dht.get(data.key.clone()) {
        Some(value) => HttpResponse::Ok().json(value),
        None => HttpResponse::NotFound().json("Data not found"),
    }
}

async fn run_discovery_loop(discv5: Discv5, interface: Arc<Protocol>) {
    let mut event_stream = discv5.event_stream().await.unwrap();

    // construct a 30 second interval to search for new peers.
    let mut query_interval = tokio::time::interval(Duration::from_secs(30));

    //Implement logic to lookup new nodes and managing our DHT accordingly
    loop {
        tokio::select! {
            _ = query_interval.tick() => {
                // execute a FINDNODE query every 30 seconds to register new nodes and update routing table
                lookup_nodes(&discv5).await;
            }
            Some(discv5_ev) = event_stream.recv() => {
                match discv5_ev {
                    Event::Discovered(enr) => {
                        //Derive ip address and port from enr as well as node ID
                        info!(%enr, "Enr discovered");
                        //Pinging new discovered node to store it in our dht
                        let enr_info = derive_info(&enr);
                        if let Some(ip) = enr_info.udp4{
                            let node = Node::new(ip.ip().to_string(), ip.port() + 1);
                            let res = interface.ping(node);
                            if res{
                                info!("Node stored under our DHT successfully");
                            }else{
                                info!("Failed at receiving PONG response. Node not stored.");
                            }
                        }else{
                            info!("Could not derive node information")
                        }
                    },
                    Event::NodeInserted { node_id, replaced: _ } => info!(%node_id, "Node inserted"), //derive
                    Event::SessionEstablished(enr, _) => info!(%enr, "Session established"),
                    Event::SocketUpdated(addr) => info!(%addr, "Socket updated"), //Find key in dht and update addr + port
                    Event::TalkRequest(_) => info!("Talk request received"), // Check if dht still has storage
                    _ => {}
                };
            }

        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .try_init();

    //Deriving node information from args passed
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

    //Generating fresh ENR for node
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

    //Initializing discv5 server
    let mut discv5 = start_discv5_service(enr, enr_key, config).await;
    // construct the discv5 server
    let bootstrap_file = Some("bootstrap.json".to_string());
    // if we know of another peer's ENR, add it known peers -> Bootstrap process
    boostrap(&mut discv5, bootstrap_file.clone()).await.unwrap();

    // start the discv5 service
    discv5.start().await.unwrap();

    //Using bootstrap.json to get an optional Node that we pass to our interface to bootstrap
    let bootstrap_result = get_bootstrap_if_exists(bootstrap_file);
    //Starting root with local ip address and port + 1
    let root = Node::new(utils::get_local_ip().unwrap(), 8001);

    //DHT interface responsible for adding nodes and data
    let dht_protocol = Arc::new(Protocol::new(
        root.ip.clone(),
        root.port.clone(),
        bootstrap_result,
    ));

    // Clone the Arc for use in the discovery loop on a separate thread for shared state
    let dht_protocol_for_loop = dht_protocol.clone();
    tokio::spawn(run_discovery_loop( discv5, dht_protocol_for_loop));

    //Exposing external api to interact with the dht
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(dht_protocol.clone()))
            .service(store_data)
            .service(retrieve_data)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
