NGINX_GATEWAY_FABRIC_TAG="v2.3.0"
GATEWAY_API_TAG="v1.4.1"

kubectl config use-context orbstack

# Gateway api
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/${GATEWAY_API_TAG}/standard-install.yaml
kubectl kustomize "https://github.com/nginx/nginx-gateway-fabric/config/crd/gateway-api/standard?ref=${NGINX_GATEWAY_FABRIC_TAG}" | kubectl apply -f -
kubectl apply --server-side -f https://raw.githubusercontent.com/nginx/nginx-gateway-fabric/${NGINX_GATEWAY_FABRIC_TAG}/deploy/crds.yaml
kubectl apply -f https://raw.githubusercontent.com/nginx/nginx-gateway-fabric/${NGINX_GATEWAY_FABRIC_TAG}/deploy/default/deploy.yaml

# Local overlay

kubectl apply -k k8s/overlays/local

# Check
kubectl get pods