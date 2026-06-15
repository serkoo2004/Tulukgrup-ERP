# Realtime Services

Go servisleri PDF mimarisindeki realtime katmanidir.

Baslangic kapsami:

- WebSocket gateway
- TCP socket server
- GPS tracking mesaj formati icin ilk kanal
- NATS/Kafka queue entegrasyonuna hazir servis siniri

gRPC servis tanimlari sonraki adimda `proto/` altinda eklenecek.

## Local kontrol

```powershell
.\scripts\check-realtime.ps1
.\scripts\start-realtime.ps1
```

Bu servis Go runtime ister. Go kurulu degilse check scripti hizli sekilde hata verir.
