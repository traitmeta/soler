use clap::{Command, Arg};
use rdkafka::config::ClientConfig;
use std::io::Write;
use std::thread;

use chrono::prelude::*;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::{LevelFilter, Record};

use std::boxed::Box;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub fn get_config() -> Result<(String, ClientConfig), Box<dyn std::error::Error>> {
    let matches = Command::new("rust client example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .arg(
            Arg::new("config")
                .help("path to confluent cloud config file")
                .long("config")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("topic")
                .help("test topic to use")
                .long("topic")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut kafka_config = ClientConfig::new();

    let file = File::open(matches.value_of("config").ok_or("error parsing config")?)?;
    for line in BufReader::new(&file).lines() {
        let cur_line: String = line?.trim().to_string();
        if cur_line.starts_with('#') || cur_line.len() < 1 {
            continue;
        }
        let key_value: Vec<&str> = cur_line.split("=").collect();
        kafka_config.set(
            *key_value.get(0).ok_or("malformed key")?,
            *key_value.get(1).ok_or("malformed value")?,
        );
    }

    Ok((
        matches
            .value_of("topic")
            .ok_or("error parsing topic")?
            .to_string(),
        kafka_config,
    ))
}


pub fn setup_logger(log_thread: bool, rust_log: Option<&str>) {
    let output_format = move |formatter: &mut Formatter, record: &Record| {
        let thread_name = if log_thread {
            format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
        } else {
            "".to_string()
        };

        let local_time: DateTime<Local> = Local::now();
        let time_str = local_time.format("%H:%M:%S%.3f").to_string();
        write!(
            formatter,
            "{} {}{} - {} - {}\n",
            time_str,
            thread_name,
            record.level(),
            record.target(),
            record.args()
        )
    };

    let mut builder = Builder::new();
    builder
        .format(output_format)
        .filter(None, LevelFilter::Info);

    rust_log.map(|conf| builder.parse_filters(conf));

    builder.init();
}

#[allow(dead_code)]
fn main() {
    println!("This is not an example");
}