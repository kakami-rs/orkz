use std::net::SocketAddr;
use std::sync::Arc;
use bifrost::QUdpSocket;
use bifrost::quicc::endpoint::Endpoint;
use bifrost::util::Conn;
use bytes::Bytes;
use super::CSArgs;

use protos::prom_protos::message as pm;
use protos::prom_codegen::message as pc;
use tokio::net::UdpSocket;
use tokio::time::Instant;
use protobuf::Message as _;

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
                        role: "server".to_string(),
                        tag: "server".to_string(),
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
                    if _pkt.len() < 1024 {
                        let msg = bytes::Bytes::from(_pkt);
                        let _ = stream.send_bytes(msg).await;
                    }
                }
            }
        }
    }
}

pub async fn handle_quic_client(args: &CSArgs, offset: i64, tag: &str) {
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
    let mut ticker1 = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let buffer = vec!['n' as u8; 8192];
    let msg = Bytes::from(buffer);
    let mut size = 0;
    let mut send_bytes: u64 = 0;
    let mut tn = Instant::now();
    let mut rtt = 0i64;
    let mut count = 0;
    loop {
        if count > 100 {
            break;
        }
        tokio::select! {
            biased;
            _ = ticker.tick() => {
                if let Some(stats) = endpoint.stats().await {
                    let elasped = tn.elapsed().as_millis();
                    if elasped == 0 {
                        continue;
                    }
                    count += 1;
                    let diff = stats.sent_bytes - send_bytes;
                    send_bytes = stats.sent_bytes;
                    let line = pm::Line {
                        local_addr: args.local_addr.clone(),
                        remote_addr: args.remote_addr.clone(),
                        role: "client".to_string(),
                        tag: tag.to_string(),
                        proto: args.protocol.clone(),
                        cc: args.cc.clone(),
                        priority: args.priority as i32,
                        rtt,
                        output_bw: (diff*1000/elasped as u64) as i64,
                        output_rate: diff as f32 / size as f32,
                        output_loss: 0.0,
                        input_bw: 0,
                        input_rate: 0.0,
                        input_loss: 0.0,
                        timestamp: chrono::Utc::now().timestamp_millis() - offset,
                    };
                    let line = serde_json::to_string(&line).unwrap();
                    if count > 3 {
                        log::info!("{line}");
                    }
                }
                size = 0;
                tn = Instant::now();
            }
            _ = ticker1.tick() => {
                let mut msg = pc::Message::new();
                let mut pp = pc::PingPong::new();
                pp.timestamp = chrono::Utc::now().timestamp_millis();
                msg.set_ping_pong(pp);
                let mm = bytes::Bytes::from(msg.write_to_bytes().unwrap());
                let _ = stream.send_bytes(mm).await;
            }
            result = stream.next() => {
                if let Ok(ref bytes) = result {
                    if bytes.len() < 1024 {
                        if let Ok(msg_in) = pc::Message::parse_from_bytes(bytes) {
                            match msg_in.union {
                                Some(pc::message::Union::PingPong(pp)) => {
                                    let ts = chrono::Utc::now().timestamp_millis();
                                    rtt = ts - pp.timestamp;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            ret = stream.send_bytes(msg.clone()) => {
                if let Ok(_) = ret {
                    size += msg.len();
                }
            }
        }
    }
}

