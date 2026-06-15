import { createColumnHelper, type ColumnDef } from "@tanstack/react-table";
import { Fuel, Pencil, ReceiptText, SearchCheck, Shield, ShowerHead, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useMutation, useQueries, useQueryClient } from "@tanstack/react-query";
import { appConfirm } from "../components/AppDialog";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { OperationRecord } from "../lib/types";
import { formatDate, formatMoney, humanize } from "../lib/utils";

const operationTypes = [
  {
    key: "vehicle-inspections",
    label: "Teslim / Kontrol",
    icon: SearchCheck,
    statusKey: "inspection_status",
    dateKey: "inspection_date",
    amountKey: "fee"
  },
  {
    key: "value-loss-claims",
    label: "Değer Kaybı",
    icon: ReceiptText,
    statusKey: "claim_status",
    dateKey: "accident_date",
    amountKey: "requested_amount"
  },
  {
    key: "fuel-entries",
    label: "Yakıt",
    icon: Fuel,
    statusKey: "",
    dateKey: "period_month",
    amountKey: "paid_amount"
  },
  {
    key: "vehicle-washes",
    label: "Yıkama",
    icon: ShowerHead,
    statusKey: "",
    dateKey: "wash_date",
    amountKey: "amount"
  },
  {
    key: "insurance-quotes",
    label: "Sigorta Teklif",
    icon: Shield,
    statusKey: "quote_status",
    dateKey: "valid_until",
    amountKey: "gross_premium"
  }
];

const columnHelper = createColumnHelper<OperationRecord>();

type FieldDef = {
  name: string;
  label: string;
  type?: "text" | "number" | "date" | "select" | "textarea" | "checkbox";
  required?: boolean;
  options?: Array<{ value: string; label: string }>;
};

