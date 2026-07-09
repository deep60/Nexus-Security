# Verdyx - Terraform Infrastructure

This Terraform configuration deploys the cloud infrastructure for Verdyx on AWS.

## Resources Created

- **VPC**: Network with public/private subnets across 3 AZs
- **EKS**: Managed Kubernetes cluster with autoscaling node groups
- **RDS**: PostgreSQL database with multi-AZ support
- **ElastiCache**: Redis cluster for caching
- **S3**: Bucket for file uploads

## Prerequisites

1. AWS CLI configured with appropriate credentials
2. Terraform >= 1.0
3. kubectl for Kubernetes management

> **Note on RDS/EKS for launch:** the launch stack runs the app tier on a
> single VM via Docker Compose and uses **RDS + ElastiCache** as its managed
> data tier (see `../DEPLOYMENT.md`). The **EKS** cluster defined here is the
> Phase-2 scale-out path and is not required to go live — you can `terraform
> apply -target` just the VPC/RDS/ElastiCache/S3 resources if you want to skip
> EKS for now.

## Remote state (do this first)

State is stored in S3 with DynamoDB locking (see the `backend "s3"` block in
`main.tf`). The bucket and lock table must exist **before** the first
`terraform init`:

```bash
# One-time, idempotent. Needs AWS creds that can create S3 + DynamoDB.
./scripts/bootstrap-tf-state.sh
```

Then proceed to Quick Start. If you already have local state, `terraform init`
will offer to migrate it into S3.

## Quick Start

```bash
# Initialize Terraform (uses the S3 backend created above)
terraform init

# Copy and customize variables
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values

# Preview changes
terraform plan

# Apply infrastructure
terraform apply

# Configure kubectl
$(terraform output -raw configure_kubectl)
```

## Environment-Specific Deployment

### Development
```bash
terraform workspace new development
terraform apply -var="environment=development"
```

### Production
```bash
terraform workspace new production
terraform apply -var="environment=production" \
  -var="rds_instance_class=db.r5.large" \
  -var="redis_node_type=cache.r5.large" \
  -var="min_node_count=3" \
  -var="max_node_count=20"
```

## Outputs

After applying, important outputs include:
- `cluster_endpoint`: EKS API endpoint
- `rds_endpoint`: PostgreSQL connection endpoint
- `redis_endpoint`: Redis connection endpoint
- `database_url`: Connection string for applications

## Cost Optimization

For development:
- Single NAT gateway
- Smaller instance types
- No Multi-AZ for RDS/Redis

For production:
- Multi-AZ enabled
- Higher-spec instances
- Enhanced monitoring

## Security

- All databases in private subnets
- Security groups restrict access to EKS only
- S3 bucket has public access blocked
- Encryption at rest enabled

## Cleanup

```bash
terraform destroy
```

**Warning**: This will delete all resources including databases. Ensure backups are made first.
