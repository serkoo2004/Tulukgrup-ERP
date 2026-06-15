import { createColumnHelper } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FileUp, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "../components/DataTable";
import { appConfirm } from "../components/AppDialog";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api, uploadFile } from "../lib/api";
import type { FileDocumentRow } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";

const moduleOptions = [
  { value: "vehicles", label: "Araç" },
  { value: "insurance_policies", label: "Poliçe" },
  { value: "expenses", label: "Gider" },
  { value: "maintenances", label: "Bakım" },
  { value: "damages", label: "Hasar" },
  { value: "sold_vehicles", label: "Satılan Araç" },
  { value: "inventory_products", label: "Stok Ürünü" },
  { value: "support_tickets", label: "IT Destek Talebi" },
  { value: "support_knowledge_base", label: "IT Bilgi Bankası" }
];

const fileTypeOptions: Record<string, Array<{ value: string; label: string }>> = {
  vehicles: [
    { value: "registration", label: "Ruhsat" },
    { value: "delivery_form", label: "Teslim Formu" },
    { value: "vehicle_photo", label: "Araç Fotoğrafı" }
  ],
  insurance_policies: [
    { value: "policy_pdf", label: "Poliçe PDF" },
    { value: "offer", label: "Teklif" },
    { value: "invoice", label: "Fatura" },
    { value: "payment_receipt", label: "Dekont" }
  ],
  expenses: [
    { value: "invoice", label: "Fatura" },
    { value: "payment_receipt", label: "Dekont" },
    { value: "expense_document", label: "Gider Belgesi" }
  ],
  maintenances: [
    { value: "invoice", label: "Fatura" },
    { value: "payment_receipt", label: "Dekont" },
    { value: "expense_document", label: "Bakım Belgesi" }
  ],
  damages: [
    { value: "damage_report", label: "Hasar Raporu" },
    { value: "expert_report", label: "Eksper Raporu" },
    { value: "damage_photo", label: "Hasar Fotoğrafı" },
    { value: "invoice", label: "Fatura" }
  ],
  sold_vehicles: [
    { value: "sale_document", label: "Satış Evrakı" },
    { value: "transfer_document", label: "Devir Evrakı" }
  ],
  inventory_products: [
    { value: "invoice", label: "Fatura" },
    { value: "technical_document", label: "Teknik Doküman" },
    { value: "warranty_document", label: "Garanti Belgesi" },
    { value: "product_photo", label: "Ürün Görseli" },
    { value: "usage_instruction", label: "Kullanım Talimatı" }
  ],
  support_tickets: [
    { value: "support_screenshot", label: "Ekran Görüntüsü" },
    { value: "support_video", label: "Destek Videosu" },
    { value: "support_log", label: "Log Dosyası" },
    { value: "support_document", label: "Destek Dokümanı" }
  ],
  support_knowledge_base: [
    { value: "knowledge_attachment", label: "Bilgi Bankası Eki" },
    { value: "technical_document", label: "Teknik Doküman" },
    { value: "usage_instruction", label: "Kullanım Talimatı" }
  ]
};

const columnHelper = createColumnHelper<FileDocumentRow>();

