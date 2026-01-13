# LibSQL Server Setup

Self-hosted libSQL database server on Mac Mini with weekly backups and authentication.

## 🏗️ Architecture

```
┌─────────────┐         ┌──────────────┐
│  Pi 5       │         │  Mac Mini    │
│  (Backend)  │ ◄─────► │  (Database)  │
│             │  ~5ms   │              │
└─────────────┘         └──────────────┘
                              │
                              ▼
                        ┌──────────────┐
                        │  Backups     │
                        │  (Weekly)    │
                        └──────────────┘
```

## 📦 What's Included

- **LibSQL Server**: Primary database server
- **Authentication**: JWT-based security
- **Automatic Backups**: Weekly backups (every Sunday 2 AM)
- **Backup Retention**: 60 days (~8-9 weekly backups)
- **Compression**: Backups are gzip compressed
- **Health Checks**: Automatic monitoring

## 🚀 Quick Start

### 1. Initial Setup (Mac Mini)

```bash
# Make the setup script executable
chmod +x setup-libsql.sh

# Run setup
./setup-libsql.sh
```

This will:
- Create required directories
- Generate JWT authentication key
- Create `.env.pi.example` with your auth token
- Display your Mac Mini's IP address

### 2. Start the Server

```bash
# Start services
docker-compose up -d

# Verify they're running
docker-compose ps

# Check logs
docker-compose logs -f
```

### 3. Configure Your Pi

Copy the `.env.pi.example` to your Pi:

```bash
# From Mac Mini
scp .env.pi.example pi@your-pi-ip:/path/to/aether-backend/.env

# On your Pi, the .env should look like:
LIBSQL_URL=http://192.168.1.x:8080
LIBSQL_AUTH_TOKEN=your-generated-token-here
LIBSQL_USE_REPLICA=true
LIBSQL_SYNC_INTERVAL=10s
```

### 4. Test Connection

```bash
# From your Pi
curl http://your-mac-mini-ip:8080/health
```

## 🔐 Authentication

The server uses JWT-based authentication. The token is stored in `jwt-key.pem` and must be provided by clients.

**Keep your `jwt-key.pem` secure!** If compromised:

```bash
# Stop server
docker-compose down

# Generate new key
openssl rand -base64 32 > jwt-key.pem
chmod 600 jwt-key.pem

# Update .env on your Pi with new token
# Restart server
docker-compose up -d
```

## 💾 Backup Management

### Automatic Backups

Backups run every **Sunday at 2 AM** Amsterdam time.

Schedule: `0 2 * * 0` (cron format)

### Check Backups

```bash
# List all backups
ls -lh libsql-backups/

# View backup logs
docker-compose logs backup
```

### Manual Backup

```bash
# Trigger backup immediately
docker-compose exec backup /backup.sh
```

### Restore from Backup

```bash
# Make restore script executable
chmod +x restore-backup.sh

# Run restore (interactive)
./restore-backup.sh
```

The restore script will:
1. Show all available backups
2. Let you select one
3. Stop the server
4. Backup current database (safety)
5. Restore selected backup
6. Restart server

## 📊 Monitoring

### Check Server Status

```bash
# View all services
docker-compose ps

# Check server health
curl http://localhost:8080/health

# View real-time logs
docker-compose logs -f libsql-server
```

### Check Disk Usage

```bash
# Database size
du -sh libsql-data/

# Backup sizes
du -sh libsql-backups/
```

### Performance Metrics

Expected latencies (Pi → Mac Mini):
- **Writes**: ~5-10ms
- **Reads** (replica mode): <1ms (local)
- **Reads** (direct mode): ~2-5ms

## 🛠️ Maintenance

### Update LibSQL Server

```bash
# Pull latest image
docker-compose pull

# Restart with new image
docker-compose up -d
```

### Clean Old Backups Manually

Backups are automatically cleaned after 60 days, but you can manually clean:

```bash
# Remove backups older than 30 days
find libsql-backups/ -name "backup-*.sqld.gz" -mtime +30 -delete
```

### Migrate to New Mac

```bash
# On old Mac
docker-compose down
tar czf libsql-backup.tar.gz libsql-data/ libsql-backups/ jwt-key.pem docker-compose.yml

# Copy to new Mac
scp libsql-backup.tar.gz new-mac:~/

# On new Mac
tar xzf libsql-backup.tar.gz
docker-compose up -d
```

## 🔧 Troubleshooting

### Server Won't Start

```bash
# Check logs
docker-compose logs libsql-server

# Common issues:
# 1. Port already in use
lsof -i :8080

# 2. Permission issues
chmod -R 755 libsql-data/

# 3. Corrupt database
# Restore from backup using restore-backup.sh
```

### Pi Can't Connect

```bash
# From Pi, test connection
ping your-mac-mini-ip
curl http://your-mac-mini-ip:8080/health

# Check firewall on Mac Mini
# System Settings → Network → Firewall
# Ensure ports 8080 and 5001 are allowed

# Check auth token matches
cat jwt-key.pem  # On Mac Mini
echo $LIBSQL_AUTH_TOKEN  # On Pi (should match)
```

### Slow Performance

```bash
# Check resource usage
docker stats libsql-server

# Check network latency from Pi
ping your-mac-mini-ip

# Increase resource limits in docker-compose.yml:
# deploy.resources.limits.memory: 4G
# deploy.resources.limits.cpus: '4'
```

### Backup Failed

```bash
# Check backup logs
docker-compose logs backup

# Check disk space
df -h

# Manual backup
docker-compose exec backup /backup.sh
```

## 📁 Directory Structure

```
.
├── docker-compose.yml          # Docker configuration
├── setup-libsql.sh            # Initial setup script
├── restore-backup.sh          # Backup restore script
├── jwt-key.pem                # JWT authentication key (keep secure!)
├── .env.pi.example            # Example Pi configuration
├── libsql-data/               # Database files
│   └── data.sqld              # Main database file
└── libsql-backups/            # Backup storage
    └── backup-*.sqld.gz       # Compressed weekly backups
```

## 🔒 Security Best Practices

1. **Keep `jwt-key.pem` secure** - never commit to git
2. **Firewall**: Only allow connections from your Pi's IP
3. **Backups**: Store off-site copies of critical backups
4. **Updates**: Keep Docker images updated
5. **Monitoring**: Set up alerts for backup failures

## 📈 Scaling

If you need more performance:

1. **Increase resources** in `docker-compose.yml`
2. **Add read replicas** on other machines
3. **Use direct mode** instead of replica mode for even faster writes
4. **Optimize network** - use wired connection, QoS

## 🆘 Support

- LibSQL Documentation: https://docs.turso.tech
- GitHub Issues: https://github.com/tursodatabase/libsql
- Discord: https://discord.gg/turso

## 📝 License

This setup uses libSQL Server, which is Apache 2.0 licensed.