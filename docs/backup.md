# Backup

Backup sistemi PostgreSQL ve dosya storage alanini kapsar.

## Icerik

- PostgreSQL custom dump
- `STORAGE_ROOT` altindaki dosyalar
- Manifest dosyasi

## Lokal Backup

```powershell
$env:DATABASE_URL="postgres://erp_user:erp_password@localhost:5432/tuluklar_erp"
$env:STORAGE_ROOT=".\storage"
$env:BACKUP_DIR=".\backups"
.\scripts\backup-postgres.ps1
```

## Restore

```powershell
$env:DATABASE_URL="postgres://erp_user:erp_password@localhost:5432/tuluklar_erp"
.\scripts\restore-postgres.ps1 -DumpPath ".\backups\backup-YYYYMMDD-HHMMSS\postgres.dump"
```

## Server Notu

Ubuntu Server ortaminda ayni mantik `pg_dump`, `pg_restore` ve storage klasoru ile calisir. Docker Compose kullaniliyorsa scriptler host uzerinden PostgreSQL portuna veya container network adresine baglanacak sekilde calistirilir.
