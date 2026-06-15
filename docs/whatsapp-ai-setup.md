# WhatsApp ve AI Baglanti Hazirligi

Bu dokuman, WhatsApp Business API ve AI model entegrasyonunun hangi ayarlarla devreye alinacagini tanimlar.

## WhatsApp Business API

Backend hazirlik durumu:

- Provider durumu: `GET /api/v1/notifications/provider-status`
- Bildirim olusturma: `POST /api/v1/notifications`
- Bildirim dispatch: `POST /api/v1/notifications/:id/dispatch`
- Webhook dogrulama: `GET /api/v1/notifications/whatsapp/webhook`
- Webhook payload alma: `POST /api/v1/notifications/whatsapp/webhook`

Gerekli ortam degiskenleri:

- `WHATSAPP_ENABLED=true`
- `WHATSAPP_BUSINESS_API_URL=https://graph.facebook.com/v20.0`
- `WHATSAPP_PHONE_NUMBER_ID`
- `WHATSAPP_ACCESS_TOKEN`
- `WHATSAPP_TEMPLATE_LANGUAGE=tr`
- `WHATSAPP_WEBHOOK_VERIFY_TOKEN`
- `WHATSAPP_APP_SECRET`
- `WHATSAPP_TIMEOUT_SECONDS=20`

Mevcut guvenlik kurali:

- `recipient_phone` yoksa WhatsApp dispatch yapilmaz.
- Provider ayarlari eksikse bildirim silinmez; `failed` durumuna alinir ve `delivery_error` yazilir.
- Gercek access token hicbir provider-status cevabinda geri donmez.
- Webhook payloadlari `system_error_logs` tablosuna denetlenebilir sekilde kaydedilir.
- `WHATSAPP_APP_SECRET` girildiginde webhook POST isteklerinde `X-Hub-Signature-256` HMAC-SHA256 imzasi dogrulanir.
- Provider ayarlari tamamlandiginda `POST /notifications/:id/dispatch` Meta Graph API `/{phone_number_id}/messages` endpointine text mesaj gonderir.
- Access token response veya provider payload icinde saklanmaz.

Sonraki baglama adimi:

1. Meta Business panelinde uygulama ve WhatsApp Business API aktif edilir.
2. Phone Number ID ve Access Token alinir.
3. `.env` dosyasina WhatsApp degiskenleri girilir.
4. Server public URL hazir olunca webhook callback URL olarak `/api/v1/notifications/whatsapp/webhook` girilir.
5. `WHATSAPP_WEBHOOK_VERIFY_TOKEN` Meta panelindeki verify token ile ayni verilir.
6. `WHATSAPP_APP_SECRET` girilerek webhook imza dogrulamasi aktif edilir.
7. Test bildirimi olusturulur ve dispatch edilir. Test numarasinda alici telefon Meta panelindeki allowed recipient listesinde olmalidir.

## AI Provider

Backend hazirlik durumu:

- Provider durumu: `GET /api/v1/ai/provider`
- Kullanim alanlari: `GET /api/v1/ai/capabilities`
- Prompt sozlesmeleri: `GET /api/v1/ai/prompt-templates`
- Veri bazli oneriler: `GET /api/v1/ai/recommendations`
- Job listesi/olusturma: `GET/POST /api/v1/ai/jobs`
- Job sonuc guncelleme: `PATCH /api/v1/ai/jobs/:id`
- Onay: `POST /api/v1/ai/jobs/:id/approve`
- Gorev taslagi: `GET /api/v1/ai/jobs/:id/task-draft`
- Bildirim taslagi: `GET /api/v1/ai/jobs/:id/notification-draft`

Dis model icin gerekli ayarlar:

- `AI_PROVIDER=openai` veya `gemini` veya `claude`
- `AI_MODEL`
- `AI_API_KEY`
- `AI_TIMEOUT_SECONDS=60`

Local model icin gerekli ayarlar:

- `AI_PROVIDER=ollama`
- `AI_MODEL=llama3.1`
- `AI_BASE_URL=http://127.0.0.1:11434`
- `AI_TIMEOUT_SECONDS=60`

Mevcut guvenlik kurali:

- AI kritik veritabanı degisikligini otomatik uygulamaz.
- AI sonucu once `ai_analysis_jobs` kaydina yazilir.
- Finansal/status etkisi olan aksiyonlar insan onayi ister.
- Frontend serbest veri birlestirmez; context paketleri backend tarafinda uretilir.
- AI sonucundan gorev veya bildirim sadece taslak olarak uretilir.

Sonraki baglama adimi:

1. Local deneme icin Ollama veya LM Studio kurulur.
2. Model indirilir ve servis calistirilir.
3. `.env` icinde `AI_PROVIDER`, `AI_MODEL`, `AI_BASE_URL` girilir.
4. Backend yeniden baslatilir.
5. AI ekranindaki "AI Baglanti Hazirligi" panelinde eksik ayar kalmadigi kontrol edilir.
6. Worker katmani pending/queued AI job kayitlarini isleyip `PATCH /api/v1/ai/jobs/:id` ile sonucu yazar.
