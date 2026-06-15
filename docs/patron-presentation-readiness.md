# Patron Sunumu Hazirlik ve Kabul Listesi

Bu liste, uygulama sunumunda "bunu da yapalim" sorusunu azaltmak icin ana modulleri, tamamlanan yetenekleri ve sunuma hazirlik kriterlerini takip eder.

## 1. Genel Sistem

Tamamlananlar:

- PostgreSQL tabanli merkezi veri mimarisi.
- Backend API uzerinden erisim; veritabani dogrudan disari acilmiyor.
- JWT login, refresh token ve rol bazli yetki altyapisi.
- Audit log ve sistem hata loglari.
- Swagger UI ve OpenAPI JSON.
- Docker/Docker Compose uyumlu servis yapisi.
- Lokal deneme icin Postgres ve backend/frontend start-stop scriptleri.

Sunumda gosterilecekler:

- Login.
- Dashboard.
- Swagger uzerinden API test edilebilirligi.
- Audit/system log kayitlari.

Kabul kriteri:

- Yetkisiz kullanici korumali endpointlere erisememeli.
- Kritik create/update/delete islemleri audit log uretmeli.
- Backend ve frontend build hatasiz gecmeli.

## 2. Arac ve Filo Yonetimi

Tamamlananlar:

- Arac kartlari.
- Arac profil ekrani.
- KM kayitlari ve supheli KM dogrulama.
- Bakim, police, gider, hasar, operasyon kayitlari.
- Dosya ekleri ve import altyapisi.

Sunumda gosterilecekler:

- Bir arac profili acilip KM, bakim, police, hasar ve gider ozetleri incelenir.
- Dosya ekleri ve raporlar gosterilir.

Kabul kriteri:

- Arac karti tek ekrandan operasyonel ozet verebilmeli.
- Eski ve yeni kayitlar filtrelenebilir olmalı.
- Eksik veri durumunda ekran bozulmamali.

## 3. Merkez Stok ve Envanter

Tamamlananlar:

- Urun karti.
- Kategori, alt kategori, marka, aciklama, aktif/pasif.
- Ana birim ve koli/paket carpan sistemi.
- Minimum, maksimum, kritik ve guvenlik stok seviyeleri.
- Lot takibi ve SKT takibi.
- Depo girisi, depo cikisi, subeye sevk, subeden iade, sayim duzeltmesi, hurda, arac kullanimi, manuel hareketler.
- Stok eksiye dusmeme korumasi.
- Lot-urun eslesme korumasi.
- Hareket iptali; hareket silinmez.
- Cok kalemli sevkiyat.
- Cok kalemli sayim.
- Sayim tamamlaninca stok duzeltme.
- Kritik stok ve tukenen stok alarmlari.
- Satin alma onerileri.
- Satin alma talebi ve onay miktari.
- Dashboard: stok degeri, kritik/tukenen stok, bekleyen sevkiyat/talep, SKT riski, son hareketler, son bildirimler.
- XLSX raporlar.
- Urun ve hareket filtreleri.

Sunumda gosterilecekler:

- Koli bazli giris: 5 koli = 5000 adet.
- Adet bazli cikis: 150 adet cikildiginda stok azalmasi.
- Lot secimli hareket.
- Kritik stok listesi.
- Sevkiyat olusturma ve onay.
- Sayim olusturma ve tamamlama.
- Satin alma talebi ve onay miktari.
- XLSX rapor indirme.

Kabul kriteri:

- Koli/adet donusumu kullaniciya acik gorunmeli.
- Lot takipli urunde yanlis lot ile hareket engellenmeli.
- Stok hicbir hareketle negatife dusmemeli.
- Hareket kaydi silinmemeli, sadece iptal edilebilmeli.
- Patron rapor acinca sayilar okunabilir ve Excel bicimli olmali.

## 4. WhatsApp Bildirimleri

Tamamlananlar:

- Sistem/WhatsApp/e-posta kanal alanlari.
- Notification kaydi.
- WhatsApp provider status endpointi.
- WhatsApp dispatch hazirlik endpointi.
- Recipient phone zorunlulugu.
- Provider payload, external message id, delivery status ve delivery error alanlari.
- WhatsApp webhook verify endpointi.
- WhatsApp webhook payload loglama.
- Env ve Docker ayarlari.

Sunumda gosterilecekler:

- Provider status ekrani/API cevabi.
- Ayar eksikken dispatch sonucunun failed ve delivery_error uretmesi.
- Ayar girilince queued payload uretilmesi.

Kabul kriteri:

- Access token response icinde hic donmemeli.
- Telefon numarasi olmadan WhatsApp dispatch yapilmamali.
- Provider eksikse sistem hata vermek yerine kayitli ve denetlenebilir hata durumu olusturmali.

## 5. AI Entegrasyonu

Tamamlananlar:

- AI provider status.
- Local/dıs model hazirlik sozlesmesi.
- AI capabilities.
- Prompt template sozlesmeleri.
- Backend verisinden AI onerileri.
- Arac context paketi.
- AI job olusturma.
- AI sonuc/status guncelleme.
- AI job onaylama.
- AI sonucundan gorev taslagi.
- AI sonucundan bildirim taslagi.
- Frontend AI yonetim ekrani.
- AI baglanti hazirligi paneli.
- IT destek talebi AI context paketi.
- IT destek talebi AI analiz job'u.
- IT destek triage, SLA risk, kok neden ve cevap taslagi prompt sozlesmeleri.

Sunumda gosterilecekler:

