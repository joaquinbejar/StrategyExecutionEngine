/*******************************************************************************
Copyright (c) 2024.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
******************************************************************************/

/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/5/24
******************************************************************************/

use crate::{KafkaClient, NatsClient, RabbitMQClient, RedisClient, ZeroMQClient};

/// Trait for a messaging client.
pub trait MessagingClient {
    fn produce(&self, topic: &str, message: &str) -> Result<(), String>;
    fn consume(&self, topic: &str) -> Result<String, String>;
}

pub enum ClientType {
    Kafka,
    Redis,
    Nats,
    RabbitMQ,
    ZeroMQ,
}

pub struct MessagingClientFactory;

impl MessagingClientFactory {
    pub fn create_client(client_type: ClientType) -> Box<dyn MessagingClient> {
        match client_type {
            ClientType::Kafka => Box::new(KafkaClient::new(
                "localhost:9092".to_string(),  // 默认 broker
                "default-group".to_string()    // 默认 group_id
            )),
            ClientType::Redis => Box::new(RedisClient::new()),
            ClientType::Nats => Box::new(NatsClient::new()),
            ClientType::RabbitMQ => Box::new(RabbitMQClient::new()),
            ClientType::ZeroMQ => Box::new(ZeroMQClient::new()),
        }
    }
}

pub struct MessagingService {
    client: Box<dyn MessagingClient>,
}

impl MessagingService {
    pub fn new(client_type: ClientType) -> Self {
        let client = MessagingClientFactory::create_client(client_type);
        MessagingService { client }
    }

    pub fn produce(&self, topic: &str, message: &str) -> Result<(), String> {
        self.client.produce(topic, message)
    }

    pub fn consume(&self, topic: &str) -> Result<String, String> {
        self.client.consume(topic)
    }
}

/*******************************************************************************

    let kafka_service = MessagingService::new(ClientType::Kafka);
    kafka_service.produce("test_topic", "Hello, Kafka!").unwrap();
    let message = kafka_service.consume("test_topic").unwrap();
    println!("Consumed message: {}", message);

******************************************************************************/
