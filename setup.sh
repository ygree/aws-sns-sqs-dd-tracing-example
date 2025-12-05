#!/bin/bash
set -e

echo "🚀 Setting up AWS SNS/SQS infrastructure..."
echo ""

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI is not installed. Please install it first:"
    echo "   https://aws.amazon.com/cli/"
    exit 1
fi

# Check if AWS_PROFILE is set
if [ -z "$AWS_PROFILE" ]; then
    echo "❌ AWS_PROFILE environment variable is not set."
    exit 1
fi

# Check AWS credentials
if ! aws sts get-caller-identity &> /dev/null; then
    echo "❌ AWS credentials not configured. Run 'aws sso login' first."
    exit 1
fi

echo "✅ AWS CLI configured"
echo ""

# Step 1: Create SQS Queue
echo "📦 Creating SQS queue..."
QUEUE_NAME="sns-sqs-example-queue"
if SQS_QUEUE_URL=$(aws sqs create-queue --queue-name $QUEUE_NAME --query 'QueueUrl' --output text 2>&1); then
    echo "   Queue created"
else
    echo "   Queue may already exist, getting URL..."
    SQS_QUEUE_URL=$(aws sqs get-queue-url --queue-name $QUEUE_NAME --query 'QueueUrl' --output text)
fi

echo "   Queue URL: $SQS_QUEUE_URL"

# Get Queue ARN
SQS_QUEUE_ARN=$(aws sqs get-queue-attributes \
    --queue-url $SQS_QUEUE_URL \
    --attribute-names QueueArn \
    --query 'Attributes.QueueArn' \
    --output text)

echo "   Queue ARN: $SQS_QUEUE_ARN"
echo ""

# Step 2: Create SNS Topic
echo "📢 Creating SNS topic..."
TOPIC_NAME="sns-sqs-example-topic"
SNS_TOPIC_ARN=$(aws sns create-topic --name $TOPIC_NAME --query 'TopicArn' --output text 2>/dev/null || \
                aws sns list-topics --query "Topics[?contains(TopicArn, '$TOPIC_NAME')].TopicArn" --output text | head -1)

echo "   Topic ARN: $SNS_TOPIC_ARN"
echo ""

# Step 3: Subscribe SQS to SNS
echo "🔗 Subscribing SQS queue to SNS topic..."
SUBSCRIPTION_ARN=$(aws sns subscribe \
    --topic-arn $SNS_TOPIC_ARN \
    --protocol sqs \
    --notification-endpoint $SQS_QUEUE_ARN \
    --query 'SubscriptionArn' \
    --output text 2>/dev/null || echo "Already subscribed")

echo "   Subscription ARN: $SUBSCRIPTION_ARN"

# Enable RawMessageDelivery to use the recommended out-of-band approach, passing trace context via message attributes instead of the message body.
# Otherwise, SNS uses the default in-band approach, wrapping the message and serializing attributes into the body.
# This breaks distributed tracing, as trace context is expected out-of-band when extracting from SQS messages.
aws sns set-subscription-attributes \
    --subscription-arn "$SUBSCRIPTION_ARN" \
    --attribute-name RawMessageDelivery \
    --attribute-value true
echo ""

# Step 4: Set Queue Policy
echo "🔐 Setting queue policy..."

# Create a temporary file for the policy (escaped for JSON)
POLICY_FILE=$(mktemp)
cat > "$POLICY_FILE" <<EOF
{"Policy":"{\"Version\":\"2012-10-17\",\"Statement\":[{\"Effect\":\"Allow\",\"Principal\":{\"Service\":\"sns.amazonaws.com\"},\"Action\":\"sqs:SendMessage\",\"Resource\":\"$SQS_QUEUE_ARN\",\"Condition\":{\"ArnEquals\":{\"aws:SourceArn\":\"$SNS_TOPIC_ARN\"}}}]}"}
EOF

aws sqs set-queue-attributes \
    --queue-url "$SQS_QUEUE_URL" \
    --attributes file://"$POLICY_FILE" > /dev/null

# Clean up temp file
rm -f "$POLICY_FILE"

echo "   Queue policy set"
echo ""

# Step 5: Create .env file
echo "📝 Creating .env file..."
cat > .env <<EOF
AWS_PROFILE=$AWS_PROFILE
SNS_TOPIC_ARN=$SNS_TOPIC_ARN
SQS_QUEUE_URL=$SQS_QUEUE_URL
EOF
echo "   .env file created"
echo ""

# Step 6: Test the setup
echo "🧪 Testing the setup..."
TEST_MESSAGE="Test message from setup script"
aws sns publish \
    --topic-arn $SNS_TOPIC_ARN \
    --message "$TEST_MESSAGE" \
    --output text > /dev/null

sleep 2

# Check if message arrived
MESSAGE_COUNT=$(aws sqs get-queue-attributes \
    --queue-url $SQS_QUEUE_URL \
    --attribute-names ApproximateNumberOfMessages \
    --query 'Attributes.ApproximateNumberOfMessages' \
    --output text)

if [ "$MESSAGE_COUNT" -gt 0 ]; then
    echo "   ✅ Test message received in queue!"
    # Clean up test message
    aws sqs receive-message --queue-url $SQS_QUEUE_URL --query 'Messages[0].ReceiptHandle' --output text | \
    xargs -I {} aws sqs delete-message --queue-url $SQS_QUEUE_URL --receipt-handle {} 2>/dev/null || true
else
    echo "   ⚠️  Test message not received yet (may take a few seconds)"
fi
echo ""

echo "✅ Setup complete!"
echo ""
echo "To run the example:"
echo "  ./run.sh"
echo ""
echo "To run producer only:"
echo "  ./run-producer.sh"
echo ""
echo "To run consumer only:"
echo "  ./run-consumer.sh"
echo ""
echo "⚠️  Note: If you get authentication errors, run:"
echo "  aws sso login --profile $AWS_PROFILE"
echo ""
echo "To cleanup resources, run:"
echo "  ./cleanup.sh"

