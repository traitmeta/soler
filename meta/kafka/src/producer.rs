use std::time::Duration;

use clap::{Arg, ArgAction, Command};
use log::info;

use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;

use kafka::utils::setup_logger;

async fn produce(brokers: &str, topic_name: &str) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .payload(&format!("Message {}", i))
                        .key(&format!("Key {}", i))
                        .headers(OwnedHeaders::new().insert(Header {
                            key: "header_key",
                            value: Some("header_value"),
                        })),
                    Duration::from_secs(0),
                )
                .await;

            // This will be executed when the result is received.
            info!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        info!("Future completed. Result: {:?}", future.await);
    }
}

#[tokio::main]
async fn main() {
    let matches = Command::new("producer example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple command line producer")
        .arg(
            Arg::new("brokers")
                .short('b')
                .long("brokers")
                .help("Broker list in kafka format")
                .action(ArgAction::SetTrue)
                .default_value("localhost:9092"),
        )
        .arg(
            Arg::new("log-conf")
                .long("log-conf")
                .help("Configure the logging format (example: 'rdkafka=trace')")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("topic")
                .short('t')
                .long("topic")
                .help("Destination topic")
                .action(ArgAction::SetTrue)
                .required(true),
        )
        .get_matches();

    setup_logger(
        true,
        Some(
            matches
                .get_one::<String>("log-conf")
                .expect("read brokers fail from config")
                .as_str(),
        ),
    );

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let brokers = matches
        .get_one::<String>("brokers")
        .expect("read brokers fail from config")
        .as_str();
    let topic = matches
        .get_one::<String>("topic")
        .expect("read topic fail from config")
        .as_str();
    produce(brokers, topic).await;
}
