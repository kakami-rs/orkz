use std::net::SocketAddr;
use std::sync::Arc;
use bifrost::QUdpSocket;
use bifrost::quicc::endpoint::Endpoint;
use bifrost::util::Conn;
use bytes::Bytes;
use super::CSArgs;

use protos::prom_protos::message as pm;
use tokio::net::UdpSocket;
use tokio::time::Instant;

pub async fn handle_quic_server(args: &CSArgs) {
    let local_addr: SocketAddr = args.local_addr.parse().unwrap();
    let remote_addr: SocketAddr = args.remote_addr.parse().unwrap();
    let socket = UdpSocket::bind(local_addr).await.unwrap();
    let cc = match args.cc.as_str() {
        "reno" => 0,
        "cubic" => 1,
        "bbr" => 2,
        "bbr2" => 3,
        _ => 1,
    };

    let qsocket = QUdpSocket::new(socket, local_addr, remote_addr);
    let conn: Arc<dyn Conn + Send + Sync> = Arc::new(qsocket);
    let endpoint = Endpoint::server(conn, args.priority, cc).await.unwrap();
    let stream = endpoint.accept().await.unwrap();
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(3));
    let mut size = 0;
    let mut recv_bytes: u64 = 0;
    let mut tn = Instant::now();
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Some(stats) = endpoint.stats().await {
                    let elasped = tn.elapsed().as_millis();
                    if elasped == 0 {
                        continue;
                    }
                    let diff = stats.recv_bytes - recv_bytes;
                    recv_bytes = stats.recv_bytes;
                    let line = pm::Line {
                        local_addr: args.local_addr.clone(),
                        remote_addr: args.remote_addr.clone(),
                        role: pm::line::Role::Server as i32,
                        proto: args.protocol.clone(),
                        cc: args.cc.clone(),
                        priority: args.priority as i32,
                        rtt: 0,
                        input_bw: (diff*1000/elasped as u64) as i64,
                        input_rate: diff as f32 / size as f32,
                        input_loss: 0.0,
                        output_bw: 0,
                        output_rate: 0.0,
                        output_loss: 0.0,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    };
                    let line = serde_json::to_string(&line).unwrap();
                    log::info!("{line}");
                    size = 0;
                    tn = Instant::now();
                }
            }
            result = stream.next() => {
                if let Ok(_pkt) = result {
                    size += _pkt.len();
                }
            }
        }
    }
}

pub async fn handle_quic_client(args: &CSArgs) {
    let local_addr: SocketAddr = args.local_addr.parse().unwrap();
    let remote_addr: SocketAddr = args.remote_addr.parse().unwrap();
    let socket = UdpSocket::bind(local_addr).await.unwrap();
    let cc = match args.cc.as_str() {
        "reno" => 0,
        "cubic" => 1,
        "bbr" => 2,
        "bbr2" => 3,
        _ => 1,
    };

    let qsocket = QUdpSocket::new(socket, local_addr, remote_addr);
    let conn: Arc<dyn Conn + Send + Sync> = Arc::new(qsocket);
    let endpoint = Endpoint::client(conn, args.priority, cc).await.unwrap();
    let stream = endpoint.open().await.unwrap();

    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(3));
    let buffer = vec!['n' as u8; 8192];
    let msg = Bytes::from(buffer);
    let mut size = 0;
    let mut send_bytes: u64 = 0;
    let mut tn = Instant::now();
    loop {
        tokio::select! {
            biased;
            _ = ticker.tick() => {
                if let Some(stats) = endpoint.stats().await {
                    let elasped = tn.elapsed().as_millis();
                    if elasped == 0 {
                        continue;
                    }
                    let diff = stats.sent_bytes - send_bytes;
                    send_bytes = stats.sent_bytes;
                    let line = pm::Line {
                        local_addr: args.local_addr.clone(),
                        remote_addr: args.remote_addr.clone(),
                        role: pm::line::Role::Server as i32,
                        proto: args.protocol.clone(),
                        cc: args.cc.clone(),
                        priority: args.priority as i32,
                        rtt: 0,
                        output_bw: (diff*1000/elasped as u64) as i64,
                        output_rate: diff as f32 / size as f32,
                        output_loss: 0.0,
                        input_bw: 0,
                        input_rate: 0.0,
                        input_loss: 0.0,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    };
                    let line = serde_json::to_string(&line).unwrap();
                    log::info!("{line}");
                }
                size = 0;
                tn = Instant::now();
            }
            ret = stream.send_bytes(msg.clone()) => {
                if let Ok(_) = ret {
                    size += msg.len();
                }
            }
        }
    }
}

