use std::net::SocketAddr;
use super::CSArgs;

// use protos::prom_protos::message as pm;
use tokio::net::UdpSocket;
use tokio::time::Instant;

pub async fn handle_udp_server(args: &CSArgs) {
    let local_addr: SocketAddr = args.local_addr.parse().unwrap();
    // let remote_addr: SocketAddr = args.remote_addr.parse().unwrap();
    let socket = UdpSocket::bind(local_addr).await.unwrap();
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(3));
    let mut buf = vec![0u8; 1400];
    let mut size = 0;
    let mut tn = Instant::now();
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let elasped = tn.elapsed().as_millis();
                log::info!("speed: {} MBps", size / 1024 / elasped as usize);
                size = 0;
                tn = Instant::now();
            }
            result = socket.recv_from(&mut buf) => {
                if let Ok((pkt, _)) = result {
                    size += pkt;
                }
            }
        }
    }
}

pub async fn handle_udp_client(args: &CSArgs) {
    let local_addr: SocketAddr = args.local_addr.parse().unwrap();
    let remote_addr: SocketAddr = args.remote_addr.parse().unwrap();
    let socket = UdpSocket::bind(local_addr).await.unwrap();
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let buffer = vec!['n' as u8; 1300];
    let mut count = 0;
    let mut size = 0;
    let mut tn = Instant::now();
    loop {
        tokio::select! {
            biased;
            _ = ticker.tick() => {
                count += 1;
                if count % 3 == 0 {
                    let elasped = tn.elapsed().as_millis();
                    log::info!("speed: {} MBps", size / 1024 / elasped as usize);
                    size = 0;
                    tn = Instant::now();
                }
            }
            ret = socket.send_to(&buffer, remote_addr) => {
                if let Ok(l) = ret {
                    size += l;
                }
            }
        }
    }
}

