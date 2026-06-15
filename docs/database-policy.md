# Database Policy

Bu projede SQLite kullanilmaz.

## Kural

- Gelistirme ortaminda PostgreSQL kullanilir.
- Ana bilgisayar gecici ortam olarak kullanilacaksa PostgreSQL yine lokal servis veya Docker container olarak calisir.
- Server ortaminda ayni uygulama `DATABASE_URL` degistirilerek Ubuntu Server uzerindeki PostgreSQL'e baglanir.
- Database disariya direkt acilmaz; tum erisim Rust API uzerinden yapilir.
- Migration yonetimi Rust API icindeki SQLx migration sistemiyle calisir.
- SQLx sadece PostgreSQL feature'i ile kullanilir. `cargo tree -i sqlx-sqlite` ve `cargo tree -i libsqlite3-sys` aktif bagimlilik olmadigini dogrulamalidir.
- `Cargo.lock` icinde gorulebilen SQLite isimleri SQLx paket metadata/opisyonel kayitlaridir; aktif runtime veya build bagimliligi degildir.

## Ortamlar

Lokal ana bilgisayar:

```text
DATABASE_URL=postgres://erp_user:erp_password@localhost:5432/tuluklar_erp
```

Docker Compose:

```text
DATABASE_URL=postgres://erp_user:erp_password@postgres:5432/tuluklar_erp
```

Ubuntu Server:

```text
DATABASE_URL=postgres://<user>:<password>@<postgres-host>:5432/<database>
```
