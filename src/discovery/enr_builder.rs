
use crate::discovery::args::FindNodesArgs;
use crate::discovery::{parse_args};
use discv5::{enr,enr::CombinedKey};
use std:: net::{Ipv4Addr, Ipv6Addr};

pub fn build_enr(
    args: &FindNodesArgs,
    enr_key: &CombinedKey,
    port: u16,
    port6: u16,
) -> enr::Enr<CombinedKey> {
    // Clone the key to use in the ENR builder
    let mut builder = enr::Enr::builder();
    if let Some(ip4) = args.enr_ip4 {
        if ip4.is_unspecified() {
            builder.ip4(Ipv4Addr::LOCALHOST).udp4(port);
        } else {
            builder.ip4(ip4).udp4(port);
        }
    }
    if let Some(ip6) = args.enr_ip6 {
        if ip6.is_unspecified() {
            builder.ip6(Ipv6Addr::LOCALHOST).udp6(port6);
        } else {
            builder.ip6(ip6).udp6(port6);
        }
    }
    // Pass the cloned CombinedKey directly to the build method
    builder.build(enr_key).unwrap()
}