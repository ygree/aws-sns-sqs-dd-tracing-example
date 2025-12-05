use anyhow::{Context as AnyhowContext, Result};
use aws_sdk_sqs::Client as SqsClient;
use opentelemetry::global;
use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry_aws_messaging::SqsMessageAttributesExtractor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::process;
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
    // Initialize the Datadog OpenTelemetry tracer
    let tracer_provider = datadog_opentelemetry::tracing().init();
    let tracer = global::tracer("my-sqs-consumer");

    println!("📥 SQS Consumer");
    println!("===============\n");

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = SqsClient::new(&config);

    let queue_url = env::var("SQS_QUEUE_URL")
        .context("SQS_QUEUE_URL environment variable not set")?;

    println!("📌 Consuming from: {}", queue_url);
    println!("🔄 Polling for messages... (Press Ctrl+C to stop)\n");

    let mut message_count = 0;

    // Set up Ctrl+C handler for graceful shutdown
    ctrlc::set_handler(move || {
        println!("\n👋 Shutting down gracefully...");
        tracer_provider
            .shutdown_with_timeout(Duration::from_secs(5))
            .expect("Failed to shutdown tracer provider");
        println!("✅ Shutdown complete");
        process::exit(0);
    })?;

    loop {
        match client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(20) // Long polling
            .message_attribute_names("All")
            .send()
            .await
        {
            Ok(response) => {
                if let Some(messages) = response.messages {
                    if !messages.is_empty() {
                        for msg in messages {
                            message_count += 1;

                            // Debug: inspect what SQS message provides
                            println!("   [debug] msg.message_attributes().is_some(): {}", msg.message_attributes().is_some());

                            // Extract trace context from SQS message attributes
                            let empty = HashMap::new();
                            let attrs = msg.message_attributes().unwrap_or(&empty);

                            let parent_cx = global::get_text_map_propagator(|propagator| {
                                propagator.extract(&SqsMessageAttributesExtractor(attrs))
                            });

                            let parent_span_ctx = parent_cx.span().span_context().clone();
                            println!("   [debug] Parent context valid: {}", parent_span_ctx.is_valid());

                            let span = tracer
                                .span_builder("sqs.process")
                                .with_kind(SpanKind::Consumer)
                                .start_with_context(&tracer, &parent_cx);
                            let cx = parent_cx.with_span(span);
                            let _guard = cx.attach();

                            if let Some(body) = msg.body() {
                                match serde_json::from_str::<Message>(body) {
                                    Ok(message) => {
                                        println!("📨 [{}] Received: {}", message_count, message.content);
                                        println!("   ID: {}, Timestamp: {}", message.id, message.timestamp);
                                    }
                                    Err(_) => {
                                        println!("📨 [{}] Received: {}", message_count, body);
                                    }
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

