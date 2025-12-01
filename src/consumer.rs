/// Standalone SQS Consumer
/// Run with: cargo run --bin consumer
use anyhow::{Context, Result};
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("📥 SQS Consumer");
    println!("===============\n");

    let config = aws_config::load_from_env().await;
    let client = SqsClient::new(&config);

    let queue_url = std::env::var("SQS_QUEUE_URL")
        .context("SQS_QUEUE_URL environment variable not set")?;

    println!("📌 Consuming from: {}", queue_url);
    println!("🔄 Polling for messages... (Press Ctrl+C to stop)\n");

    let mut message_count = 0;

    loop {
        match client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(20) // Long polling
            .send()
            .await
        {
            Ok(response) => {
                if let Some(messages) = response.messages {
                    if !messages.is_empty() {
                        for msg in messages {
                            message_count += 1;
                            
                            if let Some(body) = msg.body() {
                                // Parse SNS envelope
                                if let Ok(sns_envelope) = serde_json::from_str::<serde_json::Value>(body) {
                                    if let Some(message_str) = sns_envelope.get("Message").and_then(|m| m.as_str()) {
                                        match serde_json::from_str::<Message>(message_str) {
                                            Ok(message) => {
                                                println!("📨 [{}] Received: {}", message_count, message.content);
                                                println!("   ID: {}, Timestamp: {}", message.id, message.timestamp);
                                            }
                                            Err(_) => {
                                                println!("📨 [{}] Received: {}", message_count, message_str);
                                            }
                                        }
                                    }
                                } else {
                                    println!("📨 [{}] Raw message: {}", message_count, body);
                                }
                            }

                            // Delete the message
                            if let Some(receipt_handle) = msg.receipt_handle() {
                                if let Err(e) = client
                                    .delete_message()
                                    .queue_url(&queue_url)
                                    .receipt_handle(receipt_handle)
                                    .send()
                                    .await
                                {
                                    eprintln!("⚠️  Failed to delete message: {}", e);
                                } else {
                                    println!("   ✓ Deleted\n");
                                }
                            }
                        }
                    } else {
                        print!(".");
                        io::stdout().flush().ok();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error receiving messages: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

use std::io::{self, Write};

