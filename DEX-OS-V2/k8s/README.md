# Kubernetes deployment quickstart

1. Build and push images to your registry:
   - API: `docker build -t ghcr.io/your-org/dex-api:latest -f dex-api/Dockerfile .`
   - UI: `docker build -t ghcr.io/your-org/dex-ui:latest -f dex-ui/Dockerfile .`
   - Push both images (`docker push ...`).
2. Update image names/tags in:
   - `k8s/api-deployment.yaml`
   - `k8s/ui-deployment.yaml`
3. Set real secrets in `k8s/api-secret.yaml` (`JWT_SECRET`, `DATABASE_URL`, `TRADER_SECRETS` as needed). Consider using an external secret manager instead of in-cluster secrets.
4. Apply manifests: `kubectl apply -f k8s/`.
5. Expose externally via Ingress/LoadBalancer as needed (services are `ClusterIP` by default).
