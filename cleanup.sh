#!/bin/bash
set -e

echo "🧹 Cleaning up AWS resources..."
echo ""

# Load environment variables from .env
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found"
    echo "   Cannot determine which resources to delete."
    echo "   Please run ./setup.sh first or create .env manually."
    exit 1
fi

echo "📝 Loading configuration from .env file..."
source .env
echo ""

# Verify required variables are set
if [ -z "$SNS_TOPIC_ARN" ] || [ -z "$SQS_QUEUE_URL" ]; then
    echo "❌ Error: Required variables not found in .env"
    echo "   Expected: SNS_TOPIC_ARN and SQS_QUEUE_URL"
    exit 1
fi

echo "📌 Resources to delete:"
echo "   SNS Topic ARN: $SNS_TOPIC_ARN"
echo "   SQS Queue URL: $SQS_QUEUE_URL"
echo ""

# Unsubscribe and delete SNS topic
if [ ! -z "$SNS_TOPIC_ARN" ]; then
    echo "📢 Deleting SNS subscriptions and topic..."
    
    # Get all subscriptions for the topic
    SUBSCRIPTIONS=$(aws sns list-subscriptions-by-topic --topic-arn $SNS_TOPIC_ARN --query 'Subscriptions[*].SubscriptionArn' --output text 2>/dev/null || echo "")
    
    # Unsubscribe each
    for SUB_ARN in $SUBSCRIPTIONS; do
        if [ "$SUB_ARN" != "PendingConfirmation" ]; then
            aws sns unsubscribe --subscription-arn $SUB_ARN 2>/dev/null || true
            echo "   Deleted subscription: $SUB_ARN"
        fi
    done
    
    # Delete topic
    aws sns delete-topic --topic-arn $SNS_TOPIC_ARN 2>/dev/null || true
    echo "   Deleted topic: $SNS_TOPIC_ARN"
    echo ""
fi

# Delete SQS queue
if [ ! -z "$SQS_QUEUE_URL" ]; then
    echo "📦 Deleting SQS queue..."
    aws sqs delete-queue --queue-url $SQS_QUEUE_URL 2>/dev/null || true
    echo "   Deleted queue: $SQS_QUEUE_URL"
    echo ""
fi

# Remove .env file
if [ -f .env ]; then
    rm .env
    echo "📝 Removed .env file"
    echo ""
fi

echo "✅ Cleanup complete!"
echo ""
echo "Note: It may take up to 60 seconds for queues to be fully deleted."
echo ""
echo "💡 If you sourced .env in your shell, you can unset the variables with:"
echo "   unset SNS_TOPIC_ARN SQS_QUEUE_URL"
echo ""
echo "   Or run: source <(echo 'unset SNS_TOPIC_ARN SQS_QUEUE_URL')"