- AI ekraninda provider hazirligi.
- AI onerileri.
- Prompt sozlesmeleri.
- Job olusturma/guncelleme/onay.
- Gorev/bildirim taslagi uretimi.
- IT destek ekranindan AI analiz job'u baslatma.

Kabul kriteri:

- AI kritik islemi otomatik uygulamamali.
- Insan onayi gereken analizler acikca isaretlenmeli.
- AI context backend tarafindan uretilmeli.
- IT destek AI context'i ticket, SLA, event ve reporter verisini kontrollu paketlemeli.
- Local AI ve dis model ayar eksikleri ekranda gorunmeli.

## 6. IT Destek ve Sikayet Talep Akisi

Tamamlananlar:

- IT destek talebi veri modeli.
- Talep numarasi, baslik, aciklama, kategori, oncelik, durum, kaynak kanal.
- Kaynak kanallar: web, mobil, WhatsApp, sistem.
- Reporter ad/telefon/kullanici bilgisi.
- Atanan kullanici ve atanan birim.
- Cozum notu, resolved/closed zamanlari.
- SLA ilk cevap hedefi ve cozum hedefi.
- SLA durumu: zamaninda, yaklasan, ilk cevap geciken, cozum geciken.
- Eskalasyon seviyesi.
- Cozum sonrasi memnuniyet puani ve notu.
- Talep event gecmisi.
- WhatsApp webhook mesajindan otomatik IT destek talebi olusturma.
- Ayni WhatsApp mesaj id'si tekrar gelirse cift ticket olusmasini engelleyen indeks.
- IT Destek frontend ekrani.
- Dashboard IT destek KPI'lari.
- IT destek ekrani genel KPI summary endpoint'i.
- Global aramadan destek taleplerini bulma.
- IT destek talepleri XLSX raporu.
- SLA gecikme uyarilari ve XLSX SLA kolonlari.
- IT destek bilgi bankasi.
- Cozulen destek talebinden bilgi bankasi kaydi olusturma.
- IT bilgi bankasi global arama ve XLSX raporu.

Sunumda gosterilecekler:

- Web uzerinden destek talebi acma.
- Talebi IT personeline veya birime atama.
- Durumu `in_progress`, `waiting_user`, `resolved`, `closed` olarak ilerletme.
- Cozum notu girme.
- Talep gecmisini gorme.
- WhatsApp webhook test payload'i ile otomatik destek talebi olusmasi.
- Dashboard ve XLSX raporda acik/kritik/WhatsApp kaynakli destek taleplerini gorme.
- Destek ekraninda tablo filtrelenirken ust KPI'lar genel toplam olarak dogru kalir.
- Geciken SLA taleplerini filtreleme ve hangi hedefin kacirildigini gorme.
- Tekrar eden sorunlar icin bilgi bankasinda cozum arama.
- Bilgi bankasi kayitlarini rapor olarak indirme.

Kabul kriteri:

- Mobil ve WhatsApp ayni destek ticket akisini kullanmali.
- Talep silinmeden durum/event gecmisiyle takip edilmeli.
- WhatsApp'tan gelen bos veya text olmayan event ticket acmamali.
- Tekrar gelen ayni WhatsApp mesaj id'si cift talep uretmemeli.
- IT cozum notu ve zaman bilgisi raporlanabilir olmalı.
- Talep arama ve raporlama akisi ana ERP ekranlarindan kopuk olmamali.
- SLA hedefleri kayit acilisinda otomatik hesaplanmali ve rapora cikmali.
- Cozum bilgisi tek ticket icinde kalmamali; tekrar kullanilabilir bilgi bankasina aktarilabilmeli.

## 7. Raporlama

Tamamlananlar:

- Stok urun listesi XLSX.
- Stok hareketleri XLSX.
- Kritik stok XLSX.
- IT destek talepleri XLSX.
- Excel sayi formatlari okunabilir hale getirildi.

Sunumda gosterilecekler:

- XLSX dosyasi indirilip Excel'de acilir.
- Rapor baslik, kolon ve sayi bicimleri kontrol edilir.

Kabul kriteri:

- CSV zorunlulugu yok; patron tarafina XLSX cikti verilir.
- Sayilar gereksiz ondalik sifirlarla gelmemeli.

## 8. Mobil ve Gelecek Surum

Hazirlananlar:

- Mobil uygulama altyapisi ayrica baslatilabilir.
- QR/sayim/stok hareketi mobilde ileride genisletilecek sekilde ayrildi.

Sunumda soylenmeyecek detay:

- Mobil tamamlanmis gibi sunulmayacak; "sonraki fazda saha/depo mobil uygulamasi" olarak konumlanacak.

## 9. Sunum Oncesi Teknik Kontrol

Calistirilacak kontroller:

- `cargo check`
- `npm run build`
- `scripts/smoke-backend.ps1`
- `scripts/smoke-inventory.ps1`
- `scripts/smoke-reports-xlsx.ps1`

Sunum oncesi riskler:

- Gercek WhatsApp gonderimi icin Meta Business API bilgileri gerekir.
- Gercek AI sonucu icin local model veya dis model worker baglanmalidir.
- Mobil depo uygulamasi sonraki fazdir.
- Go runtime yoksa realtime gateway local testleri tamamlanamaz.

Net sunum cumlesi:

Bu sistem su anda merkezi PostgreSQL, API, yetki, audit, filo, operasyon, stok, rapor, WhatsApp hazirlik ve AI job sozlesmesiyle calisan kurumsal ERP temelidir. WhatsApp ve AI servis bilgileri girildiginde dis entegrasyonlar devreye alinabilir.
