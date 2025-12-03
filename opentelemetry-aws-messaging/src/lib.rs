//! OpenTelemetry context propagation carriers for AWS SNS and SQS.
//!
//! This crate provides [`Injector`] and [`Extractor`] implementations for AWS SNS and SQS
//! message attributes, enabling distributed tracing context propagation across AWS messaging
//! services.
//!
//! # Features
//!
//! - `sns` - Enables SNS message attribute injection (enabled by default)
//! - `sqs` - Enables SQS message attribute extraction (enabled by default)
//!
//! # Example
//!
//! ## Publishing to SNS with trace context
//!
//! ```ignore
//! use opentelemetry::global;
//! use opentelemetry_aws_messaging::sns::MessageAttributesInjector;
//! use std::collections::HashMap;
//!
//! let mut attributes = HashMap::new();
//! global::get_text_map_propagator(|propagator| {
//!     propagator.inject_context(&cx, &mut MessageAttributesInjector(&mut attributes));
//! });
//! // Use `attributes` in your SNS publish call
//! ```
//!
//! ## Consuming from SQS with trace context
//!
//! ```ignore
//! use opentelemetry::global;
//! use opentelemetry_aws_messaging::sqs::MessageAttributesExtractor;
//!
//! let parent_cx = global::get_text_map_propagator(|propagator| {
//!     propagator.extract(&MessageAttributesExtractor(msg.message_attributes()))
//! });
//! // Use `parent_cx` to create child spans
//! ```

#[cfg(feature = "sns")]
pub mod sns;

#[cfg(feature = "sqs")]
pub mod sqs;

// Re-exports for convenience
#[cfg(feature = "sns")]
pub use sns::MessageAttributesInjector as SnsMessageAttributesInjector;

#[cfg(feature = "sqs")]
pub use sqs::MessageAttributesExtractor as SqsMessageAttributesExtractor;

