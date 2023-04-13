use aya_bpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::HashMap,
    programs::XdpContext,
};
use network_types::{
    eth::EthHdr,
    ip::{Ipv4Hdr, Ipv6Hdr},
};

pub(super) type XDPAction = u32;

#[map(name = "IPV4_BLOCKLIST")]
static mut IPV4_BLOCKLIST: HashMap<u32, u8> = HashMap::<u32, u8>::with_max_entries(1024, 0);

#[map(name = "IPV6_BLOCKLIST")]
static mut IPV6_BLOCKLIST: HashMap<[u8; 16], u8> =
    HashMap::<[u8; 16], u8>::with_max_entries(1024, 0);

#[xdp(name = "network_monitor")]
pub fn network_monitor(ctx: XdpContext) -> XDPAction {
    match try_network_monitor(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

pub(super) fn try_network_monitor(ctx: XdpContext) -> Result<XDPAction, ()> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?;

    let action = match unsafe { (*ethhdr).ether_type } {
        network_types::eth::EtherType::Ipv4 => {
            let ipv4_header: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN)?;
            let src_addr = u32::from_be(unsafe { (*ipv4_header).src_addr });
            check_ipv4_blocklist(&src_addr)
        }
        network_types::eth::EtherType::Ipv6 => {
            let ipv6_header: *const Ipv6Hdr = ptr_at(&ctx, EthHdr::LEN)?;
            let src_addr: [u8; 16] = unsafe { (*ipv6_header).src_addr.in6_u.u6_addr8 };
            check_ipv6_blocklist(&src_addr)
        }
        _ => xdp_action::XDP_PASS,
    };

    Ok(action)
}

#[inline(always)]
fn check_ipv4_blocklist(addr: &u32) -> XDPAction {
    if unsafe { IPV4_BLOCKLIST.get(addr).is_some() } {
        xdp_action::XDP_DROP
    } else {
        xdp_action::XDP_PASS
    }
}

#[inline(always)]
fn check_ipv6_blocklist(addr: &[u8; 16]) -> XDPAction {
    if unsafe { IPV6_BLOCKLIST.get(addr).is_some() } {
        xdp_action::XDP_DROP
    } else {
        xdp_action::XDP_PASS
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = core::mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}
