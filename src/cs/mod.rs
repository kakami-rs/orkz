use clap::Args;

mod quic;
mod udp;

#[derive(Args, Debug)]
pub struct CSArgs {
    #[arg(long, default_value = "127.0.0.1:40100")]
    local_addr: String,
    #[arg(long, default_value = "127.0.0.1:40101")]
    remote_addr: String,
    #[arg(long, default_value = "false")]
    server: bool,
    #[arg(long, default_value = "quic")]
    protocol: String,
    #[arg(long, default_value = "bbr")]
    cc: String,
    #[arg(long, short = 'p', default_value = "2")]
    priority: usize,
}

pub async fn handle_cs(args: &CSArgs, offset: i64) {
    log::info!("{:?}", args);
    match args.protocol.as_str() {
        "quic" => {
            if args.server {
                quic::handle_quic_server(args).await;
            } else {
                quic::handle_quic_client(args, offset).await;
            }
        }
        "udp" => {
            if args.server {
                udp::handle_udp_server(args).await;
            } else {
                udp::handle_udp_client(args).await;
            }
        }
        _ => {
            log::error!("Unsupported protocol: {}", args.protocol);
        }
    }
}

pub fn to_speed(size: usize, elasped: u128) -> String {
    if elasped == 0 {
        return "0 MB/s".to_string();
    }
    let speed = size as f64 * 1000.0 / elasped as f64;
    if speed < 1024.0 {
        format!("{:.2} B/s", speed)
    } else if speed < 1024.0 * 1024.0 {
        format!("{:.2} KB/s", speed / 1024.0)
    } else {
        format!("{:.2} MB/s", speed / 1024.0 / 1024.0)
    }
}
