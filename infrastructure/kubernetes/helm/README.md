# Verdyx Helm Chart (skeleton)

This is a starting point for packaging Verdyx as a Helm chart. The raw YAML
under `infrastructure/kubernetes/<service>/` is the source of truth today;
this chart wraps those resources for easier multi-environment rollout.

## Status

- `Chart.yaml` and `values.yaml` define the structure and defaults.
- Templates are not yet implemented. To add a service, create
  `templates/<service>-deployment.yaml` with `{{ .Values.services.<svc> }}`
  bindings, mirroring the existing raw manifest.

## Workflow

```bash
# Install in staging
helm upgrade --install verdyx ./infrastructure/kubernetes/helm \
  -f ./infrastructure/kubernetes/helm/values-staging.yaml \
  --namespace verdyx --create-namespace

# Install in production
helm upgrade --install verdyx ./infrastructure/kubernetes/helm \
  --namespace verdyx --create-namespace
```

## Until templates are written

Apply the raw manifests directly:

```bash
kubectl apply -f infrastructure/kubernetes/namespace.yaml
kubectl apply -f infrastructure/kubernetes/base/
kubectl apply -f infrastructure/kubernetes/<service>/
kubectl apply -f infrastructure/kubernetes/ingress.yaml
```