const operationFields: Record<string, FieldDef[]> = {
  "vehicle-inspections": [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "branch", label: "Şube" },
    { name: "inspection_date", label: "Kontrol Tarihi", type: "date", required: true },
    { name: "exterior_ok", label: "Dış Aksam OK", type: "checkbox" },
    { name: "interior_ok", label: "İç Aksam OK", type: "checkbox" },
    { name: "equipment_ok", label: "Ekipman OK", type: "checkbox" },
    { name: "documents_ok", label: "Evrak OK", type: "checkbox" },
    { name: "fee", label: "Ücret", type: "number" },
    {
      name: "inspection_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "completed", label: "Tamamlandı" },
        { value: "closed", label: "Kapalı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "damage_note", label: "Hasar Notu", type: "textarea" },
    { name: "action_note", label: "Aksiyon Notu", type: "textarea" }
  ],
  "value-loss-claims": [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "accident_date", label: "Kaza Tarihi", type: "date", required: true },
    { name: "vehicle_purchase_date", label: "Araç Alım Tarihi", type: "date" },
    { name: "tramer_amount", label: "Tramer Tutarı", type: "number" },
    { name: "deprivation_days", label: "Mahrumiyet Gün", type: "number" },
    { name: "requested_amount", label: "Talep Tutarı", type: "number" },
    { name: "received_amount", label: "Alınan Tutar", type: "number" },
    {
      name: "claim_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "in_review", label: "İncelemede" },
        { value: "paid", label: "Ödendi" },
        { value: "closed", label: "Kapalı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "fuel-entries": [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "period_year", label: "Yıl", type: "number", required: true },
    { name: "period_month", label: "Ay", type: "number", required: true },
    { name: "fuel_limit", label: "Yakıt Limiti", type: "number" },
    { name: "paid_amount", label: "Ödenen Tutar", type: "number" },
    { name: "remaining_limit", label: "Kalan Limit", type: "number" },
    { name: "distance_km", label: "Mesafe KM", type: "number" },
    { name: "current_km", label: "Güncel KM", type: "number" },
    { name: "liter_amount", label: "Litre", type: "number" },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "vehicle-washes": [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "branch", label: "Şube" },
    { name: "wash_company", label: "Yıkama Firması" },
    { name: "wash_date", label: "Yıkama Tarihi", type: "date", required: true },
    { name: "amount", label: "Tutar", type: "number" },
    { name: "user_id", label: "Kullanıcı ID", type: "number" },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "insurance-quotes": [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "quote_type", label: "Teklif Tipi" },
    { name: "insurance_company", label: "Sigorta Şirketi" },
    { name: "agency_name", label: "Acente" },
    { name: "gross_premium", label: "Brüt Prim", type: "number" },
    { name: "installment_count", label: "Taksit", type: "number" },
    {
      name: "quote_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "accepted", label: "Kabul" },
        { value: "rejected", label: "Red" },
        { value: "expired", label: "Süresi Doldu" }
      ]
    },
    { name: "valid_until", label: "Geçerlilik", type: "date" },
    { name: "note", label: "Not", type: "textarea" }
  ]
};

const operationEditFields: Record<string, FieldDef[]> = {
  "vehicle-inspections": [
    { name: "branch", label: "Şube" },
    { name: "inspection_date", label: "Kontrol Tarihi", type: "date" },
    { name: "inspector_user_id", label: "Kontrol Eden Kullanıcı ID", type: "number" },
    { name: "exterior_ok", label: "Dış Aksam OK", type: "checkbox" },
    { name: "interior_ok", label: "İç Aksam OK", type: "checkbox" },
    { name: "equipment_ok", label: "Ekipman OK", type: "checkbox" },
    { name: "documents_ok", label: "Evrak OK", type: "checkbox" },
    { name: "fee", label: "Ücret", type: "number" },
    {
      name: "inspection_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "completed", label: "Tamamlandı" },
        { value: "closed", label: "Kapalı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "damage_note", label: "Hasar Notu", type: "textarea" },
    { name: "action_note", label: "Aksiyon Notu", type: "textarea" },
    { name: "expert_report", label: "Ekspertiz Raporu", type: "textarea" }
  ],
  "value-loss-claims": [
    { name: "accident_date", label: "Kaza Tarihi", type: "date" },
    { name: "vehicle_purchase_date", label: "Araç Alım Tarihi", type: "date" },
    { name: "tramer_amount", label: "Tramer Tutarı", type: "number" },
    { name: "deprivation_days", label: "Mahrumiyet Gün", type: "number" },
    { name: "requested_amount", label: "Talep Tutarı", type: "number" },
    { name: "received_amount", label: "Alınan Tutar", type: "number" },
    {
      name: "claim_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "in_review", label: "İncelemede" },
        { value: "paid", label: "Ödendi" },
        { value: "closed", label: "Kapalı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "fuel-entries": [
    { name: "fuel_limit", label: "Yakıt Limiti", type: "number" },
    { name: "paid_amount", label: "Ödenen Tutar", type: "number" },
    { name: "remaining_limit", label: "Kalan Limit", type: "number" },
    { name: "distance_km", label: "Mesafe KM", type: "number" },
    { name: "current_km", label: "Güncel KM", type: "number" },
    { name: "liter_amount", label: "Litre", type: "number" },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "vehicle-washes": [
    { name: "branch", label: "Şube" },
    { name: "wash_company", label: "Yıkama Firması" },
    { name: "wash_date", label: "Yıkama Tarihi", type: "date" },
    { name: "amount", label: "Tutar", type: "number" },
    { name: "user_id", label: "Kullanıcı ID", type: "number" },
    { name: "note", label: "Not", type: "textarea" }
  ],
  "insurance-quotes": [
    { name: "quote_type", label: "Teklif Tipi" },
    { name: "insurance_company", label: "Sigorta Şirketi" },
    { name: "agency_name", label: "Acente" },
    { name: "gross_premium", label: "Brüt Prim", type: "number" },
    { name: "installment_count", label: "Taksit", type: "number" },
    {
      name: "quote_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "accepted", label: "Kabul" },
        { value: "rejected", label: "Red" },
        { value: "expired", label: "Süresi Doldu" }
      ]
    },
    { name: "valid_until", label: "Geçerlilik", type: "date" },
    { name: "note", label: "Not", type: "textarea" }
  ]
};

export function OperationsPage() {
  const [activeKey, setActiveKey] = useState(operationTypes[0].key);
  const [createOpen, setCreateOpen] = useState(false);
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [editingRow, setEditingRow] = useState<OperationRecord | null>(null);
  const [editValues, setEditValues] = useState<Record<string, string>>({});
  const queryClient = useQueryClient();
  const queries = useQueries({
    queries: operationTypes.map((type) => ({
      queryKey: ["operations", type.key],
      queryFn: () => api.operations(type.key, 50)
    }))
  });

  const activeIndex = operationTypes.findIndex((type) => type.key === activeKey);
  const activeType = operationTypes[activeIndex];
  const activeQuery = queries[activeIndex];
  const activeData = activeQuery.data ?? [];
  const fields = operationFields[activeKey] ?? [];
  const editFields = operationEditFields[activeKey] ?? [];
  const createMutation = useMutation({
    mutationFn: (payload: Record<string, unknown>) => api.createOperation(activeKey, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["operations", activeKey] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setFormValues({});
      setCreateOpen(false);
    }
  });
  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string | number; payload: Record<string, unknown> }) =>
      api.updateOperation(activeKey, id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["operations", activeKey] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setEditingRow(null);
      setEditValues({});
    }
  });
  const deleteMutation = useMutation({
    mutationFn: (id: string | number) => api.deleteOperation(activeKey, id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["operations", activeKey] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    }
  });

  const columns = useMemo<ColumnDef<OperationRecord, any>[]>(
    () => [
      columnHelper.accessor("plate", {
        header: "Plaka",
        cell: (info) => <span className="font-semibold text-primary">{String(info.getValue() ?? "-")}</span>
      }),
      columnHelper.accessor(activeType.dateKey, {
        header: activeType.key === "fuel-entries" ? "Dönem" : "Tarih",
        cell: (info) =>
          activeType.key === "fuel-entries"
            ? `${info.row.original.period_month ?? "-"} / ${info.row.original.period_year ?? "-"}`
            : formatDate(String(info.getValue() ?? ""))
      }),
      columnHelper.accessor(activeType.amountKey, {
        header: "Tutar",
        cell: (info) => formatMoney(info.getValue() as string | number | null)
      }),
      columnHelper.display({
        id: "detail",
        header: "Detay",
        cell: (info) => {
          const row = info.row.original;
          return (
            <span className="text-muted-foreground">
              {String(row.branch ?? row.insurance_company ?? row.wash_company ?? row.assigned_user_name ?? row.note ?? "-")}
            </span>
          );
        }
      }),
      columnHelper.display({
        id: "status",
        header: "Durum",
        cell: (info) =>
          activeType.statusKey ? (
            <StatusBadge value={String(info.row.original[activeType.statusKey] ?? "")} />
          ) : (
            <span className="text-muted-foreground">-</span>
          )
      }),
      columnHelper.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => {
          const row = info.row.original;
          const recordId = row.id;
          const canMutate = typeof recordId === "string" || typeof recordId === "number";
          return (
            <div className="flex justify-end gap-2">
              <Button
                size="sm"
                variant="secondary"
                disabled={!canMutate}
                onClick={() => {
                  setCreateOpen(false);
                  setEditingRow(row);
                  setEditValues(valuesFromRow(editFields, row));
                }}
              >
                <Pencil size={14} />
                Düzenle
              </Button>
              <Button
                size="sm"
                variant="danger"
                disabled={!canMutate || deleteMutation.isPending}
                onClick={async () => {
                  if (!canMutate) return;
                  const confirmed = await appConfirm(`${activeType.label} #${String(recordId)} arşivlensin mi?`, {
                    title: "Kayıt arşivle",
                    confirmLabel: "Arşivle",
                    tone: "danger"
                  });
                  if (confirmed) deleteMutation.mutate(recordId);
                }}
              >
                <Trash2 size={14} />
                Arşivle
              </Button>
            </div>
          );
        }
      })
    ],
    [activeType, deleteMutation, editFields]
  );

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Operasyon</h1>
        <p className="mt-1 text-sm text-muted-foreground">Araç kontrol, değer kaybı, yakıt, yıkama ve sigorta teklif kayıtları.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        {operationTypes.map((type, index) => {
          const data = queries[index].data ?? [];
          return (
            <button
              key={type.key}
              className="text-left"
              type="button"
              onClick={() => {
                setActiveKey(type.key);
                setCreateOpen(false);
                setEditingRow(null);
                setFormValues({});
                setEditValues({});
              }}
            >
              <MetricCard icon={type.icon} label={type.label} value={data.length} helper={activeKey === type.key ? "Seçili kayıt grubu" : "Son 50 kayıt"} tone={activeKey === type.key ? "success" : "default"} />
            </button>
          );
        })}
      </div>

      <Panel
        title={activeType.label}
        action={
          <div className="flex flex-wrap gap-2">
            <Button size="sm" onClick={() => setCreateOpen((value) => !value)}>
              Yeni Kayıt
            </Button>
            {operationTypes.map((type) => (
              <Button
                key={type.key}
                variant={activeKey === type.key ? "primary" : "secondary"}
                size="sm"
                onClick={() => {
                  setActiveKey(type.key);
                  setCreateOpen(false);
                  setEditingRow(null);
                  setFormValues({});
                  setEditValues({});
                }}
              >
                {type.label}
              </Button>
            ))}
          </div>
        }
      >
        {createOpen && (
          <form
            className="mb-4 rounded-lg border border-border bg-background p-4"
            onSubmit={(event) => {
              event.preventDefault();
              createMutation.mutate(buildPayload(fields, formValues));
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              {fields.map((field) => (
                <OperationField
                  key={field.name}
                  field={field}
                  value={formValues[field.name] ?? ""}
                  onChange={(value) => setFormValues((current) => ({ ...current, [field.name]: value }))}
                />
              ))}
            </div>
            {createMutation.isError && (
              <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {createMutation.error instanceof Error ? createMutation.error.message : "Kayıt oluşturulamadı"}
              </div>
            )}
            <div className="mt-4 flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setCreateOpen(false)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={createMutation.isPending}>
                Kaydet
              </Button>
            </div>
          </form>
        )}
        {editingRow && (
          <form
            className="mb-4 rounded-lg border border-border bg-background p-4"
            onSubmit={(event) => {
              event.preventDefault();
              const recordId = editingRow.id;
              if (typeof recordId !== "string" && typeof recordId !== "number") return;
              updateMutation.mutate({
                id: recordId,
                payload: buildPayload(editFields, editValues)
              });
            }}
          >
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
              <div>
                <h2 className="text-sm font-semibold">Operasyon Kaydı Düzenle #{String(editingRow.id)}</h2>
                <p className="mt-1 text-xs text-muted-foreground">
                  Bu form backend’in bu operasyon için güncellenebilir kabul ettiği alanları gösterir.
                </p>
              </div>
              <Button type="button" size="sm" variant="secondary" onClick={() => setEditingRow(null)}>
                Kapat
              </Button>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              {editFields.map((field) => (
                <OperationField
                  key={field.name}
                  field={field}
                  value={editValues[field.name] ?? ""}
                  onChange={(value) => setEditValues((current) => ({ ...current, [field.name]: value }))}
                />
              ))}
            </div>
            {updateMutation.isError && (
              <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {updateMutation.error instanceof Error ? updateMutation.error.message : "Kayıt güncellenemedi"}
              </div>
            )}
            <div className="mt-4 flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setEditingRow(null)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                Kaydet
              </Button>
            </div>
          </form>
        )}
        {activeQuery.isLoading ? (
          <LoadingBlock />
        ) : activeQuery.isError ? (
          <ErrorBlock error={activeQuery.error} />
        ) : (
          <DataTable data={activeData} columns={columns} emptyText={`${humanize(activeType.label)} kaydı yok.`} />
        )}
      </Panel>
    </div>
  );
}

