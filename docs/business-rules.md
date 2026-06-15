# Business Rules

## Database

1. SQLite kullanilmaz.
2. Lokal gelistirme ve server ortami PostgreSQL kullanir.
3. Silme islemleri soft delete ile yapilir.
4. Kritik islemler audit log'a yazilir.
5. Dosya icerikleri DB icinde tutulmaz; DB sadece path/reference tutar.
6. Dosya upload isleminde hedef module/entity kaydi PostgreSQL'de var olmak zorundadir.
7. Dosya tipi hedef module icin izinli degilse upload reddedilir.

## Authorization

1. `admin` tum modullere erisir.
2. `manager` operasyonel kayitlari ve raporlari gorur.
3. `operation` arac, KM, bakim, police, gorev ve notification akisini yonetir.
4. `accounting` rapor, police, dosya ve muhasebe odakli kayitlara erisir.
5. `user` sadece kendi atanan arac ve gorev/bildirim kayitlarini gorebilir.
6. Kullanici rol/profil guncellemesi sadece `admin` tarafindan yapilir.
7. Kullanici pasiflestirilince acik refresh token kayitlari iptal edilir.
8. Aktif oturumdaki admin kendi hesabini soft delete yapamaz.
9. Logout islemi ilgili refresh token kaydini iptal eder.
10. Sifre degistirme mevcut sifre dogrulamasi ister ve kullanicinin tum refresh tokenlarini iptal eder.

## Vehicles

1. Plaka benzersizdir.
2. Arac sirket, departman ve kullanici ile iliskilenebilir.
3. Arac satildiginda silinmez.
4. Satis isleminde arac `sold` durumuna alinir.
5. Satis isleminde aktif gorevler `cancelled` yapilir.
6. Satilan veya pasif araca yeni KM, bakim ve police kaydi acilmaz.

## Vehicle Assignments

1. Arac kullanici gecmisi `vehicle_assignments` tablosunda tutulur.
2. Yeni atama acildiginda ayni aracin onceki acik atamasi kapatilir.
3. Arac uzerindeki guncel `user_id` yeni kullaniciya cekilir.
4. Atama birakildiginda guncel kullanici referansi bosaltilir.
5. Satilan veya pasif araca yeni kullanici atanamaz.

## Organization

1. Sirket adi benzersizdir.
2. Departman adi ayni sirket icinde benzersizdir.
3. Sirket ve departman kayitlari aktiflik mantigiyla yonetilir.
4. Arac ve kullanici kayitlari sirket/departman referanslarini kullanir.

## KM Tracking

1. KM negatif olamaz.
2. Yeni KM onceki KM'den dusukse kayit `suspicious` olur.
3. Yeni KM onceki KM'den 5000 km'den fazla yuksekse kayit `suspicious` olur.
4. Supheli KM kaydi operasyon gorevi olusturur.
5. Operasyon/yetkili kullanici KM kaydini `verified` veya `rejected` yapabilir.

## Maintenance

1. Bakim kaydi sadece aktif araclara acilir.
2. Durumlar: `planned`, `scheduled`, `completed`, `cancelled`.
3. Bakim maliyeti rapor ve satilan arac gecmisinde korunur.
4. Bakim kaydi silinmez; arsivleme soft delete ile yapilir.

## Insurance Policies

1. Police kaydi sadece aktif araclara acilir.
2. Police tipleri: `trafik`, `kasko`, `imm`, `ferdi_kaza`.
3. Yenileme durumlari: `active`, `approaching`, `renewing`, `ended`.
4. Police PDF dosyasi DB'de degil dosya storage alaninda saklanir.
5. Police kaydi silinmez; arsivleme soft delete ile yapilir.

## Expenses

1. Gider kaydi sadece aktif araclara acilir.
2. Gider tipleri: `fuel`, `insurance`, `maintenance`, `tax`, `fine`, `tire`, `service`.
3. Odeme durumlari: `pending`, `paid`, `cancelled`, `overdue`.
4. Fatura, dekont ve gider belgeleri DB'de tutulmaz; `file_documents` uzerinden path/reference olarak tutulur.
5. `expenses.invoice_file` sadece eski sistemden gelen veya harici path/reference icin kullanilir; ana belge kaydi dosya moduludur.
6. Gider detay cevabi belge sayilarini `file_summary` icinde dondurur.
7. Satilan arac gider gecmisi korunur ve satilan arac detay ozetinde kullanilir.
8. Satilan araca yeni gider kaydi acilmaz; gecmis kayitlar raporlanir.
9. Gider kaydi silinmez; arsivleme soft delete ile yapilir.

## Damages

