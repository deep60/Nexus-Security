# Nexus Security - Terraform Infrastructure

This Terraform configuration deploys the cloud infrastructure for Nexus Security on AWS.

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

## Quick Start

```bash
# Initialize Terraform
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
