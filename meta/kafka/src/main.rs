use clap::Parser;
use config::Args;
use config::{base::BaseConfig, Config};
use kafka::consumer::consume_and_print;
use kafka::producer::produce;
use kafka::utils::setup_logger;
use log::info;
use rdkafka::util::get_rdkafka_version;

#[tokio::main]
async fn main() {
    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let args = Args::parse();
    let config = BaseConfig::load(&args.config_path).unwrap();
    let kafka_cfg = config.kafka.unwrap();
    info!("Started Kafka endpoint at {:?}", &kafka_cfg);
    match &kafka_cfg.log_level {
        Some(log_level) => setup_logger(true, Some(log_level.as_str())),
        None => (),
    }
    produce(&kafka_cfg).await;
    consume_and_print(&kafka_cfg).await;
}
