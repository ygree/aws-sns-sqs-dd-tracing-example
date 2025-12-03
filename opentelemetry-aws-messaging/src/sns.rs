//! SNS message attribute carrier for OpenTelemetry context propagation.
//!
//! This module provides an [`Injector`] implementation that allows injecting
//! trace context into SNS message attributes.

use aws_sdk_sns::types::MessageAttributeValue;
use opentelemetry::propagation::Injector;
use std::collections::HashMap;

/// An [`Injector`] implementation for SNS message attributes.
///
/// Wraps a mutable reference to a `HashMap` of SNS message attributes and
/// implements the OpenTelemetry `Injector` trait, allowing trace context
/// to be injected into SNS messages.
///
/// # Example
///
/// ```ignore
/// use opentelemetry::global;
/// use opentelemetry_aws_messaging::sns::MessageAttributesInjector;
/// use aws_sdk_sns::types::MessageAttributeValue;
/// use std::collections::HashMap;
///
/// let mut attributes: HashMap<String, MessageAttributeValue> = HashMap::new();
///
/// // Inject trace context from the current span
/// global::get_text_map_propagator(|propagator| {
///     propagator.inject_context(&cx, &mut MessageAttributesInjector(&mut attributes));
/// });
///
/// // Now use `attributes` when publishing to SNS
/// client.publish()
///     .topic_arn(&topic_arn)
///     .message(&message_body)
///     .set_message_attributes(Some(attributes))
///     .send()
///     .await?;
/// ```
pub struct MessageAttributesInjector<'a>(pub &'a mut HashMap<String, MessageAttributeValue>);

impl Injector for MessageAttributesInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(
            key.to_string(),
            MessageAttributeValue::builder()
                .data_type("String")
                .string_value(value)
                .build()
                .expect("MessageAttributeValue build should not fail with valid String data_type"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injector_sets_string_attribute() {
        let mut attrs = HashMap::new();
        let mut injector = MessageAttributesInjector(&mut attrs);

        injector.set("traceparent", "00-abc123-def456-01".to_string());

        assert!(attrs.contains_key("traceparent"));
        let attr = attrs.get("traceparent").unwrap();
        assert_eq!(attr.data_type(), "String");
        assert_eq!(attr.string_value(), Some("00-abc123-def456-01"));
    }

    #[test]
    fn test_injector_overwrites_existing_key() {
        let mut attrs = HashMap::new();
        let mut injector = MessageAttributesInjector(&mut attrs);

        injector.set("key", "value1".to_string());
        injector.set("key", "value2".to_string());

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs.get("key").unwrap().string_value(), Some("value2"));
    }
}

