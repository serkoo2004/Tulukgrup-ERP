# Tuluklar ERP Mobile

Expo + React Native + TypeScript mobil uygulama iskeleti.

## Kapsam

- Giriş ekranı
- Ana dashboard özeti
- Merkez stok dashboard
- Mobil ürün kartı oluşturma
- Kritik stok listesi
- IT destek talebi oluşturma
- IT destek durum/SLA takibi
- Araç servis/arıza bildirimi
- Foto/video seçip destek talebine ekleme
- Ürün sipariş talebi oluşturma
- Araç KM bildirimi
- KM fotoğrafı yükleyip OCR/AI doğrulama işi oluşturma
- Araç hızlı listesi
- Hesap / çıkış ekranı
- Backend `auth/me` profilinden mobil yetki okuma
- Yetkiye göre tab ve aksiyon gösterme/gizleme

## Mobil Yetkiler

Mobil uygulama girişten sonra `GET /api/v1/auth/me` çağrısı yapar ve `mobile_permissions` listesini okur.

Yönetim panelinde `Kullanıcılar > Düzenle > Mobil Uygulama Yetkileri` alanından şu yetkiler atanır:

- `dashboard`: mobil panel
- `inventory_view`: stok ekranı ve stok hareketi
- `inventory_order`: ürün sipariş talebi
- `vehicle_view`: araç listesi
- `vehicle_fault`: araç servis/arıza bildirimi
- `km_log`: KM girişi
- `support_ticket`: IT destek talebi
- `media_upload`: foto/video yükleme

Şirket/departman boş bırakılırsa yetki globaldir. Dolu bırakılırsa kullanıcının şirket/departman kapsamı ile çalışır.

## API Adresi

Mobil cihaz bilgisayardaki backend'e `127.0.0.1` ile erişemez. Test ortamına göre API adresi verilmelidir.
Giriş ekranında `API adresi` alanı vardır; Tailscale, Wi-Fi veya emulator adresi buradan değiştirilebilir. Ortam değişkeni verilirse alan başlangıçta o değerle gelir.

Tailscale örneği:

```powershell
$env:EXPO_PUBLIC_API_BASE_URL="http://100.95.238.26:8080"
npm run start
```

Wi-Fi örneği:

```powershell
$env:EXPO_PUBLIC_API_BASE_URL="http://192.168.1.105:8080"
npm run start
```

Android emulator için:

```powershell
$env:EXPO_PUBLIC_API_BASE_URL="http://10.0.2.2:8080"
npm run android
```

## Kurulum

```powershell
cd mobile
npm install
npm run start
```

Expo Go ile telefondan QR okutularak denenebilir.

Tip kontrol:

```powershell
npm run typecheck
```

## Not

Bu mobil faz backend API'ye direkt bağlanan operasyon uygulamasıdır. Stok, ürün siparişi, IT destek, araç arıza bildirimi, medya eki, KM bildirimi, KM fotoğrafı kanıtı ve mobil yetki kontrolü canlı API ile çalışır. KM fotoğrafı seçilirse kayıt `entry_type=ocr` olarak açılır ve backend `ocr_verification` AI job oluşturur. Destek ekran görüntüsü/video/log/dokümanları `support_media_analysis`, araç/hasar görselleri `vehicle_media_inspection` AI job akışına düşer. Local vision model bağlandığında bu işler fotoğraf/video yorumlama ve KM okuma için işlenecektir; kritik sonuçlar insan onayı bekler.
