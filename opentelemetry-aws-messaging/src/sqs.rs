//! SQS message attribute carrier for OpenTelemetry context propagation.
//!
//! This module provides an [`Extractor`] implementation that allows extracting
//! trace context from SQS message attributes.

use aws_sdk_sqs::types::MessageAttributeValue;
use opentelemetry::propagation::Extractor;
use std::collections::HashMap;

/// An [`Extractor`] implementation for SQS message attributes.
///
/// Wraps a reference to a `HashMap` of SQS message attributes and
/// implements the OpenTelemetry `Extractor` trait, allowing trace context
/// to be extracted from SQS messages.
///
/// # Example
///
/// ```ignore
/// use opentelemetry::global;
/// use opentelemetry::trace::{SpanKind, Tracer};
/// use opentelemetry_aws_messaging::sqs::MessageAttributesExtractor;
///
/// // Extract trace context from SQS message
/// let attrs = msg.message_attributes().unwrap_or(&HashMap::new());
/// let parent_cx = global::get_text_map_propagator(|propagator| {
///     propagator.extract(&MessageAttributesExtractor(attrs))
/// });
///
/// // Create a child span linked to the extracted context
/// let span = tracer
///     .span_builder("sqs.process")
///     .with_kind(SpanKind::Consumer)
///     .start_with_context(&tracer, &parent_cx);
/// ```
pub struct MessageAttributesExtractor<'a>(pub &'a HashMap<String, MessageAttributeValue>);

impl Extractor for MessageAttributesExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.string_value())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attr(value: &str) -> MessageAttributeValue {
        MessageAttributeValue::builder()
            .data_type("String")
            .string_value(value)
            .build()
            .unwrap()
    }

    #[test]
    fn test_extractor_gets_existing_key() {
        let mut attrs = HashMap::new();
        attrs.insert("traceparent".to_string(), make_attr("00-abc123-def456-01"));

        let extractor = MessageAttributesExtractor(&attrs);

        assert_eq!(extractor.get("traceparent"), Some("00-abc123-def456-01"));
    }

    #[test]
    fn test_extractor_returns_none_for_missing_key() {
        let attrs = HashMap::new();
        let extractor = MessageAttributesExtractor(&attrs);

        assert_eq!(extractor.get("nonexistent"), None);
    }

    #[test]
    fn test_extractor_keys_returns_all_keys() {
        let mut attrs = HashMap::new();
        attrs.insert("key1".to_string(), make_attr("value1"));
        attrs.insert("key2".to_string(), make_attr("value2"));

        let extractor = MessageAttributesExtractor(&attrs);
        let mut keys = extractor.keys();
        keys.sort();

        assert_eq!(keys, vec!["key1", "key2"]);
    }
}

