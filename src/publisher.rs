/// Standalone SNS Publisher
/// Run with: cargo run --bin publisher
use anyhow::{Context, Result};
use aws_sdk_sns::Client as SnsClient;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u32,
    content: String,
    timestamp: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("📤 SNS Publisher");
    println!("================\n");

    let config = aws_config::load_from_env().await;
    let client = SnsClient::new(&config);

    let topic_arn = std::env::var("SNS_TOPIC_ARN")
        .context("SNS_TOPIC_ARN environment variable not set")?;

    println!("📌 Publishing to: {}\n", topic_arn);

    let mut message_id = 1;

    loop {
        print!("Enter message (or 'quit' to exit): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            println!("👋 Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        let message = Message {
            id: message_id,
            content: input.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let message_body = serde_json::to_string(&message)?;

        match client
            .publish()
            .topic_arn(&topic_arn)
            .message(&message_body)
            .subject(format!("Message {}", message_id))
            .send()
            .await
        {
            Ok(response) => {
                println!(
                    "✅ Published! MessageId: {:?}\n",
                    response.message_id().unwrap_or("unknown")
                );
                message_id += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to publish: {}\n", e);
            }
        }
    }

    Ok(())
}

