# Kubernetes Deployment

This repository includes Kubernetes manifests in `infrastructure/kubernetes`.

## Prerequisites

- Kubernetes cluster (dev/staging/prod)
- `kubectl` configured
- Container images available in your registry
- Secrets prepared for DB, Redis, JWT, blockchain credentials

## Suggested Apply Order

1. Namespace/base objects
2. Database/cache dependencies
3. Core services
4. Ingress
5. Monitoring and autoscaling objects

Example:

```bash
kubectl apply -f infrastructure/kubernetes/base
kubectl apply -f infrastructure/kubernetes/database
kubectl apply -f infrastructure/kubernetes/api-gateway
kubectl apply -f infrastructure/kubernetes/analysis-engine
kubectl apply -f infrastructure/kubernetes/bounty-manager
kubectl apply -f infrastructure/kubernetes/ingress
```

## Verify Rollout

```bash
kubectl get pods -n verdyx
kubectl get svc -n verdyx
kubectl get ingress -n verdyx
```

## Health Verification

Use port-forward + health endpoint:

```bash
kubectl port-forward svc/verdyx-api-gateway 8080:8080 -n verdyx
curl -f http://localhost:8080/api/v1/health
```

## Runtime Configuration

Use per-environment config templates from `config/`:

- `config/staging/*`
- `config/production/*`

Inject values via ConfigMaps and Secrets.

## Troubleshooting

- Crash loop: `kubectl logs <pod> -n verdyx --previous`
- Bad env/secret: `kubectl describe pod <pod> -n verdyx`
- Service routing: inspect Ingress class and service names
