use crate::util::print_json;
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Subcommand)]
pub enum NetworkCmd {
    /// Analyze an IPv4/IPv6 CIDR block (JSON)
    Subnet {
        /// CIDR, e.g. 192.168.1.0/24 or 2001:db8::/48
        cidr: String,
    },
    /// First and last address of a CIDR block (JSON)
    CidrToRange {
        /// CIDR, e.g. 10.0.0.0/22
        cidr: String,
    },
    /// Smallest set of CIDR blocks covering an IP range (JSON)
    RangeToCidr {
        /// First IP of the range
        start: String,
        /// Last IP of the range
        end: String,
    },
    /// Check whether an IP or CIDR is contained in a CIDR block
    CidrContains {
        /// Outer CIDR, e.g. 10.0.0.0/8
        outer: String,
        /// IP address or CIDR to test
        inner: String,
    },
    /// Convert an IP address between dotted, integer, hex and binary forms (JSON)
    Ip {
        /// IPv4/IPv6 address, or an integer (interpreted as IPv4)
        input: String,
    },
}

fn parse_net(raw: &str) -> Result<IpNet> {
    IpNet::from_str(raw.trim())
        .or_else(|_| IpAddr::from_str(raw.trim()).map(IpNet::from))
        .with_context(|| format!("'{raw}' is not a valid IP address or CIDR"))
}

pub fn run(cmd: NetworkCmd) -> Result<()> {
    match cmd {
        NetworkCmd::Subnet { cidr } => {
            let net = parse_net(&cidr)?;
            match net {
                IpNet::V4(net) => {
                    let hosts = 2u64.pow(32 - net.prefix_len() as u32);
                    let usable = if net.prefix_len() >= 31 {
                        hosts
                    } else {
                        hosts - 2
                    };
                    print_json(&serde_json::json!({
                        "cidr": net.to_string(),
                        "network": net.network().to_string(),
                        "broadcast": net.broadcast().to_string(),
                        "netmask": net.netmask().to_string(),
                        "wildcard": net.hostmask().to_string(),
                        "prefix": net.prefix_len(),
                        "addresses": hosts,
                        "usable_hosts": usable,
                        "first_usable": if net.prefix_len() >= 31 { net.network().to_string() } else { Ipv4Addr::from(u32::from(net.network()) + 1).to_string() },
                        "last_usable": if net.prefix_len() >= 31 { net.broadcast().to_string() } else { Ipv4Addr::from(u32::from(net.broadcast()) - 1).to_string() },
                    }))?;
                }
                IpNet::V6(net) => {
                    let bits = 128 - net.prefix_len() as u32;
                    let addresses = if bits >= 128 {
                        "340282366920938463463374607431768211456".to_string()
                    } else {
                        (1u128 << bits).to_string()
                    };
                    print_json(&serde_json::json!({
                        "cidr": net.to_string(),
                        "network": net.network().to_string(),
                        "last": net.broadcast().to_string(),
                        "prefix": net.prefix_len(),
                        "addresses": addresses,
                    }))?;
                }
            }
        }
        NetworkCmd::CidrToRange { cidr } => {
            let net = parse_net(&cidr)?;
            let count = match net {
                IpNet::V4(n) => 2u128.pow(32 - n.prefix_len() as u32).to_string(),
                IpNet::V6(n) => {
                    let bits = 128 - n.prefix_len() as u32;
                    if bits >= 128 {
                        "340282366920938463463374607431768211456".to_string()
                    } else {
                        (1u128 << bits).to_string()
                    }
                }
            };
            print_json(&serde_json::json!({
                "cidr": net.to_string(),
                "first": net.network().to_string(),
                "last": net.broadcast().to_string(),
                "addresses": count,
            }))?;
        }
        NetworkCmd::RangeToCidr { start, end } => {
            let cidrs: Vec<String> =
                match (IpAddr::from_str(start.trim()), IpAddr::from_str(end.trim())) {
                    (Ok(IpAddr::V4(a)), Ok(IpAddr::V4(b))) => {
                        if a > b {
                            bail!("start address is after end address");
                        }
                        ipnet::Ipv4Subnets::new(a, b, 0)
                            .map(|n: Ipv4Net| n.to_string())
                            .collect()
                    }
                    (Ok(IpAddr::V6(a)), Ok(IpAddr::V6(b))) => {
                        if a > b {
                            bail!("start address is after end address");
                        }
                        ipnet::Ipv6Subnets::new(a, b, 0)
                            .map(|n: Ipv6Net| n.to_string())
                            .collect()
                    }
                    (Ok(_), Ok(_)) => bail!("start and end must be the same IP version"),
                    _ => bail!("invalid IP address in range"),
                };
            print_json(&serde_json::json!({ "cidrs": cidrs }))?;
        }
        NetworkCmd::CidrContains { outer, inner } => {
            let outer_net = parse_net(&outer)?;
            let inner_net = parse_net(&inner)?;
            let contains = outer_net.contains(&inner_net);
            print_json(&serde_json::json!({
                "outer": outer_net.to_string(),
                "inner": inner_net.to_string(),
                "contains": contains,
            }))?;
            if !contains {
                std::process::exit(2);
            }
        }
        NetworkCmd::Ip { input } => {
            let raw = input.trim();
            if let Ok(v4) = Ipv4Addr::from_str(raw) {
                let n = u32::from(v4);
                print_json(&serde_json::json!({
                    "dotted": v4.to_string(),
                    "decimal": n,
                    "hex": format!("0x{n:08x}"),
                    "binary": format!("{n:032b}"),
                    "ipv6_mapped": v4.to_ipv6_mapped().to_string(),
                }))?;
            } else if let Ok(v6) = Ipv6Addr::from_str(raw) {
                let n = u128::from(v6);
                let expanded = v6
                    .segments()
                    .iter()
                    .map(|s| format!("{s:04x}"))
                    .collect::<Vec<_>>()
                    .join(":");
                print_json(&serde_json::json!({
                    "compressed": v6.to_string(),
                    "expanded": expanded,
                    "decimal": n.to_string(),
                    "hex": format!("0x{n:032x}"),
                }))?;
            } else if let Ok(n) = raw.parse::<u32>() {
                let v4 = Ipv4Addr::from(n);
                print_json(&serde_json::json!({
                    "decimal": n,
                    "dotted": v4.to_string(),
                    "hex": format!("0x{n:08x}"),
                    "binary": format!("{n:032b}"),
                }))?;
            } else {
                bail!("'{raw}' is not an IP address or a 32-bit integer");
            }
        }
    }
    Ok(())
}
