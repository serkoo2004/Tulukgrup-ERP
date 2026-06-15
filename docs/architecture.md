# Architecture

## Runtime Topology

```text
Tauri Desktop / React Frontend
        |
        v
Nginx Reverse Proxy
        |
        +--> Rust API (/api/v1)
        |       - Auth
        |       - Organization
        |       - ERP core
        |       - Audit logs
        |       - PostgreSQL access
        |
        +--> Go Realtime Gateway (/ws)
                - GPS WebSocket streams
                - TCP socket ingest
                - Queue bridge for NATS/Kafka
```

## Data Access Rule

PostgreSQL disariya acilmaz. Tum veritabani erisimleri Rust API ve gerekli servis hesaplari uzerinden yapilir.

## Current Backend Scope

- JWT access token
- Refresh token persistence
- bcrypt password hashing
- Role based authorization
- Audit log writes
- Company and department management
- Vehicles CRUD
- Sold vehicle transaction flow
- Users API and role catalogue
- KM tracking and suspicious entry task creation
- Maintenance record lifecycle
- Insurance policy lifecycle
- Task lifecycle and assigned-user task view
- Notification lifecycle with WhatsApp-ready channel field
- File storage with database path/reference only
- Excel `.xlsx` export reports
- Import job tracking and validation error records
- System error log records
- AI analysis job records with human approval
- Basic API rate limiting
- PostgreSQL migration baseline

## Current Realtime Scope

- `/health` service status
- `/ws` and `/ws/gps` WebSocket clients
- TCP GPS ingest on `REALTIME_TCP_PORT`
- JSON GPS packet broadcast to connected WebSocket clients
