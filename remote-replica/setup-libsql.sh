#!/bin/bash

# LibSQL Server Setup Script
# Run this on your Mac Mini to set up the libSQL server

set -e

echo "🚀 LibSQL Server Setup"
echo "====================="
echo ""

# Check if directories exist
if [ -d "libsql-data" ] || [ -d "libsql-backups" ] || [ -f "jwt-key.pem" ]; then
    echo "⚠️  Warning: Existing setup detected"
    echo "Found existing files/directories:"
    [ -d "libsql-data" ] && echo "  - libsql-data/"
    [ -d "libsql-backups" ] && echo "  - libsql-backups/"
    [ -f "jwt-key.pem" ] && echo "  - jwt-key.pem"
    echo ""
    read -p "Do you want to continue? This will NOT delete existing data. (y/N): " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Setup cancelled."
        exit 0
    fi
fi

echo ""
echo "📁 Creating directories..."
mkdir -p libsql-data
mkdir -p libsql-backups
echo "✓ Directories created"

echo ""
echo "🔑 Generating Ed25519 key pair for JWT authentication..."
if [ -f "jwt-key.pem" ]; then
    echo "⚠️  jwt-key.pem already exists, skipping generation"
    echo "   If you want to regenerate, delete jwt-key.pem and run this script again"
else
    # Generate Ed25519 private key
    openssl genpkey -algorithm ed25519 -out jwt-key.pem
    chmod 600 jwt-key.pem
    echo "✓ JWT Ed25519 key pair generated: jwt-key.pem"
fi

echo ""
echo "📋 Extracting public key for authentication token..."
# Extract the base64-encoded public key from the private key
# LibSQL uses the base64-encoded raw public key as the auth token
AUTH_TOKEN=$(openssl pkey -in jwt-key.pem -pubout -outform DER | tail -c 32 | base64)
echo "✓ Authentication token ready"

echo ""
echo "📝 Configuration Summary"
echo "======================="
echo ""
echo "LibSQL Server:"
echo "  - Data directory: $(pwd)/libsql-data"
echo "  - Backup directory: $(pwd)/libsql-backups"
echo "  - JWT key: $(pwd)/jwt-key.pem"
echo ""
echo "Backup Schedule:"
echo "  - Frequency: Weekly (every Sunday at 2 AM)"
echo "  - Retention: 60 days (~8-9 weekly backups)"
echo "  - Format: Compressed (.sqld.gz)"
echo ""
echo "🔐 Authentication Token (use this on your Pi):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$AUTH_TOKEN"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Get Mac Mini IP address
echo "🌐 Detecting Mac Mini IP address..."
MAC_IP=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || echo "UNABLE_TO_DETECT")

if [ "$MAC_IP" != "UNABLE_TO_DETECT" ]; then
    echo "✓ Detected IP: $MAC_IP"
else
    echo "⚠️  Could not auto-detect IP. Please find it manually with: ifconfig"
fi

echo ""
echo "📄 Creating .env.example for your Pi..."

cat > .env.pi.example << EOF
# Configuration for Pi 5 (Aether Backend)
# Copy this to your Pi and rename to .env

# LibSQL Server connection (your Mac Mini)
LIBSQL_URL=http://${MAC_IP}:8080
LIBSQL_AUTH_TOKEN=${AUTH_TOKEN}

# Use embedded replica for fast local reads + fast local network writes
LIBSQL_USE_REPLICA=true
LIBSQL_SYNC_INTERVAL=10s
EOF

echo "✓ Created .env.pi.example"

echo ""
echo "🎯 Next Steps"
echo "============="
echo ""
echo "1. Start the libSQL server on Mac Mini:"
echo "   docker-compose up -d"
echo ""
echo "2. Verify it's running:"
echo "   docker-compose ps"
echo "   docker-compose logs -f libsql-server"
echo ""
echo "3. Copy .env.pi.example to your Pi 5:"
echo "   scp .env.pi.example pi@your-pi-ip:/path/to/aether-backend/.env"
echo ""
echo "4. On your Pi, restart the Aether backend"
echo ""
echo "5. Test the connection from your Pi:"
echo "   curl -H 'Authorization: Bearer ${AUTH_TOKEN}' http://${MAC_IP}:8080/health"
echo ""
echo "6. Monitor backups:"
echo "   docker-compose logs -f backup"
echo "   ls -lh libsql-backups/"
echo ""
echo "7. Manual backup (if needed):"
echo "   docker-compose exec backup /backup.sh"
echo ""
echo "✨ Setup complete! Your libSQL server is ready to start."
echo ""