use log::info;
use std::time::Duration;

use config::kafka::Kafka as KafkaCfg;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};

#[allow(clippy::get_first)]
pub async fn produce(kfk_cfg: &KafkaCfg) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", kfk_cfg.brokers_to_str())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");
    let topic_name = kfk_cfg.topics.get(0).unwrap();
    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    // TODO build payload with struct
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
