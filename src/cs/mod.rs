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
}

pub async fn handle_cs(args: &CSArgs) {
    match args.protocol.as_str() {
        "quic" => {
            if args.server {
                quic::handle_quic_server(args).await;
            } else {
                quic::handle_quic_client(args).await;
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

