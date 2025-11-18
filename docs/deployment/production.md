# Production Deployment Guide

This guide covers deploying Nexus Security to a production environment.

## Pre-Deployment Checklist

- [ ] SSL/TLS certificates obtained
- [ ] Domain DNS configured
- [ ] Secrets securely stored (Vault, AWS Secrets Manager)
- [ ] Database backups configured
- [ ] Monitoring and alerting set up
- [ ] Security audit completed
- [ ] Load testing performed

## Infrastructure Setup

### Option 1: Terraform (Recommended)

```bash
cd infrastructure/terraform

# Initialize
terraform init

# Configure variables
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with production values

# Preview changes
terraform plan -var="environment=production"

# Apply
terraform apply -var="environment=production"
```

This creates:

- VPC with public/private subnets
- EKS cluster with autoscaling
- RDS PostgreSQL (Multi-AZ)
- ElastiCache Redis cluster
- S3 bucket for uploads
- Security groups

### Option 2: Docker Compose

For smaller deployments:

```bash
cd infrastructure/docker/production

# Configure environment
cp ../.env.example .env
# Edit .env with production values

# Build and start
docker-compose -f docker-compose.prod.yml up -d
```

## Security Configuration

### 1. Secrets Management

**Never store secrets in code or config files.**

Use external secrets management:

```bash
# AWS Secrets Manager
aws secretsmanager create-secret \
  --name nexus-security/production/database \
  --secret-string '{"password":"secure_password"}'

# Or Kubernetes External Secrets
kubectl apply -f external-secrets.yaml
```

### 2. TLS/SSL

Using cert-manager for automatic certificates:

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@nexus-security.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### 3. Network Policies

Restrict traffic between services:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-gateway-policy
  namespace: nexus-security
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgresql
    - podSelector:
        matchLabels:
          app: redis
```

## Database Configuration

### PostgreSQL Production Settings

```sql
-- Connection limits
ALTER SYSTEM SET max_connections = 200;

-- Memory settings
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET work_mem = '64MB';

-- Write performance
ALTER SYSTEM SET wal_buffers = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;

-- Enable SSL
ALTER SYSTEM SET ssl = on;
```

### Backups

```bash
# Automated daily backups (AWS RDS)
aws rds modify-db-instance \
  --db-instance-identifier nexus-postgres \
  --backup-retention-period 30 \
  --preferred-backup-window "03:00-04:00"
```

## Monitoring Setup

### Prometheus & Grafana

```bash
# Install Prometheus stack
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace
```

### Key Metrics to Monitor

- Request latency (p95, p99)
- Error rates (4xx, 5xx)
- Database connections
- Redis memory usage
- Analysis queue depth
- Blockchain transaction status

### Alerting Rules

```yaml
groups:
- name: nexus-alerts
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: High error rate detected

  - alert: AnalysisQueueBacklog
    expr: analysis_queue_depth > 100
    for: 10m
    labels:
      severity: warning
```

## Scaling Configuration

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: nexus-api-gateway
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Cluster Autoscaler

For EKS:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/autoscaler/master/cluster-autoscaler/cloudprovider/aws/examples/cluster-autoscaler-autodiscover.yaml
```

## Deployment Process

### 1. Build Images

```bash
export VERSION=$(git rev-parse --short HEAD)
./infrastructure/docker/scripts/build.sh
```

### 2. Push to Registry

```bash
export DOCKER_REGISTRY=your-registry.com/nexus-security
./infrastructure/docker/scripts/push.sh
```

### 3. Deploy to Kubernetes

```bash
# Update image tags
kubectl set image deployment/nexus-api-gateway \
  api-gateway=$DOCKER_REGISTRY/api-gateway:$VERSION \
  -n nexus-security

# Monitor rollout
kubectl rollout status deployment/nexus-api-gateway -n nexus-security
```

### 4. Verify Deployment

```bash
# Check pod status
kubectl get pods -n nexus-security

# Test health endpoints
curl https://api.nexus-security.com/health

# Check logs for errors
kubectl logs -l app=api-gateway -n nexus-security --tail=100
```

## Rollback Procedure

```bash
# Quick rollback
kubectl rollout undo deployment/nexus-api-gateway -n nexus-security

# Rollback to specific revision
kubectl rollout undo deployment/nexus-api-gateway --to-revision=2 -n nexus-security
```

## Disaster Recovery

### Database Recovery

```bash
# Restore from RDS snapshot
aws rds restore-db-instance-from-db-snapshot \
  --db-instance-identifier nexus-postgres-restored \
  --db-snapshot-identifier nexus-postgres-snapshot-20240115
```

### Full Cluster Recovery

1. Recreate infrastructure with Terraform
2. Restore database from backup
3. Deploy applications
4. Verify data integrity
5. Update DNS

## Performance Optimization

- Enable gzip compression in ingress
- Configure CDN for static assets
- Use connection pooling (PgBouncer)
- Enable Redis clustering
- Optimize database indexes

## Compliance

- Enable audit logging
- Configure log retention (1 year)
- Implement data encryption at rest
- Regular security scans
- Penetration testing quarterly
