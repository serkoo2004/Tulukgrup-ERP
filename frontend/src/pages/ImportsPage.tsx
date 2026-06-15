import { createColumnHelper } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, Upload } from "lucide-react";
import { useState } from "react";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { ImportErrorRow, ImportJobRow, ImportStagingRow } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";
import { parseFirstWorksheet } from "../lib/xlsx";

const importModules = [
  { value: "vehicles", label: "Araçlar" },
  { value: "users", label: "Kullanıcılar" },
  { value: "km_logs", label: "KM Kayıtları" },
  { value: "maintenances", label: "Bakımlar" },
  { value: "insurance_policies", label: "Poliçeler" },
  { value: "expenses", label: "Giderler" },
  { value: "damages", label: "Hasarlar" }
];

const examples: Record<string, string> = {
  vehicles: JSON.stringify([{ plate: "34 ABC 123", brand: "Ford", model: "Transit", model_year: 2024 }], null, 2),
  users: JSON.stringify([{ email: "user@tuluklar.local", full_name: "Kullanıcı", role: "user" }], null, 2),
  km_logs: JSON.stringify([{ vehicle_id: 1, km: 42000, entry_type: "manual" }], null, 2),
  maintenances: JSON.stringify([{ vehicle_id: 1, last_maintenance_km: 30000, next_maintenance_km: 45000, maintenance_status: "planned" }], null, 2),
  insurance_policies: JSON.stringify([{ vehicle_id: 1, policy_type: "trafik", policy_number: "TRF-1", renewal_status: "active" }], null, 2),
  expenses: JSON.stringify([{ vehicle_id: 1, expense_type: "fuel", amount: "1500.00", expense_date: "2026-06-01" }], null, 2),
  damages: JSON.stringify([{ vehicle_id: 1, damage_date: "2026-06-01", damage_status: "open" }], null, 2)
};

const importStatuses = [
  { value: "pending", label: "Bekliyor" },
  { value: "mapping", label: "Mapping" },
  { value: "validating", label: "Validasyon" },
  { value: "ready", label: "Hazır" },
  { value: "importing", label: "İşleniyor" },
  { value: "completed", label: "Tamamlandı" },
  { value: "failed", label: "Hatalı" },
  { value: "rolled_back", label: "Geri Alındı" }
];

const columnHelper = createColumnHelper<ImportJobRow>();
const rowColumnHelper = createColumnHelper<ImportStagingRow>();
const errorColumnHelper = createColumnHelper<ImportErrorRow>();
const columns = [
  columnHelper.accessor("id", {
    header: "Job",
    cell: (info) => <span className="font-semibold">#{info.getValue()}</span>
  }),
  columnHelper.accessor("target_module", {
    header: "Modül",
    cell: (info) => humanize(info.getValue())
  }),
  columnHelper.accessor("import_status", {
    header: "Durum",
    cell: (info) => <StatusBadge value={info.getValue()} />
  }),
  columnHelper.accessor("total_rows", { header: "Toplam" }),
  columnHelper.accessor("processed_rows", { header: "Geçerli" }),
  columnHelper.accessor("error_count", { header: "Hata" }),
  columnHelper.accessor("created_at", {
    header: "Tarih",
    cell: (info) => formatDateTime(info.getValue())
  })
];

const rowColumns = [
  rowColumnHelper.accessor("row_number", { header: "Satır" }),
  rowColumnHelper.accessor("row_status", {
    header: "Durum",
    cell: (info) => <StatusBadge value={info.getValue()} />
  }),
  rowColumnHelper.accessor("error_count", { header: "Hata" }),
  rowColumnHelper.display({
    id: "raw_data",
    header: "Ham Veri",
    cell: (info) => <span className="font-mono text-xs">{compactJson(info.row.original.raw_data)}</span>
  }),
  rowColumnHelper.display({
    id: "normalized_data",
    header: "Normalize",
    cell: (info) => <span className="font-mono text-xs">{compactJson(info.row.original.normalized_data)}</span>
  })
];

const errorColumns = [
  errorColumnHelper.accessor("row_number", { header: "Satır", cell: (info) => info.getValue() ?? "-" }),
  errorColumnHelper.accessor("field_name", { header: "Alan", cell: (info) => info.getValue() ?? "-" }),
  errorColumnHelper.accessor("error_message", { header: "Hata" }),
  errorColumnHelper.display({
    id: "raw_data",
    header: "Ham Veri",
    cell: (info) => <span className="font-mono text-xs">{compactJson(info.row.original.raw_data)}</span>
  })
];

