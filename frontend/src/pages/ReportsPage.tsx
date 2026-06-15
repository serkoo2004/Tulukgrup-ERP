import { Download, FileSpreadsheet } from "lucide-react";
import { useState } from "react";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { downloadReport } from "../lib/api";

const reports = [
  {
    title: "Yönetim Özeti",
    description: "Araç, görev, poliçe, bakım, gider, hasar ve operasyon KPI dosyası.",
    path: "/api/v1/reports/management.xlsx",
    file: "tuluklar-yonetim-ozeti.xlsx"
  },
  { title: "Araç Envanteri", description: "Araç teknik bilgileri ve takip kolonları.", path: "/api/v1/reports/vehicles.xlsx", file: "tuluklar-arac-envanteri.xlsx" },
  { title: "Bakım Kayıtları", description: "Planlı, randevulu ve tamamlanan bakım kayıtları.", path: "/api/v1/reports/maintenances.xlsx", file: "tuluklar-bakim.xlsx" },
  { title: "Poliçeler", description: "Trafik, kasko ve yenileme takip listesi.", path: "/api/v1/reports/insurance-policies.xlsx", file: "tuluklar-policeler.xlsx" },
  { title: "Giderler", description: "Fatura ve ödeme takibi.", path: "/api/v1/reports/expenses.xlsx", file: "tuluklar-giderler.xlsx" },
  { title: "Hasarlar", description: "Hasar, eksper, sigorta ve maliyet kayıtları.", path: "/api/v1/reports/damages.xlsx", file: "tuluklar-hasarlar.xlsx" },
  { title: "Teslim / Kontrol", description: "Araç kontrol ve ekspertiz kayıtları.", path: "/api/v1/reports/vehicle-inspections.xlsx", file: "tuluklar-arac-kontrol.xlsx" },
  { title: "Değer Kaybı", description: "Değer kaybı ve mahrumiyet dosyaları.", path: "/api/v1/reports/value-loss-claims.xlsx", file: "tuluklar-deger-kaybi.xlsx" },
  { title: "Yakıt", description: "Aylık limit, ödeme, litre ve KM kayıtları.", path: "/api/v1/reports/fuel-entries.xlsx", file: "tuluklar-yakit.xlsx" },
  { title: "Yıkama", description: "Şube, firma, kullanıcı ve tutar kayıtları.", path: "/api/v1/reports/vehicle-washes.xlsx", file: "tuluklar-yikama.xlsx" },
  { title: "Sigorta Teklifleri", description: "Teklif, acente, prim ve geçerlilik takibi.", path: "/api/v1/reports/insurance-quotes.xlsx", file: "tuluklar-sigorta-teklif.xlsx" },
  { title: "Stok Ürün Listesi", description: "Ürün kartları, birimler, koli çarpanı ve stok seviyeleri.", path: "/api/v1/reports/inventory-products.xlsx", file: "tuluklar-stok-urunleri.xlsx" },
  { title: "Stok Hareketleri", description: "Giriş, çıkış, sevkiyat, sayım, hurda ve araç kullanım hareketleri.", path: "/api/v1/reports/inventory-movements.xlsx", file: "tuluklar-stok-hareketleri.xlsx" },
  { title: "Kritik Stok", description: "Minimum/kritik stok altındaki ürünler ve önerilen sipariş miktarı.", path: "/api/v1/reports/inventory-critical.xlsx", file: "tuluklar-kritik-stok.xlsx" },
  { title: "IT Destek Talepleri", description: "Web, mobil ve WhatsApp kaynaklı talepler; durum, atama, çözüm ve olay geçmişi.", path: "/api/v1/reports/support-tickets.xlsx", file: "tuluklar-it-destek-talepleri.xlsx" },
  { title: "IT Bilgi Bankası", description: "Standart problem/çözüm kayıtları, etiketler ve kaynak talepler.", path: "/api/v1/reports/support-knowledge-base.xlsx", file: "tuluklar-it-bilgi-bankasi.xlsx" }
];

export function ReportsPage() {
  const [downloading, setDownloading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState({
    vehicle_id: "",
    status: "",
    start_date: "",
    end_date: ""
  });

  const handleDownload = async (path: string, file: string) => {
    setError(null);
    setDownloading(path);
    try {
      await downloadReport(withReportFilters(path, filters), file);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Rapor indirilemedi");
    } finally {
      setDownloading(null);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Raporlar</h1>
        <p className="mt-1 text-sm text-muted-foreground">Tüm çıktılar kurumsal biçimlendirilmiş XLSX dosyasıdır.</p>
      </div>

      {error && <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{error}</div>}

      <Panel title="Rapor Filtreleri" description="Destekleyen raporlarda araç, durum ve tarih aralığı XLSX çıktısına uygulanır.">
        <div className="grid gap-3 md:grid-cols-5">
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Araç ID</span>
            <Input
              type="number"
              value={filters.vehicle_id}
              onChange={(event) => setFilters((current) => ({ ...current, vehicle_id: event.target.value }))}
            />
          </label>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Durum</span>
            <Input
              value={filters.status}
              placeholder="active / open / pending"
              onChange={(event) => setFilters((current) => ({ ...current, status: event.target.value }))}
            />
          </label>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Başlangıç</span>
            <Input
              type="date"
              value={filters.start_date}
              onChange={(event) => setFilters((current) => ({ ...current, start_date: event.target.value }))}
            />
          </label>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Bitiş</span>
            <Input
              type="date"
              value={filters.end_date}
              onChange={(event) => setFilters((current) => ({ ...current, end_date: event.target.value }))}
            />
          </label>
          <div className="flex items-end">
            <Button
              className="w-full"
              type="button"
              variant="secondary"
              onClick={() => setFilters({ vehicle_id: "", status: "", start_date: "", end_date: "" })}
            >
              Temizle
            </Button>
          </div>
        </div>
      </Panel>

      <div className="grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
        {reports.map((report) => (
          <Panel key={report.path} className="h-full">
            <div className="flex h-full flex-col gap-4">
              <div className="flex gap-3">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-emerald-100 text-emerald-700">
                  <FileSpreadsheet size={19} aria-hidden="true" />
                </div>
                <div className="min-w-0">
                  <h2 className="text-sm font-semibold">{report.title}</h2>
                  <p className="mt-1 text-sm leading-5 text-muted-foreground">{report.description}</p>
                </div>
              </div>
              <Button
                className="mt-auto w-full"
                variant="secondary"
                onClick={() => handleDownload(report.path, report.file)}
                disabled={downloading === report.path}
              >
                <Download size={17} />
                XLSX İndir
              </Button>
            </div>
          </Panel>
        ))}
      </div>
    </div>
  );
}

function withReportFilters(path: string, filters: Record<string, string>) {
  const params = new URLSearchParams();
  Object.entries(filters).forEach(([key, value]) => {
    if (value) params.set(key, value);
  });
  const query = params.toString();
  return query ? `${path}?${query}` : path;
}
