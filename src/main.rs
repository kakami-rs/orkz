#![allow(dead_code)]

use std::env;
use clap::{Parser, Subcommand};
use configparser::ini::Ini;

mod cs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    options: Options,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
struct Options {
    #[arg(long, default_value=".")]
    log_path: String,
    #[arg(long)]
    tag: String,
}

#[derive(Subcommand)]
enum Commands {
    CS(cs::CSArgs),
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    init_fixed_window_roller_log(args.options.log_path.as_str());
    let path = match current_path("orkz.ini") {
        Some(path) => path,
        None => {
            log::error!("orkz.ini not found");
            return;
        }
    };
    let mut config = Ini::new();
    let _ = config.load(&path);
    let ts = match config.getuint("TagList", args.options.tag.as_str()) {
        Ok(Some(ts)) => ts as i64,
        _ => {
            let ts = chrono::Utc::now().timestamp_millis();
            config.set("TagList", args.options.tag.as_str(), Some(format!("{}",ts)));
            ts
        }
    };
    config.write(&path).unwrap();
    let offset = chrono::Utc::now().timestamp_millis() - ts;

    match args.command {
        Commands::CS(cs_args) => {
            cs::handle_cs(&cs_args, offset, &args.options.tag).await;
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

fn init_fixed_window_roller_log(log_dir: &str) {
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

    let path = if log_dir.is_empty() || log_dir == "." {
        get_log_path().unwrap()
    } else {
        log_dir.to_string()
    };

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

fn get_log_path() -> Option<String> {
    let mut rpath = env::current_exe().ok()?;
    rpath.push("../logs");
    let opath = rpath.to_str()?;
    Some(clean_path(opath))
}

fn current_path(mark: &str) -> Option<String> {
    let mut rpath = env::current_exe().ok()?;
    rpath.push("../");
    rpath.push(mark);
    let opath = rpath.to_str()?;
    Some(clean_path(opath))
}

fn clean_path(path: &str) -> String {
    let result = path.split('/').fold(Vec::new(), |mut stack, c| {
        if c == ".." {
            stack.pop();
        } else if !c.is_empty() && c != "." {
            stack.push(c);
        }
        stack
    }).join("/");

    if path.starts_with("/") {
        format!("/{}", result)
    } else {
        result
    }
}