export function ImportsPage() {
  const [targetModule, setTargetModule] = useState("vehicles");
  const [note, setNote] = useState("");
  const [rowsText, setRowsText] = useState(examples.vehicles);
  const [selectedJobId, setSelectedJobId] = useState("");
  const [jobStatus, setJobStatus] = useState("ready");
  const [result, setResult] = useState<string | null>(null);
  const [detectedHeaders, setDetectedHeaders] = useState<string[]>([]);
  const [xlsxError, setXlsxError] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const jobsQuery = useQuery({ queryKey: ["import-jobs"], queryFn: api.importJobs });
  const selectedJobNumber = Number(selectedJobId);
  const rowsQuery = useQuery({
    queryKey: ["import-rows", selectedJobNumber],
    queryFn: () => api.importRows(selectedJobNumber),
    enabled: Boolean(selectedJobNumber)
  });
  const errorsQuery = useQuery({
    queryKey: ["import-errors", selectedJobNumber],
    queryFn: () => api.importErrors(selectedJobNumber),
    enabled: Boolean(selectedJobNumber)
  });

  const createMutation = useMutation({
    mutationFn: () =>
      api.createImportJob({
        target_module: targetModule,
        column_mapping: {},
        note
      }),
    onSuccess: async (job) => {
      await queryClient.invalidateQueries({ queryKey: ["import-jobs"] });
      setSelectedJobId(String(job.id));
      setResult(`Import job oluşturuldu: #${job.id}`);
    }
  });

  const stageMutation = useMutation({
    mutationFn: () => {
      const jobId = Number(selectedJobId);
      if (!jobId) throw new Error("Job seçilmedi");
      const parsed = JSON.parse(rowsText) as unknown;
      if (!Array.isArray(parsed)) throw new Error("Satırlar JSON array olmalı");
      return api.stageImportRows(jobId, parsed);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["import-jobs"] });
      await queryClient.invalidateQueries({ queryKey: ["import-rows", Number(selectedJobId)] });
      setResult("Satırlar staging alanına yazıldı.");
    }
  });

  const validateMutation = useMutation({
    mutationFn: () => {
      const jobId = Number(selectedJobId);
      if (!jobId) throw new Error("Job seçilmedi");
      return api.validateImportJob(jobId);
    },
    onSuccess: async (summary) => {
      await queryClient.invalidateQueries({ queryKey: ["import-jobs"] });
      await queryClient.invalidateQueries({ queryKey: ["import-rows", Number(selectedJobId)] });
      await queryClient.invalidateQueries({ queryKey: ["import-errors", Number(selectedJobId)] });
      setResult(
        `Validasyon tamamlandı: toplam ${summary.total_rows}, geçerli ${summary.valid_rows}, hatalı ${summary.invalid_rows}`
      );
    }
  });

  const statusMutation = useMutation({
    mutationFn: () => {
      const jobId = Number(selectedJobId);
      if (!jobId) throw new Error("Job seçilmedi");
      return api.updateImportJob(jobId, { import_status: jobStatus, note: note || undefined });
    },
    onSuccess: async (job) => {
      await queryClient.invalidateQueries({ queryKey: ["import-jobs"] });
      setResult(`Job #${job.id} durumu güncellendi: ${humanize(job.import_status)}`);
    }
  });

  const currentError = createMutation.error ?? stageMutation.error ?? validateMutation.error ?? statusMutation.error;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Import</h1>
        <p className="mt-1 text-sm text-muted-foreground">Excel import altyapısı için job, staging ve validasyon akışı.</p>
      </div>

      <Panel title="Import Hazırlığı" description="XLSX dosyası okunur, satırlar staging alanına yazılır ve backend validasyonundan geçirilir.">
        <div className="space-y-4">
          <div className="grid gap-3 md:grid-cols-4">
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Hedef Modül</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={targetModule}
                onChange={(event) => {
                  const value = event.target.value;
                  setTargetModule(value);
                  setRowsText(examples[value] ?? "[]");
                }}
              >
                {importModules.map((module) => (
                  <option key={module.value} value={module.value}>
                    {module.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Not</span>
              <Input value={note} onChange={(event) => setNote(event.target.value)} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Job ID</span>
              <Input
                value={selectedJobId}
                onChange={(event) => setSelectedJobId(event.target.value)}
                placeholder="Oluşturunca dolar"
              />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Job Durumu</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={jobStatus}
                onChange={(event) => setJobStatus(event.target.value)}
              >
                {importStatuses.map((status) => (
                  <option key={status.value} value={status.value}>
                    {status.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">XLSX Dosyası</span>
            <Input
              type="file"
              accept=".xlsx"
              onChange={async (event) => {
                const file = event.target.files?.[0];
                if (!file) return;
                setXlsxError(null);
                try {
                  const parsed = await parseFirstWorksheet(file);
                  setRowsText(JSON.stringify(parsed.rows, null, 2));
                  setDetectedHeaders(parsed.headers);
                  setResult(`${parsed.sheetName} sayfasından ${parsed.rows.length} satır okundu.`);
                } catch (err) {
                  setXlsxError(err instanceof Error ? err.message : "Excel dosyası okunamadı");
                }
              }}
            />
          </label>

          {detectedHeaders.length > 0 && (
            <div className="rounded-md border border-border bg-background px-3 py-2 text-xs text-muted-foreground">
              Okunan kolonlar: {detectedHeaders.join(", ")}
            </div>
          )}

          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Satırlar</span>
            <textarea
              className="min-h-52 w-full rounded-md border border-input bg-white px-3 py-2 font-mono text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
              value={rowsText}
              onChange={(event) => setRowsText(event.target.value)}
            />
          </label>

          {xlsxError && (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {xlsxError}
            </div>
          )}
          {currentError && (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {currentError instanceof Error ? currentError.message : "Import işlemi başarısız"}
            </div>
          )}
          {result && (
            <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
              {result}
            </div>
          )}

          <div className="flex flex-wrap justify-end gap-2">
            <Button variant="secondary" onClick={() => createMutation.mutate()} disabled={createMutation.isPending}>
              <Upload size={17} />
              Job Oluştur
            </Button>
            <Button variant="secondary" onClick={() => stageMutation.mutate()} disabled={stageMutation.isPending}>
              Satırları Yaz
            </Button>
            <Button onClick={() => validateMutation.mutate()} disabled={validateMutation.isPending}>
              <CheckCircle2 size={17} />
              Validate Et
            </Button>
            <Button variant="secondary" onClick={() => statusMutation.mutate()} disabled={statusMutation.isPending}>
              Durumu Güncelle
            </Button>
          </div>
        </div>
      </Panel>

      <Panel title="Import Jobları">
        {jobsQuery.isLoading ? (
          <LoadingBlock />
        ) : jobsQuery.isError ? (
          <ErrorBlock error={jobsQuery.error} />
        ) : (
          <DataTable
            data={jobsQuery.data ?? []}
            columns={columns}
            emptyText="Import job kaydı yok."
            onRowClick={(row) => setSelectedJobId(String(row.id))}
          />
        )}
      </Panel>

      {selectedJobNumber > 0 && (
        <Panel title={`Import Job Detayı #${selectedJobNumber}`}>
          <div className="grid gap-5 xl:grid-cols-2">
            <div>
              <h2 className="mb-3 text-sm font-semibold">Staging Satırları</h2>
              {rowsQuery.isLoading ? (
                <LoadingBlock />
              ) : rowsQuery.isError ? (
                <ErrorBlock error={rowsQuery.error} />
              ) : (
                <DataTable data={rowsQuery.data ?? []} columns={rowColumns} emptyText="Staging satırı yok." />
              )}
            </div>
            <div>
              <h2 className="mb-3 text-sm font-semibold">Validasyon Hataları</h2>
              {errorsQuery.isLoading ? (
                <LoadingBlock />
              ) : errorsQuery.isError ? (
                <ErrorBlock error={errorsQuery.error} />
              ) : (
                <DataTable data={errorsQuery.data ?? []} columns={errorColumns} emptyText="Validasyon hatası yok." />
              )}
            </div>
          </div>
        </Panel>
      )}
    </div>
  );
}

function compactJson(value: unknown) {
  if (value === null || value === undefined) return "-";
  try {
    const text = typeof value === "string" ? value : JSON.stringify(value);
    return text.length > 120 ? `${text.slice(0, 120)}...` : text;
  } catch {
    return "-";
  }
}
