use std::time::Duration;

use lazy_static::lazy_static;
use rdkafka::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

lazy_static! {
    static ref KAFKA_BROKER: String = std::env::var("KAFKA_BROKER").expect("Can't read Kafka broker address");
    static ref KAFKA_TOPIC: String = std::env::var("KAFKA_TOPIC").expect("Can't read Kafka topic name");
}

pub(crate) fn create_producer() -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", &KAFKA_BROKER)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation failed")
}

// TODO: make a message read by multiple consumers in the same group
pub(crate) fn create_consumer() -> StreamConsumer {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "graphql-group")
        .set("bootstrap.servers", &KAFKA_BROKER)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&[&KAFKA_TOPIC])
        .expect("Can't subscribe to specified topics");

    consumer
}

// TODO: send without caller blocking
pub(crate) async fn send_message(producer: &FutureProducer, message: String) {
    let send_to_kafka_result = producer.send(
        FutureRecord::to(&KAFKA_TOPIC)
            .payload(&message)
            .key("and this is a key"),
        Timeout::After(Duration::from_secs(0)),
    ).await;

    match send_to_kafka_result {
        Ok(_) => println!("Message was sent"),
        Err(res) => println!("Message wasn't sent: {}", res.0)
    }
}