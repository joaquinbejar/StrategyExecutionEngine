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
   Date: 24/5/24
******************************************************************************/

#[cfg(test)]
mod config_tests {
    use lazy_static::lazy_static;
    use std::env;
    use std::sync::Mutex;
    use strategy_execution_engine::config::{
        Config, JsonSerializable, KafkaConfig, NatsConfig, RabbitMqConfig, RedisConfig,
        ZeroMqConfig,
    };

    lazy_static! {
        static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_kafka_config_serialization() {
        let kafka_config = KafkaConfig {
            kafka_url: "localhost:9092".to_string(),
        };
        let json = kafka_config.print_as_json().unwrap();
        assert_eq!(json, "{\n  \"kafka_url\": \"localhost:9092\"\n}");
    }

    #[test]
    fn test_nats_config_serialization() {
        let nats_config = NatsConfig {
            nats_url: "nats://localhost:4222".to_string(),
        };
        let json = nats_config.print_as_json().unwrap();
        assert_eq!(json, "{\n  \"nats_url\": \"nats://localhost:4222\"\n}");
    }

    #[test]
    fn test_rabbitmq_config_serialization() {
        let rabbitmq_config = RabbitMqConfig {
            rabbitmq_url: "amqp://localhost:5672".to_string(),
        };
        let json = rabbitmq_config.print_as_json().unwrap();
        assert_eq!(json, "{\n  \"rabbitmq_url\": \"amqp://localhost:5672\"\n}");
    }

    #[test]
    fn test_zeromq_config_serialization() {
        let zeromq_config = ZeroMqConfig {
            zmq_url: "tcp://localhost:5555".to_string(),
        };
        let json = zeromq_config.print_as_json().unwrap();
        assert_eq!(json, "{\n  \"zmq_url\": \"tcp://localhost:5555\"\n}");
    }

    #[test]
    fn test_redis_config_serialization() {
        let redis_config = RedisConfig {
            redis_url: "redis://localhost:6379".to_string(),
        };
        let json = redis_config.print_as_json().unwrap();
        assert_eq!(json, "{\n  \"redis_url\": \"redis://localhost:6379\"\n}");
    }

    #[test]
    fn test_config_initialization_and_serialization() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // Setting environment variables for the test
        env::set_var("KAFKA_URL", "localhost:9092");
        env::set_var("NATS_URL", "nats://localhost:4222");
        env::set_var("RABBITMQ_URL", "amqp://localhost:5672");
        env::set_var("ZMQ_URL", "tcp://localhost:5555");
        env::set_var("REDIS_URL", "redis://localhost:6379");

        let config = Config::new().unwrap();
        let json = config.print_as_json().unwrap();

        let expected_json = r#"
{
  "kafka": "{\n  \"kafka_url\": \"localhost:9092\"\n}",
  "nats": "{\n  \"nats_url\": \"nats://localhost:4222\"\n}",
  "rabbitmq": "{\n  \"rabbitmq_url\": \"amqp://localhost:5672\"\n}",
  "redis": "{\n  \"redis_url\": \"redis://localhost:6379\"\n}",
  "zeromq": "{\n  \"zmq_url\": \"tcp://localhost:5555\"\n}"
}"#;

        assert_eq!(json, expected_json.trim());
    }

    #[test]
    fn test_missing_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // Unset all environment variables
        env::remove_var("KAFKA_URL");
        env::remove_var("NATS_URL");
        env::remove_var("RABBITMQ_URL");
        env::remove_var("ZMQ_URL");
        env::remove_var("REDIS_URL");

        let config = Config::new().unwrap();
        let json = config.print_as_json().unwrap();

        let expected_json = r#"
{
  "kafka": null,
  "nats": null,
  "rabbitmq": null,
  "redis": null,
  "zeromq": null
}"#;

        assert_eq!(json, expected_json.trim());
    }
}
