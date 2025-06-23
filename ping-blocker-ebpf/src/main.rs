#![no_std]
#![no_main]

use aya_bpf::{
    bindings::xdp_action,
    macros::{xdp, map},
    maps::{HashMap, Array},
    programs::XdpContext,
};
use aya_log_ebpf::info;
use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    icmp::IcmpHdr,
    tcp::TcpHdr,
};

// Yasaklı IP'leri saklamak için HashMap
#[map(name = "BLOCKED_IPS")]
static mut BLOCKED_IPS: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

// SSH bağlantı sayacı için HashMap (IP -> sayac)
#[map(name = "SSH_COUNTERS")]
static mut SSH_COUNTERS: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

// Kullanıcı parametreleri için Array (index 0: SSH eşik değeri)
#[map(name = "CONFIG")]
static mut CONFIG: Array<u32> = Array::with_max_entries(16, 0);

const ETH_HDR_LEN: usize = mem::size_of::<EthHdr>();
const IP_HDR_LEN: usize = mem::size_of::<Ipv4Hdr>();
const ICMP_HDR_LEN: usize = mem::size_of::<IcmpHdr>();
const TCP_HDR_LEN: usize = mem::size_of::<TcpHdr>();

#[xdp(name = "ping_blocker")]
pub fn ping_blocker(ctx: XdpContext) -> u32 {
    match try_ping_blocker(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_ping_blocker(ctx: XdpContext) -> Result<u32, u32> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?;
    
    // Ethernet başlığını kontrol et
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, ETH_HDR_LEN)?;
    let source_ip = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    // Yasaklı IP kontrolü
    unsafe {
        if let Some(_) = BLOCKED_IPS.get(&source_ip) {
            info!(&ctx, "Blocked IP detected: {}", source_ip);
            return Ok(xdp_action::XDP_DROP);
        }
    }

    // Protokol tipine göre işlem yap
    match unsafe { (*ipv4hdr).proto } {
        IpProto::Icmp => {
            // ICMP paketlerini kontrol et (ping engelleme)
            return handle_icmp_packet(&ctx, source_ip);
        }
        IpProto::Tcp => {
            // TCP paketlerini kontrol et (SSH DDoS koruması)
            return handle_tcp_packet(&ctx, source_ip);
        }
        _ => return Ok(xdp_action::XDP_PASS),
    }
}

fn handle_icmp_packet(ctx: &XdpContext, source_ip: u32) -> Result<u32, u32> {
    let icmphdr: *const IcmpHdr = ptr_at(ctx, ETH_HDR_LEN + IP_HDR_LEN)?;
    
    // ICMP Echo Request (ping) kontrolü
    if unsafe { (*icmphdr).type_ } == 8 {
        info!(ctx, "Ping request blocked from IP: {}", source_ip);
        return Ok(xdp_action::XDP_DROP);
    }
    
    Ok(xdp_action::XDP_PASS)
}

fn handle_tcp_packet(ctx: &XdpContext, source_ip: u32) -> Result<u32, u32> {
    let tcphdr: *const TcpHdr = ptr_at(ctx, ETH_HDR_LEN + IP_HDR_LEN)?;
    let dest_port = u16::from_be(unsafe { (*tcphdr).dest });
    
    // SSH port (22) kontrolü
    if dest_port == 22 {
        // SYN flag kontrolü (yeni bağlantı girişimi)
        let tcp_flags = unsafe { (*tcphdr).flags() };
        if tcp_flags & 0x02 != 0 { // SYN flag set
            return handle_ssh_connection(ctx, source_ip);
        }
    }
    
    Ok(xdp_action::XDP_PASS)
}

fn handle_ssh_connection(ctx: &XdpContext, source_ip: u32) -> Result<u32, u32> {
    unsafe {
        // Mevcut sayacı al veya 0'dan başla
        let current_count = SSH_COUNTERS.get(&source_ip).copied().unwrap_or(0);
        let new_count = current_count + 1;
        
        // Sayacı güncelle
        SSH_COUNTERS.insert(&source_ip, &new_count, 0)
            .map_err(|_| 1u32)?;
        
        // Eşik değerini kontrol et
        if let Some(threshold) = CONFIG.get(0) {
            if new_count > *threshold {
                // IP'yi yasakla
                BLOCKED_IPS.insert(&source_ip, &1, 0)
                    .map_err(|_| 1u32)?;
                
                info!(ctx, "IP {} blocked due to SSH flood: {} connections", 
                     source_ip, new_count);
                
                return Ok(xdp_action::XDP_DROP);
            }
        }
        
        info!(ctx, "SSH connection from {}: {} attempts", source_ip, new_count);
    }
    
    Ok(xdp_action::XDP_PASS)
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, u32> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(1);
    }

    Ok((start + offset) as *const T)
}

aya_bpf::macros::export_license!("GPL");
