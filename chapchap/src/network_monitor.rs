use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Context;
use aya::{
    maps::HashMap,
    programs::{Xdp, XdpFlags},
    Bpf,
};

pub(super) fn setup(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
    let network_monitor: &mut Xdp = bpf.program_mut("network_monitor").unwrap().try_into()?;
    network_monitor.load()?;
    network_monitor.attach("wlp4s0", XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut ipv4_blocklist: HashMap<_, u32, u8> =
        HashMap::try_from(bpf.take_map("IPV4_BLOCKLIST").unwrap())?;
    let mut ipv6_blocklist: HashMap<_, [u8; 16], u8> =
        HashMap::try_from(bpf.take_map("IPV6_BLOCKLIST").unwrap())?;

    let ipv4_block_addr: u32 = Ipv4Addr::new(95, 216, 88, 233).try_into()?;
    let ipv6_block_addr: [u8; 16] = Ipv6Addr::LOCALHOST.octets();

    ipv4_blocklist.insert(ipv4_block_addr, 0, 0)?;
    ipv6_blocklist.insert(ipv6_block_addr, 0, 0)?;

    Ok(())
}
