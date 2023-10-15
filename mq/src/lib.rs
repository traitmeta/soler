use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

#[allow(dead_code)]
fn consume_msg(consumer: &mut Consumer) {
    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                println!("{:?}", m);
            }
            let _ = consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();
    }
}

#[allow(dead_code)]
fn consume_one_msg(consumer: &mut Consumer) {
    for ms in consumer.poll().unwrap().iter() {
        for m in ms.messages() {
            println!("{:?}", m);
        }
        let _ = consumer.consume_messageset(ms);
    }
    consumer.commit_consumed().unwrap();
}

#[allow(dead_code)]
fn new_consumer() -> Consumer {
    Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic_partitions("nft-indexer-event".to_owned(), &[0])
        .with_fallback_offset(FetchOffset::Earliest)
        .with_group("test-index".to_owned())
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .create()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::{
        sync::{Arc, Mutex},
        thread::{sleep, spawn},
    };

    use crate::{consume_one_msg, new_consumer};

    #[test]
    #[ignore]
    fn test_consumer() {
        let consumer = new_consumer();
        let arc_consumer = Arc::new(Mutex::new(consumer));
        let consumer1 = arc_consumer.clone();
        let consume_msg_handler = spawn(move || {
            let mut consumer = consumer1.lock().unwrap();
            consume_one_msg(&mut consumer);
        });
        sleep(time::Duration::from_millis(10));
        let _ = consume_msg_handler.join();

        let consumer2 = arc_consumer.clone();
        let consumer = consumer2.lock().unwrap();
        println!("{}", consumer.group())
    }
}
