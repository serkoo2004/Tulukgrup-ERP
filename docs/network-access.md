# Network Access

Local development varsayilan olarak bu makinede calisir. Baska cihazdan test icin frontend ve backend `0.0.0.0` uzerinden dinler.

## Aynı Wi-Fi

Bu makinenin Wi-Fi IP adresi ornek:

```text
192.168.1.106
```

Frontend:

```text
http://192.168.1.106:5173
```

Backend:

```text
http://192.168.1.106:8080
```

## Tailscale

Bu makinenin Tailscale IP adresi ornek:

```text
100.95.238.26
```

Frontend:

```text
http://100.95.238.26:5173
```

Backend:

```text
http://100.95.238.26:8080
```

Frontend API adresini acilan host uzerinden otomatik secer. Yani Tailscale ile `100.95.238.26:5173` acilirsa API de `100.95.238.26:8080` uzerinden gider.

Mobil Expo icin API adresi ortam degiskeni ile verilir:

```powershell
$env:EXPO_PUBLIC_API_BASE_URL="http://100.95.238.26:8080"
.\scripts\start-mobile.ps1
```

## Komutlar

```powershell
.\scripts\start-local-postgres.ps1
.\scripts\start-backend.ps1
.\scripts\start-frontend.ps1
.\scripts\start-mobile.ps1
```

Durum:

```powershell
.\scripts\status-local-postgres.ps1
.\scripts\status-frontend.ps1
netstat -ano -p tcp | Select-String -Pattern ':5173|:8080'
```

Durdurma:

```powershell
.\scripts\stop-frontend.ps1
.\scripts\stop-backend.ps1
.\scripts\stop-local-postgres.ps1
```

## Firewall

Aynı Wi-Fi veya Tailscale ile sayfa acilmiyorsa Windows Firewall `5173` ve `8080` portlarini engelliyor olabilir.

Gelistirme makinesinde sadece test icin izin verilecek portlar:

```text
5173 frontend
8080 backend
```

Prod/server kurulumunda bu sekilde Vite portu acik birakilmaz; build alinip Nginx arkasinda yayinlanir.
