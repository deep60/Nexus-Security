#!/usr/bin/env bash
#
# One-time bootstrap for Terraform remote state. Creates the S3 bucket that
# holds terraform.tfstate and the DynamoDB table used for state locking, so
# multiple people/CI can run Terraform safely without corrupting state.
#
# Run this ONCE, before the first `terraform init`, with AWS credentials that
# can create S3 buckets and DynamoDB tables. It is idempotent.
#
# The names/region MUST match the `backend "s3"` block in main.tf.
#
# Usage:
#   scripts/bootstrap-tf-state.sh
#   AWS_REGION=us-east-1 BUCKET=verdyx-terraform-state scripts/bootstrap-tf-state.sh
#
set -euo pipefail

AWS_REGION="${AWS_REGION:-us-east-1}"
BUCKET="${BUCKET:-verdyx-terraform-state}"
LOCK_TABLE="${LOCK_TABLE:-verdyx-terraform-locks}"

command -v aws >/dev/null 2>&1 || { echo "[error] aws CLI is required" >&2; exit 1; }

echo "==> Region:      $AWS_REGION"
echo "==> State bucket: $BUCKET"
echo "==> Lock table:   $LOCK_TABLE"

# ── S3 bucket ────────────────────────────────────────────────────────────────
if aws s3api head-bucket --bucket "$BUCKET" 2>/dev/null; then
  echo "==> [s3] bucket already exists"
else
  echo "==> [s3] creating bucket"
  # us-east-1 must NOT pass a LocationConstraint; every other region must.
  if [ "$AWS_REGION" = "us-east-1" ]; then
    aws s3api create-bucket --bucket "$BUCKET" --region "$AWS_REGION"
  else
    aws s3api create-bucket --bucket "$BUCKET" --region "$AWS_REGION" \
      --create-bucket-configuration LocationConstraint="$AWS_REGION"
  fi
fi

echo "==> [s3] enabling versioning (lets you recover a clobbered state)"
aws s3api put-bucket-versioning --bucket "$BUCKET" \
  --versioning-configuration Status=Enabled

echo "==> [s3] enabling default encryption"
aws s3api put-bucket-encryption --bucket "$BUCKET" \
  --server-side-encryption-configuration \
  '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}'

echo "==> [s3] blocking all public access"
aws s3api put-public-access-block --bucket "$BUCKET" \
  --public-access-block-configuration \
  BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true

# ── DynamoDB lock table ──────────────────────────────────────────────────────
if aws dynamodb describe-table --table-name "$LOCK_TABLE" --region "$AWS_REGION" >/dev/null 2>&1; then
  echo "==> [dynamodb] lock table already exists"
else
  echo "==> [dynamodb] creating lock table (LockID hash key, on-demand billing)"
  aws dynamodb create-table \
    --table-name "$LOCK_TABLE" \
    --attribute-definitions AttributeName=LockID,AttributeType=S \
    --key-schema AttributeName=LockID,KeyType=HASH \
    --billing-mode PAY_PER_REQUEST \
    --region "$AWS_REGION" >/dev/null
  aws dynamodb wait table-exists --table-name "$LOCK_TABLE" --region "$AWS_REGION"
fi

echo ""
echo "==> Bootstrap complete. Now run:"
echo "      cd infrastructure/terraform && terraform init"
echo "    (Terraform will offer to migrate existing local state into S3.)"
