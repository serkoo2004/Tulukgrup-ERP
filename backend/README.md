# Rust API

ERP core backend bu klasordedir.

Baslangic modulleri:

- Auth: JWT access token + refresh token
- Users: admin seed ve role bilgisi
- Users API: kullanici listeleme, olusturma, rol listesi
- Organization: sirket ve departman yonetimi
- Vehicles: aktif/pasif/satilan arac akisi
- Assignments: arac kullanici gecmisi ve atama/birakma akisi
- Tracking: KM girisi, supheli KM tespiti, operasyon task'i
- Maintenances: bakim kaydi, listeleme, durum guncelleme
- Insurance Policies: trafik/kasko/imm/ferdi kaza poliçe kayitlari
- Expenses: yakit, sigorta, bakim, vergi, ceza, lastik ve servis giderleri
- Damages: hasar/kaza kaydi, ekspertiz/sigorta sureci ve maliyet takibi
- Tasks: gorev olusturma, listeleme, atanan gorevler, durum guncelleme
- Notifications: sistem/WhatsApp/email bildirim kayitlari ve teslim durumu
- Operations: arac kontrol, deger kaybi, yakit-KM, arac yikama ve sigorta teklif takipleri
- Files: module/entity bazli dosya upload ve DB path/reference kaydi
- Reports: yonetim ozeti, arac, bakim, police, gider, hasar ve operasyon XLSX exportlari
- Imports: import job, staging row, kolon mapping, validasyon ve hata kaydi altyapisi
- System Logs: sistem hata loglari listeleme ve kayit
- Settings: JSONB tabanli merkezi sistem ayarlari
- AI: analiz job kaydi, sonuc guncelleme ve insan onayi
- Security: CORS, body limit ve basit rate limiting
- Logs: audit log listeleme
- Database: PostgreSQL + SQLx migration

Calistirma:

```powershell
cd backend
cargo run
```

Gerekli environment degiskenleri kokteki `.env.example` icinde tanimli.
