# Tuluklar ERP Frontend

React + TypeScript + Vite tabanli kurumsal web panelidir.

## Local calistirma

```powershell
cd frontend
npm install
npm run dev
```

Adres:

```text
http://127.0.0.1:5173
```

Backend varsayilan adresi:

```text
http://127.0.0.1:8080
```

Gerekiyorsa `frontend/.env` icinde degistir:

```text
VITE_API_BASE_URL=http://127.0.0.1:8080
```

## Test kullanicisi

```text
admin@tuluklar.local
Admin12345!
```

## Ekranlar

- Dashboard
- Hesabim, sifre degistirme ve backend refresh token logout iptali
- Arac listesi ve arac profil detayi
- Arac ekleme formu
- Arac detayinda KM kaydi, KM dogrulama, kullanici atama ve atama cikisi
- Global arama
- Operasyon kayitlari, kayit duzenleme ve arsivleme
- Gorev, bakim, police, gider, hasar ve bildirim takip ekrani
- Takip modullerinde temel kayit olusturma formlari
- Takip modullerinde durum/guncelleme ve arsivleme aksiyonlari
- Bildirim olusturma, teslim durumu guncelleme ve okundu isaretleme
- Dosya yukleme ve dosya listesi
- Dosya filtreleme ve arsivleme
- Import job, staging, validasyon ve job durum guncelleme ekrani
- Import staging satirlari ve validasyon hata listesi
- XLSX rapor indirme
- Raporlarda arac/durum/tarih filtreleri
- AI provider, yetenek, oneri, prompt ve job izleme
- AI job durum/sonuc duzenleme, onay, gorev ve bildirim taslagi
- Backend rol listesiyle kullanici olusturma, duzenleme, pasife alma, ayar, sistem logu ve audit ekrani
- Manuel sistem logu olusturma
- Sirket/departman olusturma, duzenleme ve aktif/pasif yonetimi
- Sistem ayari olusturma ve duzenleme
- Route bazli lazy-load ile sayfa chunk ayrimi

Mobil uygulama ayri fazda ele alinacak; bu panel responsive web olarak hazirlandi.
