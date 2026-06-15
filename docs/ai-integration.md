# AI Integration Plan

Bu dosya uygulama icindeki yapay zeka kullanim noktalarini ve backend sozlesmesini tanimlar.

## Ana Ilke

AI kritik isleri otomatik uygulamaz. Sistem AI sonucunu job olarak kaydeder, sonucu denetlenebilir tutar ve finansal/status degisikligi gibi kritik aksiyonlarda insan onayi ister.

## Backend Endpointleri

- `GET /api/v1/ai/capabilities`
  - Sistemin destekledigi AI kullanim alanlarini dondurur.
- `GET /api/v1/ai/provider`
  - AI provider/model ayarinin yapilip yapilmadigini dondurur.
- `GET /api/v1/ai/prompt-templates`
  - Her analiz tipi icin prompt amaci, gerekli context ve beklenen cikti sozlesmesini dondurur.
- `GET /api/v1/ai/recommendations`
  - Mevcut ERP verisinden AI calistirilmasi gereken alanlari onerir.
- `GET /api/v1/ai/vehicles/:id/context`
  - Bir arac icin AI'a gidecek guvenli veri paketini dondurur.
- `POST /api/v1/ai/vehicles/:id/analyze`
  - Arac baglamindan AI job olusturur.
- `GET /api/v1/ai/support/tickets/:id/context`
  - Destek talebi, SLA, reporter, atama, memnuniyet ve event gecmisinden guvenli AI veri paketi dondurur.
- `POST /api/v1/ai/support/tickets/:id/analyze`
  - Destek talebi baglamindan AI job olusturur.
- `GET /api/v1/ai/jobs`
  - AI job listesini verir.
- `POST /api/v1/ai/jobs`
  - Manuel AI job olusturur.
- `PATCH /api/v1/ai/jobs/:id`
  - AI sonucu/status gunceller.
- `POST /api/v1/ai/jobs/:id/approve`
  - Kritik AI sonucunu yonetici onayina alir.
- `GET /api/v1/ai/jobs/:id/task-draft`
  - AI sonucundan gorev taslagi uretir; otomatik gorev acmaz.
- `GET /api/v1/ai/jobs/:id/notification-draft`
  - AI sonucundan bildirim taslagi uretir; otomatik gondermez.

## Provider Ayarlari

`.env` uzerindeki alanlar:

- `AI_PROVIDER`
- `AI_MODEL`
- `AI_API_KEY`
- `AI_BASE_URL`

Su an `AI_PROVIDER=disabled` oldugunda sistem dis model cagirmadan job/context/taslak sozlesmesini calistirir. OpenAI, Gemini, Claude veya lokal model baglandiginda worker bu ayarlari okuyup job'u isler. Lokal kurulum icin `AI_PROVIDER=ollama`, vision destekli bir `AI_MODEL` ve `AI_BASE_URL=http://127.0.0.1:11434` kullanilabilir.

## Kullanim Alanlari

1. Arac risk ozeti
   - Kaynaklar: arac, KM, bakim, police, hasar, gider, operasyon kayitlari.
   - Analiz tipi: `vehicle_risk_summary`.
   - Kritik kararlar insan onayi ister.

2. Dashboard yorumlama
   - Kaynaklar: dashboard metrikleri, audit hareketleri.
   - Analiz tipi: `dashboard_insight`.
   - Yonetim panelinde aksiyon onceligi uretir.

3. Police yenileme asistanı
   - Kaynaklar: police, teklif, bitis tarihi, tutar.
   - Analiz tipi: `policy_renewal_advice`.
   - Teklif onayi insan onayi ister.

4. Hasar dosya asistanı
   - Kaynaklar: hasar, eksper, fotograf/dosya, sigorta dosya no.
   - Analiz tipi: `damage_claim_assistant`.
   - Dosya sureci onerisi uretir.

5. Deger kaybi / hak mahrumiyeti asistanı
   - Kaynaklar: kaza tarihi, tramer, talep, gelen tutar, notlar.
   - Analiz tipi: `value_loss_assistant`.
   - Dava/surec onerisini insan denetimine birakir.

6. Yakit-KM analiz asistanı
   - Kaynaklar: yakit limiti, odenen yakit, kalan limit, gidilen KM, KM log.
   - Analiz tipi: `fuel_analysis`.
   - Anomali ve verimlilik uyarisi uretir.

