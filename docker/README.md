# Docker

Docker Compose ana dosyasi kok dizindeki `docker-compose.yml` dosyasidir.

Bu klasor ileride Ubuntu Server deploy scriptleri, backup job tanimlari ve servis override dosyalari icin ayrildi.

## Guvenlik notu

- PostgreSQL, Redis ve NATS portlari compose icinde sadece `127.0.0.1` uzerinden publish edilir.
- Dis erisim Nginx reverse proxy uzerinden olur.
- Uygulama kodu lokal PC'den Ubuntu Server'a gecerken degismez; `.env` ve compose environment degerleri degisir.
- Production ortaminda `.env` icindeki JWT secret ve PostgreSQL sifreleri mutlaka degistirilir.

## Temel komutlar

```powershell
docker compose up --build
docker compose ps
docker compose logs -f backend
```

Backend health:

```text
http://localhost/health
```

Realtime health:

```text
http://localhost/realtime/health
```