1. Hasar/kaza kaydi sadece aktif araclara acilir.
2. Durumlar: `open`, `expertise`, `insurance`, `repaired`, `closed`, `cancelled`.
3. Hasar raporu, ekspertiz raporu ve hasar fotograflari dosya modulunde tutulur.
4. Satilan arac hasar gecmisi korunur ve satilan arac detay ozetinde kullanilir.
5. Satilan veya pasif araca yeni hasar kaydi acilmaz; gecmis kayitlar raporlanir.
6. Hasar kaydi silinmez; arsivleme soft delete ile yapilir.

## Tasks

1. Gorevler arac, kullanici veya departman ile iliskilenebilir.
2. Oncelikler: `low`, `medium`, `high`, `critical`.
3. Durumlar: `open`, `in_progress`, `waiting`, `completed`, `cancelled`.
4. Kullanici kendi atanan gorevlerini gorebilir.
5. Yetkili roller gorevin durumunu, atanan kullanicisini, departmanini, onceligini, tarihini ve aciklamasini guncelleyebilir.

## Notifications

1. Kanallar: `system`, `whatsapp`, `email`.
2. Teslim durumlari: `pending`, `queued`, `sent`, `failed`, `cancelled`.
3. WhatsApp provider durumu `/notifications/provider-status` ile kontrol edilir.
4. WhatsApp bildirimi dispatch edilmeden once `recipient_phone` zorunludur.
5. `WHATSAPP_ENABLED`, `WHATSAPP_PHONE_NUMBER_ID` ve `WHATSAPP_ACCESS_TOKEN` girilmeden dispatch istegi kaydi `failed` durumuna alir ve hata sebebini saklar.
6. Provider ayarlari tamamlandiginda dispatch islemi Meta Graph API'ye text mesaj gonderir; token sistem cevabinda veya provider payload icinde saklanmaz.
6. Provider payload, dis mesaj id'si ve teslim hata metni bildirim kaydinda tutulur.
7. Kullanici bildirimi okudugunda `read_at` alanina ilk okuma zamani yazilir.
8. Kullanici kendi bildirimi disindaki bildirimi okuyamaz; yetkili roller operasyonel takip icin isaretleyebilir.

## Import / Export

1. Import dogrudan DB'ye basilmadan once job olarak kaydedilir.
2. Kolon mapping JSON olarak tutulur.
3. Hatali satirlar `import_errors` tablosunda raporlanir.
4. Rapor exportlari Excel `.xlsx` olarak saglanir.
5. Yonetim raporlari sadece ham tablo degil; baslik, KPI, filtre, renkli durum, yazdirma duzeni ve takip okunabilirligi ile uretilir.
6. `management.xlsx` raporu arac, gorev, police, bakim, gider ve hasar durumlarini tek yonetim ozeti olarak verir.
7. Import satirlari once `import_rows` staging tablosuna alinir.
8. Staging satirlari hedef module gore dogrulanmadan ana tablolara aktarilmaz.
9. Hatasiz job `ready`, hatali job `validating` durumunda kalir.
10. Tek import job icin ilk asama satir limiti 5000 olarak sinirlandirilir.

## AI

1. AI sonuclari kritik islemleri otomatik uygulamaz.
2. Kritik analizler insan onayi alanlariyla tutulur.
3. AI job kaydi olmadan model sonucu sisteme islenmez.
4. AI context paketleri backend tarafinda uretilir.
5. Desteklenen analiz tipleri: OCR dogrulama, maliyet analizi, bakim tahmini, anomali, yakit analizi, mesaj uretimi, arac risk ozeti, dashboard yorumu, police yenileme onerisi, hasar dosya asistanligi, deger kaybi asistanligi, import kolon eslestirme ve rapor anlatimi.
6. AI onerileri `GET /api/v1/ai/recommendations` ile operasyon verisinden uretilir.
7. Arac bazli AI analizleri `GET /api/v1/ai/vehicles/:id/context` ve `POST /api/v1/ai/vehicles/:id/analyze` akisi ile calisir.
8. AI sonucundan gorev veya bildirim sadece taslak olarak uretilir; otomatik gorev acma veya bildirim gonderme yapilmaz.
9. Dis model entegrasyonu `AI_PROVIDER`, `AI_MODEL`, `AI_API_KEY` ayarlariyla worker katmaninda calisir.

## Settings

1. Sistem ayarlari `system_settings` tablosunda JSONB olarak tutulur.
2. Ayar anahtarlari kucuk harf, sayi, nokta, alt cizgi ve tire ile sinirlandirilir.
3. Ayar yazma yetkisi sadece `admin` rolundedir.
4. `manager` ayarlari okuyabilir ama degistiremez.