7. Excel import kolon eslestirme
   - Kaynaklar: import job, satir hatalari, kolon adlari.
   - Analiz tipi: `import_mapping_assistant`.
   - Sistem alanlarina kolon eslestirme onerisi uretir.

8. Bildirim metni uretimi
   - Kaynaklar: gorev, police, hasar, bakim.
   - Analiz tipi: `message_generation`.
   - WhatsApp/e-posta/sistem mesaj taslagi uretir.

9. Rapor anlatimi
   - Kaynaklar: dashboard ve XLSX rapor ozetleri.
   - Analiz tipi: `report_narrative`.
   - Patron/yonetim icin kisa okunabilir rapor metni uretir.

10. IT destek talep siniflandirma
   - Kaynaklar: destek talebi, SLA, olay gecmisi, reporter, atama.
   - Analiz tipi: `support_ticket_triage`.
   - Kategori, oncelik, atama ve ilk aksiyon onerisi uretir; otomatik uygulamaz.

11. IT destek SLA risk analizi
   - Kaynaklar: SLA hedefleri, ilk cevap, cozum hedefi, durum ve olay gecmisi.
   - Analiz tipi: `support_sla_risk`.
   - Gecikme riski ve musteri etkisi uretir.

12. IT destek kok neden ozeti
   - Kaynaklar: destek talebi, event gecmisi, cozum notu ve memnuniyet.
   - Analiz tipi: `support_root_cause_summary`.
   - Tekrar eden sorun ve kalici iyilestirme notu hazirlar.

13. Destek cevap taslagi
   - Kaynaklar: destek talebi, cozum notu, kaynak kanal, reporter.
   - Analiz tipi: `support_reply_draft`.
   - WhatsApp/mobil cevap metni taslagi uretir; insan onayi gerekir.

14. Mobil KM fotograf okuma
   - Kaynaklar: mobil KM kaydi, arac, yuklenen `km_photo`, onceki KM.
   - Analiz tipi: `ocr_verification`.
   - Fotograftan KM okumayi, girilen KM ile farki ve kanit seviyesini uretir; dogrulamayi otomatik uygulamaz.

15. Mobil fotoğraf/video yorumlama
   - Kaynaklar: mobil destek/servis bildirimi, dosya, mime type, kullanici notu, arac baglami.
   - Analiz tipi: `mobile_media_interpretation`.
   - Foto/video/dokumandan ariza, hasar, ekran hatasi veya eksik bilgi onerisi uretir.

16. Arac gorsel inceleme
   - Kaynaklar: arac fotografi, hasar fotografi, servis/bakim baglami.
   - Analiz tipi: `vehicle_media_inspection`.
   - Lastik, hasar, uyari lambasi, parca veya servis ihtiyaci icin on inceleme notu hazirlar.

17. Destek gorsel analizi
   - Kaynaklar: IT destek talebi, ekran goruntusu, video, log, dokuman ve SLA baglami.
   - Analiz tipi: `support_media_analysis`.
   - Kategori, oncelik, muhtemel neden, ilk aksiyon ve cevap taslagi uretir.

## Entegrasyon Mimarisi

1. Frontend AI gerektiren ekranda context veya recommendation endpointini cagirir.
2. Backend guvenli veri paketini hazirlar.
3. Frontend `analyze` veya `jobs` endpointi ile AI job olusturur.
4. Worker/AI servis job'u isler ve `PATCH /ai/jobs/:id` ile sonuc yazar.
5. Kritik sonuc gerekiyorsa manager/admin `approve` endpointi ile onaylar.
6. Gerekirse `task-draft` veya `notification-draft` endpointlerinden taslak uretilir.
7. Audit log tum create/update/onay adimlarini kaydeder.

## Guvenlik Kurallari

- AI direkt veritabanina kritik degisiklik uygulamaz.
- AI sonucunda insan onayi gerektiren alanlar `requires_human_approval=true` ile tutulur.
- AI'a giden context backend tarafinda olusturulur; frontend serbest veri birlestirme yapmaz.
- Hassas dosya/veri gonderimi sonraki model entegrasyonunda ayrica maskeleme katmanindan gecirilir.
- Her AI job audit log uzerinden izlenebilir.