function OperationField({
  field,
  value,
  onChange
}: {
  field: FieldDef;
  value: string;
  onChange: (value: string) => void;
}) {
  if (field.type === "checkbox") {
    return (
      <label className="flex h-10 items-center gap-2 rounded-md border border-border bg-white px-3 text-sm">
        <input checked={value === "true"} type="checkbox" onChange={(event) => onChange(String(event.target.checked))} />
        {field.label}
      </label>
    );
  }

  return (
    <label className={field.type === "textarea" ? "block md:col-span-3" : "block"}>
      <span className="mb-1 block text-xs font-medium text-muted-foreground">
        {field.label}
        {field.required ? " *" : ""}
      </span>
      {field.type === "select" ? (
        <select
          className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
          required={field.required}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        >
          <option value="">Seçiniz</option>
          {field.options?.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      ) : field.type === "textarea" ? (
        <textarea
          className="min-h-24 w-full rounded-md border border-input bg-white px-3 py-2 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
          required={field.required}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
      ) : (
        <Input
          required={field.required}
          type={field.type ?? "text"}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
      )}
    </label>
  );
}

function buildPayload(fields: FieldDef[], values: Record<string, string>) {
  const payload: Record<string, unknown> = {};
  for (const field of fields) {
    const value = values[field.name];
    if (value === undefined || value === "") continue;
    if (field.type === "number") {
      payload[field.name] = Number(value);
    } else if (field.type === "checkbox") {
      payload[field.name] = value === "true";
    } else {
      payload[field.name] = value;
    }
  }
  return payload;
}

function valuesFromRow(fields: FieldDef[], row: OperationRecord) {
  const values: Record<string, string> = {};
  for (const field of fields) {
    const value = row[field.name];
    if (value !== null && value !== undefined) {
      values[field.name] = String(value);
    }
  }
  return values;
}
