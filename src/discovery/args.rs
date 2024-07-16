use clap::Parser;
use crate::SocketKind;
use std:: net::{Ipv4Addr, Ipv6Addr};

#[derive(Parser)]
pub struct FindNodesArgs {
    /// Type of socket to bind ['ds', 'ip4', 'ip6'].
    #[clap(long, default_value_t = SocketKind::Ds)]
    pub socket_kind: SocketKind,
    /// IpV4 to advertise in the ENR. This is needed so that other IpV4 nodes can connect to us.
    #[clap(long)]
    pub enr_ip4: Option<Ipv4Addr>,
    /// IpV6 to advertise in the ENR. This is needed so that other IpV6 nodes can connect to us.
    #[clap(long)]
    pub enr_ip6: Option<Ipv6Addr>,
    /// Port to bind. If none is provided, a random one in the 9000 - 9999 range will be picked
    /// randomly.
    #[clap(long)]
    pub port: Option<u16>,
    /// Port to bind for ipv6. If none is provided, a random one in the 9000 - 9999 range will be picked
    /// randomly.
    #[clap(long)]
    pub port6: Option<u16>,
    /// Use a default test key.
    #[clap(long)]
    pub use_test_key: bool,
    /// A remote peer to try to connect to. Several peers can be added repeating this option.
    #[clap(long)]
    pub remote_peer: Vec<discv5::Enr>,
    /// Use this option to turn on printing events received from discovery.
    #[clap(long)]
    pub events: bool
}

pub fn parse_args() -> FindNodesArgs {
    FindNodesArgs::parse()
}