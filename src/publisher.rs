/// Standalone SNS Publisher with OpenTelemetry tracing
/// Run with: cargo run --bin publisher
use anyhow::{Context, Result};
use aws_sdk_sns::Client as SnsClient;
use opentelemetry::trace::{Tracer, TraceContextExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u32,
    content: String,
    timestamp: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Datadog OpenTelemetry tracer
    let tracer_provider = datadog_opentelemetry::tracing().init();
    let tracer = opentelemetry::global::tracer("my-sns-publisher"); // this is not service name but set to the otel.scope.name tag

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

        // Create a span for the publish operation
        let span = tracer.start("sns.publish");
        let cx = opentelemetry::Context::current_with_span(span);
        let _guard = cx.attach();

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

    // Shutdown the tracer provider to flush any remaining spans
    tracer_provider
        .shutdown_with_timeout(Duration::from_secs(5))
        .context("Failed to shutdown tracer provider")?;

    Ok(())
}
