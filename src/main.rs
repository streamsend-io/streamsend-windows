use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Kafka SASL/SSL example");
    
    // Fix: rdkafka version returns a tuple (major, version_string)
    let version = rdkafka::util::get_rdkafka_version();
    println!("rdkafka version: {}", version.1);
    
    // Configuration for Confluent Cloud
    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    let security_protocol = std::env::var("KAFKA_SECURITY_PROTOCOL")
        .unwrap_or_else(|_| "PLAINTEXT".to_string());
    let sasl_username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let sasl_password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    
    println!("Connecting to: {}", bootstrap_servers);
    println!("Security protocol: {}", security_protocol);
    
    // Create Kafka producer
    let producer = create_producer(&bootstrap_servers, &security_protocol, sasl_username, sasl_password)?;
    
    println!("✅ Producer created successfully!");
    
    // Send a test message
    send_test_message(&producer).await?;
    
    println!("🎉 Example completed successfully!");
    Ok(())
}

fn create_producer(
    bootstrap_servers: &str,
    security_protocol: &str,
    sasl_username: Option<String>,
    sasl_password: Option<String>,
) -> Result<FutureProducer, Box<dyn std::error::Error>> {
    let mut config = ClientConfig::new();
    
    config
        .set("bootstrap.servers", bootstrap_servers)
        .set("security.protocol", security_protocol)
        .set("message.timeout.ms", "5000");
    
    // Configure SASL if credentials are provided
    if let (Some(username), Some(password)) = (sasl_username, sasl_password) {
        config
            .set("sasl.mechanisms", "PLAIN")
            .set("sasl.username", username)
            .set("sasl.password", password);
    }
    
    // Additional settings for SSL/Confluent Cloud
    if security_protocol.contains("SSL") {
        config
            .set("ssl.endpoint.identification.algorithm", "https")
            .set("session.timeout.ms", "45000");
    }
    
    // Fix: Create producer with explicit tokio runtime
    let producer: FutureProducer = config.create()?;
    Ok(producer)
}

async fn send_test_message(producer: &FutureProducer) -> Result<(), Box<dyn std::error::Error>> {
    let topic = "test-topic";
    let key = "test-key";
    let payload = "Hello from Rust rdkafka!";
    
    println!("📤 Sending test message to topic: {}", topic);
    
    match producer
        .send(
            FutureRecord::to(topic)
                .key(key)
                .payload(payload),
            Duration::from_secs(5),
        )
        .await
    {
        Ok((partition, offset)) => {
            println!("✅ Message sent successfully!");
            println!("   Partition: {}, Offset: {}", partition, offset);
            Ok(())
        }
        Err((kafka_error, _)) => {
            eprintln!("❌ Failed to send message: {:?}", kafka_error);
            Err(Box::new(kafka_error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer_creation() {
        let result = create_producer(
            "localhost:9092",
            "PLAINTEXT", 
            None, 
            None
        );
        assert!(result.is_ok(), "Should create producer successfully");
    }
    
    #[test]
    fn test_producer_creation_with_sasl() {
        let result = create_producer(
            "test-cluster.confluent.cloud:9092",
            "SASL_SSL",
            Some("test-key".to_string()),
            Some("test-secret".to_string())
        );
        assert!(result.is_ok(), "Should create SASL producer successfully");
    }
}
