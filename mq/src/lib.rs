use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

fn consume_msg(consumer: &mut Consumer) {
    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                println!("{:?}", m);
            }
            consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();
    }
}

fn consume_one_msg(consumer: &mut Consumer) {
    for ms in consumer.poll().unwrap().iter() {
        for m in ms.messages() {
            println!("{:?}", m);
        }
        consumer.consume_messageset(ms);
    }
    consumer.commit_consumed().unwrap();
}

fn new_consumer() -> Consumer {
    let mut consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic_partitions("nft-indexer-event".to_owned(), &[0])
        .with_fallback_offset(FetchOffset::Earliest)
        .with_group("test-index".to_owned())
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .create()
        .unwrap();

    consumer
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::{
        sync::{Arc, Mutex},
        thread::{sleep, spawn},
    };

    use crate::{consume_msg, new_consumer, consume_one_msg};

    #[test]
    #[ignore]
    fn test_consumer() {
        let mut consumer = new_consumer();
        let arc_consumer = Arc::new(Mutex::new(consumer));
        let consumer1 = arc_consumer.clone();
        let consume_msg_handler = spawn(move || {
            let mut consumer = consumer1.lock().unwrap();
            consume_one_msg(&mut consumer);
        });
        sleep(time::Duration::from_millis(10));
        consume_msg_handler.join();
        
        let consumer2 = arc_consumer.clone();
        let mut consumer = consumer2.lock().unwrap();
        println!("{}",consumer.group())
    }
}
