# Merkez Stok ve Envanter Modulu

Bu modul merkez depo stoklarini, sarf urunlerini, arac bakim malzemelerini ve sube sevkiyatlarini backend API uzerinden yonetmek icin eklendi.

## Tamamlanan Backend Kapsami

- Urun karti: ad, kategori, alt kategori, marka, aciklama, aktif/pasif. Urun kodu sistem icinde otomatik uretilir; barkod kullanicidan istenmez.
- Birim yonetimi: adet, koli, paket, kutu, kg, litre, metre.
- Koli/coklu birim: ana birim, paket birimi ve paket carpani.
- Stok alanlari: mevcut, kullanilabilir, rezerve, minimum, maksimum, kritik, guvenlik stoku, birim maliyet.
- Stok hareketleri: depo girisi, depo cikisi, subeye sevk, subeden iade, sayim duzeltmesi, hurda cikisi, arac kullanimi, manuel giris/cikis.
- Hareket guvenligi: hareketler silinmez; sadece iptal edilir ve ters stok hareketi olusturulur.
- Kritik stok: sari/turuncu/kirmizi alarm listesi ve sistem bildirimi.
- Lot ve SKT: lot numarasi, uretim tarihi, son kullanma tarihi, tedarikci.
- Sevkiyat: taslak sevkiyat olusturma, onaylama ve onayla birlikte stoktan dusme.
- Sayim: tam/kismi/kategori sayimi icin sistem stok, sayim stok, fark ve fark nedeni.
- Satin alma onerisi: son 3/6/12 ay tuketim, ortalama aylik tuketim, tahmini tukenme tarihi ve onerilen siparis miktari.
- Satin alma talebi: oneriden veya manuel olarak talep olusturma, onay/siparis/teslim/iptal durum takibi.
- Dashboard: toplam urun, stok degeri, kritik/tukenen stok, bekleyen sevkiyat, bekleyen satin alma, 30 gun SKT riski.
- Dosya yonetimi: stok urunlerine fatura, teknik dokuman, garanti belgesi, urun gorseli ve kullanim talimati eklenebilir.
- Raporlar: stok urun listesi, stok hareketleri ve kritik stok XLSX raporlari.
- Swagger: stok endpointleri `/docs` ve `/openapi.json` icinde test edilebilir.

## Frontend Kapsami

- Sol menuye `Stok` ekrani eklendi.
- Dashboard kartlari, urun listesi/formu, hareket girisi/iptali, lot/SKT, sevkiyat, sayim, kritik stok ve satin alma onerileri sekmeleri eklendi.
- Dosyalar ekraninda `Stok Urunu` modulu ve stok urun dosya tipleri eklendi.
- Raporlar ekranina stok XLSX indirme kartlari eklendi.
- Ana dashboard'a stok degeri, kritik stok, tukenmis stok, SKT riski ve bekleyen sevkiyat KPI'lari baglandi.
- Urun karti duzenleme/arsivleme ve satin alma talep durum aksiyonlari eklendi.

## Dogrulama

Calistirilan kontroller:

- `cargo check`
- `npm run build`
- `scripts/smoke-inventory.ps1`
- `scripts/smoke-reports-xlsx.ps1`

Smoke test senaryosu:

- Urun karti olusturuldu.
- Lot olusturuldu.
- 5 koli giris yapildi; 1 koli = 1000 adet ile stok 5000 adet oldu.
- 150 adet cikis yapildi; stok 4850 adet oldu.
- Sayim sonucu 4800 adet olarak tamamlandi; negatif fark dogru sekilde stoktan dusuldu.
- Stok dashboard, kritik stok, satin alma onerisi ve XLSX raporlari dogrulandi.

## Sonraki Faz Notlari

- WhatsApp Business API gercek servis anahtarlariyla baglaninca kritik stok, stok tukendi, sayim farki, sevkiyat ve satin alma bildirimleri dis kanala gonderilecek.
- AI stok tahmini icin mevcut tuketim verisi backend tarafinda hazir; dis model entegrasyonu acilinca tahmin ve tedarikci analizi genisletilecek.
- Mobil depo uygulamasi, kamera ile urun tanima, RFID, e-fatura ve Logo/Netsis entegrasyonlari sonraki faza birakildi.
