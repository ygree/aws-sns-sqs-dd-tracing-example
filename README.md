# SNS/SQS Example with OpenTelemetry Distributed Tracing

A Rust example demonstrating AWS SNS → SQS messaging with OpenTelemetry distributed tracing propagated via message attributes. Traces are sent to Datadog.

<img width="1044" height="84" alt="image" src="https://github.com/user-attachments/assets/76c22795-7cae-4fad-ab32-89f6090f6e9c" />


## Overview

This project includes:

- **Producer**: Publishes messages to an SNS topic, injecting trace context into message attributes
- **Consumer**: Polls messages from an SQS queue (subscribed to the SNS topic), extracting and continuing the trace context
- **opentelemetry-aws-messaging**: A small crate providing `Injector`/`Extractor` implementations for SNS and SQS message attributes

## Prerequisites

- Rust toolchain
- AWS CLI configured with SSO or credentials
- `AWS_PROFILE` environment variable set

## Quick Start

### 1. Set up AWS resources

```bash
export AWS_PROFILE=your-profile
./setup.sh
```

This creates:
- An SNS topic and SQS queue (names defined in `setup.sh`)
- A subscription linking them (with raw message delivery enabled)
- A `.env` file with the resource ARNs

### 2. Run the example

**Simple pub/sub demo** (no tracing, publishes 3 messages then consumes them):
```bash
./run.sh
```

**With distributed tracing** (run producer and consumer in separate terminals):

Terminal 1:
```bash
./run-consumer.sh
```

Terminal 2:
```bash
./run-producer.sh
```

Type messages in the producer terminal and see them received in the consumer with linked traces in Datadog.

### 3. Cleanup

```bash
./cleanup.sh
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AWS_PROFILE` | AWS CLI profile to use |
| `SNS_TOPIC_ARN` | ARN of the SNS topic (set by setup.sh) |
| `SQS_QUEUE_URL` | URL of the SQS queue (set by setup.sh) |
| `DD_SERVICE` | Datadog service name (set by run scripts) |
| `DD_LOG_LEVEL` | Set to `DEBUG` to enable verbose logging from the Datadog tracing library |

## Scripts

| Script | Description |
|--------|-------------|
| `setup.sh` | Creates SNS topic, SQS queue, subscription with raw message delivery, queue policy, and generates `.env` file. Runs a test message to verify setup. |
| `run.sh` | Runs the simple pub/sub demo without tracing (`src/main.rs`) |
| `run-producer.sh` | Runs the interactive producer with `DD_SERVICE=sns-producer` |
| `run-consumer.sh` | Runs the consumer with `DD_SERVICE=sns-consumer` |
| `cleanup.sh` | Deletes SNS subscriptions, topic, SQS queue, and removes `.env` file |

## Project Structure

```
├── src/
│   ├── main.rs        # Simple pub/sub demo (no tracing)
│   ├── producer.rs    # Interactive SNS publisher with tracing
│   └── consumer.rs    # SQS consumer with tracing
├── opentelemetry-aws-messaging/
│   ├── src/
│   │   ├── lib.rs     # Library exports
│   │   ├── sns.rs     # SNS message attributes injector
│   │   └── sqs.rs     # SQS message attributes extractor
│   └── Cargo.toml
├── setup.sh
├── cleanup.sh
├── run.sh
├── run-producer.sh
└── run-consumer.sh
```