export function FilesPage() {
  const [moduleName, setModuleName] = useState("vehicles");
  const [entityId, setEntityId] = useState("");
  const [fileType, setFileType] = useState("registration");
  const [note, setNote] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const [filterModule, setFilterModule] = useState("");
  const [filterEntityId, setFilterEntityId] = useState("");
  const [filterFileType, setFilterFileType] = useState("");
  const queryClient = useQueryClient();
  const filesQueryString = buildFilesQuery(filterModule, filterEntityId, filterFileType);
  const filesQuery = useQuery({
    queryKey: ["files", filesQueryString],
    queryFn: () => api.files(filesQueryString)
  });
  const uploadMutation = useMutation({
    mutationFn: () => {
      if (!file) throw new Error("Dosya seçilmedi");
      return uploadFile({
        module_name: moduleName,
        entity_id: Number(entityId),
        file_type: fileType,
        note,
        file
      });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["files"] });
      setEntityId("");
      setNote("");
      setFile(null);
    }
  });
  const deleteMutation = useMutation({
    mutationFn: (id: string | number) => api.deleteFile(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["files"] });
    }
  });

  const currentTypes = fileTypeOptions[moduleName] ?? [];
  const filterTypes = filterModule ? fileTypeOptions[filterModule] ?? [] : [];
  const columns = useMemo(
    () => [
      columnHelper.accessor("original_name", {
        header: "Dosya",
        cell: (info) => <span className="font-medium">{info.getValue()}</span>
      }),
      columnHelper.accessor("module_name", {
        header: "Modül",
        cell: (info) => humanize(info.getValue())
      }),
      columnHelper.accessor("entity_id", { header: "Kayıt ID" }),
      columnHelper.accessor("file_type", {
        header: "Tip",
        cell: (info) => <StatusBadge value={info.getValue()} />
      }),
      columnHelper.accessor("file_size", {
        header: "Boyut",
        cell: (info) => `${Math.max(1, Math.round(info.getValue() / 1024)).toLocaleString("tr-TR")} KB`
      }),
      columnHelper.accessor("created_at", {
        header: "Yükleme",
        cell: (info) => formatDateTime(info.getValue())
      }),
      columnHelper.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => (
          <Button
            size="sm"
            variant="danger"
            onClick={async () => {
              const confirmed = await appConfirm(`${info.row.original.original_name} arşivlensin mi?`, {
                title: "Dosya arşivle",
                confirmLabel: "Arşivle",
                tone: "danger"
              });
              if (confirmed) deleteMutation.mutate(info.row.original.id);
            }}
            disabled={deleteMutation.isPending}
          >
            <Trash2 size={14} />
            Arşivle
          </Button>
        )
      })
    ],
    [deleteMutation]
  );

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Dosyalar</h1>
        <p className="mt-1 text-sm text-muted-foreground">Araç, poliçe, gider, bakım, hasar, satış, stok ve IT destek evrakları.</p>
      </div>

      <Panel title="Dosya Yükle" description="Kayıt ID, ilgili modüldeki gerçek kayıt numarasıdır.">
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            uploadMutation.mutate();
          }}
        >
          <div className="grid gap-3 md:grid-cols-4">
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Modül</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={moduleName}
                onChange={(event) => {
                  const nextModule = event.target.value;
                  setModuleName(nextModule);
                  setFileType(fileTypeOptions[nextModule]?.[0]?.value ?? "");
                }}
              >
                {moduleOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Kayıt ID</span>
              <Input required type="number" value={entityId} onChange={(event) => setEntityId(event.target.value)} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Dosya Tipi</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={fileType}
                onChange={(event) => setFileType(event.target.value)}
              >
                {currentTypes.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Dosya</span>
              <Input required type="file" onChange={(event) => setFile(event.target.files?.[0] ?? null)} />
            </label>
          </div>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Not</span>
            <Input value={note} onChange={(event) => setNote(event.target.value)} />
          </label>
          {uploadMutation.isError && (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {uploadMutation.error instanceof Error ? uploadMutation.error.message : "Dosya yüklenemedi"}
            </div>
          )}
          {uploadMutation.isSuccess && (
            <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
              Dosya yüklendi.
            </div>
          )}
          <div className="flex justify-end">
            <Button type="submit" disabled={uploadMutation.isPending}>
              <FileUp size={17} />
              Yükle
            </Button>
          </div>
        </form>
      </Panel>

      <Panel title="Dosya Filtreleri">
        <div className="grid gap-3 md:grid-cols-4">
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Modül</span>
            <select
              className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
              value={filterModule}
              onChange={(event) => {
                setFilterModule(event.target.value);
                setFilterFileType("");
              }}
            >
              <option value="">Tümü</option>
              {moduleOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Kayıt ID</span>
            <Input type="number" value={filterEntityId} onChange={(event) => setFilterEntityId(event.target.value)} />
          </label>
          <label className="block">
            <span className="mb-1 block text-xs font-medium text-muted-foreground">Dosya Tipi</span>
            <select
              className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
              value={filterFileType}
              onChange={(event) => setFilterFileType(event.target.value)}
              disabled={!filterModule}
            >
              <option value="">Tümü</option>
              {filterTypes.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <div className="flex items-end">
            <Button
              className="w-full"
              type="button"
              variant="secondary"
              onClick={() => {
                setFilterModule("");
                setFilterEntityId("");
                setFilterFileType("");
              }}
            >
              Temizle
            </Button>
          </div>
        </div>
      </Panel>

      <Panel title="Son Dosyalar">
        {deleteMutation.isError && (
          <div className="mb-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
            {deleteMutation.error instanceof Error ? deleteMutation.error.message : "Dosya arşivlenemedi"}
          </div>
        )}
        {filesQuery.isLoading ? (
          <LoadingBlock />
        ) : filesQuery.isError ? (
          <ErrorBlock error={filesQuery.error} />
        ) : (
          <DataTable data={filesQuery.data ?? []} columns={columns} emptyText="Dosya kaydı yok." />
        )}
      </Panel>
    </div>
  );
}

function buildFilesQuery(moduleName: string, entityId: string, fileType: string) {
  const params = new URLSearchParams({ limit: "80" });
  if (moduleName) params.set("module_name", moduleName);
  if (entityId) params.set("entity_id", entityId);
  if (fileType) params.set("file_type", fileType);
  return params.toString();
}
