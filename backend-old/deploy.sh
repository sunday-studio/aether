#!/bin/bash
set -e

# ------------------------
# CONFIG
# ------------------------
PI_USER="alfather"
PI_HOST="192.168.1.108"               # Tailscale IP or Pi hostname
PI_BACKEND_DIR="/home/alfather/aether-backend"
PM2_NAME="aether-backend"
KEEP_LAST=3                           # Keep last N builds on Pi

# ------------------------
# VERSIONING
# ------------------------
VERSION=$(date +%Y%m%d_%H%M%S)
BINARY_NAME="aether-backend_$VERSION"

# ------------------------
# BUILD USING DOCKER
# ------------------------
echo "🔨 Building backend with Docker..."

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

docker build \
    --build-arg VERSION=$VERSION \
    --build-arg LIBSQL_URL="${LIBSQL_URL}" \
    --build-arg LIBSQL_USE_REPLICA="${LIBSQL_USE_REPLICA}" \
    --build-arg LIBSQL_SYNC_INTERVAL="${LIBSQL_SYNC_INTERVAL}" \
    --build-arg LIBSQL_AUTH_TOKEN="${LIBSQL_AUTH_TOKEN}" \
    -t aether-backend-builder .

# Create temporary container and copy binary and .env out
CONTAINER_ID=$(docker create aether-backend-builder)
docker cp $CONTAINER_ID:/src/aether-backend_$VERSION ./$BINARY_NAME
docker cp $CONTAINER_ID:/src/.env ./bundled.env
docker rm $CONTAINER_ID >/dev/null
echo "✅ Build complete: $BINARY_NAME"

# ------------------------
# COPY TO PI
# ------------------------
echo "📦 Copying binary and bundled .env to Pi..."
ssh $PI_USER@$PI_HOST "mkdir -p $PI_BACKEND_DIR"
scp $BINARY_NAME $PI_USER@$PI_HOST:$PI_BACKEND_DIR/
scp bundled.env $PI_USER@$PI_HOST:$PI_BACKEND_DIR/.env
rm -f bundled.env
echo "✅ Copy complete"

# ------------------------
# STOP OLD SERVICE AND START NEW ONE
# ------------------------
echo "🔄 Stopping old service and starting new binary..."
ssh $PI_USER@$PI_HOST "bash -l -c '
cd $PI_BACKEND_DIR
export NVM_DIR=\"\$HOME/.nvm\"
[ -s \"\$NVM_DIR/nvm.sh\" ] && \. \"\$NVM_DIR/nvm.sh\"
pm2 stop $PM2_NAME || true
pm2 delete $PM2_NAME || true
pm2 start ./$BINARY_NAME --name $PM2_NAME
echo \"✅ Backend started with new binary\"
'"

# ------------------------
# CLEAN OLD BINARIES ON PI
# ------------------------
echo "🧹 Cleaning old binaries on Pi..."
ssh $PI_USER@$PI_HOST "bash -l -c '
cd $PI_BACKEND_DIR
ls -1t aether-backend_* | tail -n +$((KEEP_LAST+1)) | xargs -r rm -f
'"

# ------------------------
# CLEAN LOCAL
# ------------------------
rm -f $BINARY_NAME
echo "✅ Deployment complete!"
