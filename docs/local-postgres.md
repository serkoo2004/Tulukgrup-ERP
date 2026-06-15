# Local PostgreSQL

Bu proje lokal gelistirmede de PostgreSQL kullanir. SQLite kullanilmaz.

Bu bilgisayarda gecici lokal cluster:

```text
database/local-postgres-data
```

Lokal baglanti:

```text
postgres://postgres@127.0.0.1:5432/tuluklar_erp
```

Durum kontrolu:

```powershell
.\scripts\status-local-postgres.ps1
```

Backend baslatma:

```powershell
.\scripts\start-backend.ps1
```

Backend durdurma:

```powershell
.\scripts\stop-backend.ps1
```

Health kontrolu:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

Backend smoke test:

```powershell
.\scripts\smoke-backend.ps1
```

Tum lokal kontrol:

```powershell
.\scripts\check-all-local.ps1
```

Tablo kontrolu:

```powershell
& 'C:\Program Files\PostgreSQL\18\bin\psql.exe' -h 127.0.0.1 -p 5432 -U postgres -d tuluklar_erp -c "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';"
```

Lokal test admin:

```text
admin@tuluklar.local / Admin12345!
```

Server'a geciste uygulama kodu degismez. Sadece `.env` icindeki `DATABASE_URL` server PostgreSQL adresine cevrilir.

Baslatma:

```powershell
.\scripts\start-local-postgres.ps1
```

Durdurma:

```powershell
.\scripts\stop-local-postgres.ps1
```
