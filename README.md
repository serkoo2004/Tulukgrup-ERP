# enterprise-erp-platform

Bu proje, `erp_teknoloji_stack_raporu (2).pdf` dosyasindaki mimariye gore baslatildi.

## Ana Stack

- Backend Core: Rust
- Realtime Services: Go
- Frontend: React + TypeScript
- Desktop App: Tauri
- Database: PostgreSQL
- Cache: Redis
- Realtime: gRPC + WebSocket
- Queue: NATS veya Kafka
- Deploy: Docker + Docker Compose
- Reverse Proxy: Nginx
- Server OS: Ubuntu Server

## Mimari

```text
Desktop App -> Rust API -> Go Services -> PostgreSQL
```

Detayli not: `docs/architecture.md`

Database kurali: SQLite kullanilmaz. Lokal ana bilgisayarda da server ortaminda da PostgreSQL kullanilir. Detay: `docs/database-policy.md`

Lokal PostgreSQL notlari: `docs/local-postgres.md`

Ilk PDF sira takibi: `docs/first-pdf-progress.md`

Backup notlari: `docs/backup.md`

API endpoint listesi: `docs/api-endpoints.md`

Is kurallari: `docs/business-rules.md`

API ornekleri: `docs/api-examples.md`

## Klasorler

- `backend`: Rust API, auth, audit log, vehicle ERP core
- `realtime`: Go servisleri, GPS tracking, WebSocket gateway, TCP socket server
- `database`: PostgreSQL migration dosyalari
- `docker`: Docker yardimci dosyalari
- `nginx`: reverse proxy ayarlari
- `frontend`: React + TypeScript + Vite kurumsal ERP paneli
- `mobile`: Expo + React Native + TypeScript mobil uygulama
- `desktop`: ileride Tauri uygulamasi icin ayrildi

## Lokal Calistirma

Gerekli araclari kontrol etmek icin:

```powershell
.\scripts\check-local.ps1
```

Lokal PostgreSQL ve Rust backend:

```powershell
.\scripts\start-local-postgres.ps1
.\scripts\start-backend.ps1
.\scripts\start-frontend.ps1
.\scripts\smoke-backend.ps1
```

Tek komutluk lokal kontrol:

```powershell
.\scripts\check-all-local.ps1
```

Windows lokal backend derleme kontrolu:

```powershell
.\scripts\check-backend.ps1
```

Mobil uygulama:

```powershell
cd mobile
npm install
$env:EXPO_PUBLIC_API_BASE_URL="http://100.95.238.26:8080"
npm run start
```

Go realtime servisi:

```powershell
.\scripts\check-realtime.ps1
.\scripts\start-realtime.ps1
```

Tum sistem Docker ile:

```powershell
Copy-Item .env.example .env
docker compose up --build
```

## Gelistirme Sirasi

1. Rust API: auth, users, vehicles, logs
2. PostgreSQL migration ve seed akisi
3. Go realtime: GPS TCP socket ve WebSocket yayinlari
4. Redis/NATS entegrasyonlari
5. React + TypeScript frontend
6. Expo + React Native mobil uygulama
7. Tauri desktop uygulamasi

Backend health:

- `http://127.0.0.1:8080/health`

Frontend:

- Local: `http://127.0.0.1:5173`
- Tailscale ornek: `http://100.95.238.26:5173`
- Wi-Fi ornek: `http://192.168.1.106:5173`

Swagger / API test ekrani:

- `http://127.0.0.1:8080/docs`
- OpenAPI JSON: `http://127.0.0.1:8080/openapi.json`

Realtime health:

- `http://127.0.0.1:8090/health`
- Nginx uzerinden: `http://localhost/realtime/health`

Local not:

- Backend 2026-05-22 tarihinde local PostgreSQL uzerinde smoke testten gecirildi.
- Swagger UI 2026-05-24 tarihinde backend icine eklendi ve `/docs` uzerinden dogrulandi.
- Frontend 2026-06-01 tarihinde React + Vite olarak eklendi ve build dogrulamasindan gecti.
- Mobile 2026-06-04 tarihinde Expo + React Native ilk iskelet olarak eklendi.
- Network/Tailscale test notlari: `docs/network-access.md`
- Go runtime bu makinede henuz aktif degilse realtime check scripti bunu hizli sekilde bildirir.
- Docker bu makinede henuz aktif degilse compose dogrulamasi server/WSL/Docker kurulumundan sonra yapilir.

## Not

Onceki gecici Python/FastAPI denemesi bu stack karariyla devreden cikarildi. Yeni kaynak mimari Rust + Go + PostgreSQL uzerinden ilerler.
