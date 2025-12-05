use anyhow::{Context, Result};
use aws_sdk_sns::Client as SnsClient;
use aws_sdk_sqs::Client as SqsClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u32,
    content: String,
    timestamp: String,
}

/// Producer that sends messages to SNS topic
struct SnsProducer {
    client: SnsClient,
    topic_arn: String,
}

impl SnsProducer {
    fn new(client: SnsClient, topic_arn: String) -> Self {
        Self { client, topic_arn }
    }

    async fn publish_message(&self, message: &Message) -> Result<()> {
        let message_body = serde_json::to_string(message)?;
        
        let response = self
            .client
            .publish()
            .topic_arn(&self.topic_arn)
            .message(&message_body)
            .subject("Test Message")
            .send()
            .await
            .context("Failed to publish message to SNS")?;

        println!(
            "✅ Published message {} to SNS. MessageId: {:?}",
            message.id,
            response.message_id()
        );

        Ok(())
    }
}

/// Consumer that receives messages from SQS queue
struct SqsConsumer {
    client: SqsClient,
    queue_url: String,
}

impl SqsConsumer {
    fn new(client: SqsClient, queue_url: String) -> Self {
        Self { client, queue_url }
    }

    async fn receive_messages(&self, max_messages: i32) -> Result<()> {
        println!("🔍 Polling SQS queue for messages...");

        let response = self
            .client
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(max_messages)
            .wait_time_seconds(10) // Long polling
            .send()
            .await
            .context("Failed to receive messages from SQS")?;

        if let Some(messages) = response.messages {
            if messages.is_empty() {
                println!("📭 No messages received");
                return Ok(());
            }

            for msg in messages {
                if let Some(body) = msg.body() {
                    // SNS wraps the message in a JSON envelope
                    if let Ok(sns_envelope) = serde_json::from_str::<serde_json::Value>(body) {
                        if let Some(message_str) = sns_envelope.get("Message").and_then(|m| m.as_str()) {
                            match serde_json::from_str::<Message>(message_str) {
                                Ok(message) => {
                                    println!("📨 Received message: {:?}", message);
                                }
                                Err(_) => {
                                    println!("📨 Received raw message: {}", message_str);
                                }
                            }
                        }
                    } else {
                        println!("📨 Received non-SNS message: {}", body);
                    }
                }

                // Delete the message after processing
                if let Some(receipt_handle) = msg.receipt_handle() {
                    self.client
                        .delete_message()
                        .queue_url(&self.queue_url)
                        .receipt_handle(receipt_handle)
                        .send()
                        .await
                        .context("Failed to delete message")?;
                    println!("🗑️  Message deleted from queue");
                }
            }
        } else {
            println!("📭 No messages received");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 SNS/SQS Example Application\n");

    // Load AWS configuration
    let config = aws_config::load_from_env().await;
    let sns_client = SnsClient::new(&config);
    let sqs_client = SqsClient::new(&config);

    // Get environment variables for SNS topic ARN and SQS queue URL
    let topic_arn = std::env::var("SNS_TOPIC_ARN")
        .context("SNS_TOPIC_ARN environment variable not set")?;
    let queue_url = std::env::var("SQS_QUEUE_URL")
        .context("SQS_QUEUE_URL environment variable not set")?;

    println!("📌 SNS Topic ARN: {}", topic_arn);
    println!("📌 SQS Queue URL: {}\n", queue_url);

    let producer = SnsProducer::new(sns_client, topic_arn);
    let consumer = SqsConsumer::new(sqs_client, queue_url);

    // Publish some test messages
    println!("--- Publishing Messages ---");
    for i in 1..=3 {
        let message = Message {
            id: i,
            content: format!("Test message number {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        producer.publish_message(&message).await?;
        sleep(Duration::from_millis(500)).await;
    }

    println!("\n--- Consuming Messages ---");
    // Wait a bit for messages to propagate
    sleep(Duration::from_secs(2)).await;

    // Consume messages (polling 3 times)
    for _ in 0..3 {
        consumer.receive_messages(10).await?;
        sleep(Duration::from_secs(1)).await;
    }

    println!("\n✨ Example completed successfully!");
    Ok(())
}

