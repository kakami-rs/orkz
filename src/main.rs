#![allow(dead_code)]

use clap::{Parser, Subcommand};

mod cs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CS(cs::CSArgs),
}

#[tokio::main]
async fn main() {
    init_fixed_window_roller_log();

    let args = Cli::parse();
    match args.command {
        Commands::CS(args) => {
            cs::handle_cs(&args).await;
        }
    }
}

fn init_stdout_log() {
    use log4rs::{
        config::{Appender, Root, Config},
        encode::pattern::PatternEncoder,
    };
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::Logger;
    use log::LevelFilter;

    let pattern = "{d(%Y-%m-%d %H:%M:%S)}\t{l}\t{M}:{L}\t{m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern))).build();

    let root = Root::builder()
        .appender("stdout")
        .build(LevelFilter::Debug);

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("root", LevelFilter::Debug))
        .build(root)
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

fn init_fixed_window_roller_log() {
    use log4rs::{
        config::{Appender, Root, Config},
        encode::pattern::PatternEncoder,
    };
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::Logger;
    use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
    use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
    use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
    use log4rs::append::rolling_file::RollingFileAppender;
    use log::LevelFilter;

    let path = "/Users/iuz/Downloads/temp/orkz";
    let pattern = "{m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern))).build();

    let log_path = format!("{path}/orkz.log");
    let roll_path = format!("{}.{{}}", log_path);

    let roller = FixedWindowRoller::builder()
        .build(&roll_path, 5).unwrap();
    let trigger = SizeTrigger::new(1024 * 1024 * 100);
    let policy = CompoundPolicy::new(Box::new(trigger),
        Box::new(roller));
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(log_path, Box::new(policy)).unwrap();

    let root = Root::builder()
        .appender("stdout")
        .appender("file")
        .build(LevelFilter::Debug);

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder().build("root", LevelFilter::Debug))
        .build(root)
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}