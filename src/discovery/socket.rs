#[derive(Clone)]
pub enum SocketKind {
    Ip4,
    Ip6,
    Ds,
}

impl std::fmt::Display for SocketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketKind::Ip4 => f.write_str("ip4"),
            SocketKind::Ip6 => f.write_str("ip6"),
            SocketKind::Ds => f.write_str("ds"),
        }
    }
}

impl std::str::FromStr for SocketKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ip4" => Ok(SocketKind::Ip4),
            "ip6" => Ok(SocketKind::Ip6),
            "ds" => Ok(SocketKind::Ds),
            _ => Err("bad kind"),
        }
    }
}
