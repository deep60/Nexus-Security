# Verdyx - Terraform Variables Example
# Copy this file to terraform.tfvars and customize values

# AWS Configuration
aws_region = "us-east-1"

# Project Configuration
project_name = "verdyx"
environment  = "development"  # development, staging, production

# VPC Configuration
vpc_cidr             = "10.0.0.0/16"
private_subnet_cidrs = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
public_subnet_cidrs  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

# EKS Configuration
cluster_name       = "verdyx-cluster"
kubernetes_version = "1.33"
# m7i-flex.large (2 vCPU / 8 GB) is the largest free-plan-eligible EC2 type on
# this account; t3/t4g large/xlarge and r5 are blocked. Counts kept small for a
# free-plan sandbox (each node beyond ~750 hrs/mo is billed).
node_instance_types = ["m7i-flex.large"]
min_node_count     = 1
max_node_count     = 2
desired_node_count = 1

# RDS Configuration
rds_instance_class        = "db.t3.micro"
rds_allocated_storage     = 20
rds_max_allocated_storage = 100

# Redis Configuration
redis_node_type = "cache.t3.medium"

# Domain
domain_name = "verdyx.com"

# Additional Tags
additional_tags = {
  Team        = "security"
  CostCenter  = "engineering"
}
