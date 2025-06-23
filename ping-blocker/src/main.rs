use aya::{
    include_bytes_aligned,
    maps::{Array, HashMap},
    programs::{Xdp, XdpFlags},
    util::online_cpus,
    Bpf,
};
use aya_log::BpfLogger;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::{info, warn, debug};
use std::{
    convert::TryFrom,
    net::Ipv4Addr,
    str::FromStr,
    thread,
    time::Duration,
};
use tokio::signal;

#[derive(Debug, Parser)]
#[clap(
    name = "ping-blocker",
    about = "eBPF-based ping blocker and SSH DDoS protection"
)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
    #[clap(short, long, default_value = "10")]
    threshold: u32,
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    // Logging seviyesini ayarla
    env_logger::Builder::from_default_env()
        .filter_level(if opt.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    info!("Starting ping-blocker on interface: {}", opt.iface);
    info!("SSH flood threshold: {} connections", opt.threshold);

    // eBPF programını yükle
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/ping-blocker-ebpf"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/ping-blocker-ebpf"
    ))?;

    // Logging'i etkinleştir
    if let Err(e) = BpfLogger::init(&mut bpf) {
        warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Konfigürasyonu ayarla
    let mut config: Array<_, u32> = Array::try_from(bpf.map_mut("CONFIG")?)?;
    config.set(0, opt.threshold, 0)?;
    info!("SSH threshold configured: {}", opt.threshold);

    // XDP programını al ve ağ arayüzüne bağla
    let program: &mut Xdp = bpf.program_mut("ping_blocker").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("Failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    info!("XDP program attached to interface: {}", opt.iface);

    // Haritaların referanslarını al
    let blocked_ips: HashMap<_, u32, u32> = HashMap::try_from(bpf.map("BLOCKED_IPS")?)?;
    let ssh_counters: HashMap<_, u32, u32> = HashMap::try_from(bpf.map("SSH_COUNTERS")?)?;

    info!("ping-blocker started successfully!");
    info!("Press Ctrl+C to stop...");

    // İstatistik gösterme görevi
    let stats_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            print_statistics(&blocked_ips, &ssh_counters).await;
        }
    });

    // Sinyal bekleme
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C, shutting down...");
        }
        Err(err) => {
            warn!("Unable to listen for shutdown signal: {}", err);
        }
    }

    stats_task.abort();
    info!("ping-blocker stopped");
    Ok(())
}

async fn print_statistics(
    blocked_ips: &HashMap<&aya::maps::MapData, u32, u32>,
    ssh_counters: &HashMap<&aya::maps::MapData, u32, u32>,
) {
    let mut blocked_count = 0;
    let mut total_ssh_attempts = 0;

    // Yasaklı IP sayısını hesapla
    for item in blocked_ips.iter() {
        if let Ok(_) = item {
            blocked_count += 1;
        }
    }

    // SSH girişim sayısını hesapla
    for item in ssh_counters.iter() {
        if let Ok((_, count)) = item {
            total_ssh_attempts += count;
        }
    }

    info!(
        "Statistics - Blocked IPs: {}, Total SSH attempts: {}",
        blocked_count, total_ssh_attempts
    );

    if blocked_count > 0 {
        info!("Blocked IP addresses:");
        for item in blocked_ips.iter() {
            if let Ok((ip, _)) = item {
                let ip_addr = Ipv4Addr::from(ip.to_be());
                info!("  - {}", ip_addr);
            }
        }
    }
}
