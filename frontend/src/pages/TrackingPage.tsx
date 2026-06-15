import { useQueries } from "@tanstack/react-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Bell, CalendarClock, ClipboardList, FileWarning, Pencil, ReceiptText, ShieldCheck, Trash2 } from "lucide-react";
import { useState } from "react";
import { appConfirm } from "../components/AppDialog";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { OperationRecord } from "../lib/types";
import { formatDate, formatDateTime, formatMoney, humanize } from "../lib/utils";

const modules = [
  { key: "tasks", path: "tasks", label: "Görevler", icon: ClipboardList, status: "task_status", date: "due_date" },
  { key: "maintenances", path: "maintenances", label: "Bakım", icon: CalendarClock, status: "maintenance_status", date: "maintenance_date" },
  { key: "insurance", path: "insurance-policies", label: "Poliçeler", icon: ShieldCheck, status: "renewal_status", date: "end_date" },
  { key: "expenses", path: "expenses", label: "Giderler", icon: ReceiptText, status: "payment_status", date: "expense_date" },
  { key: "damages", path: "damages", label: "Hasarlar", icon: FileWarning, status: "damage_status", date: "damage_date" },
  { key: "notifications", path: "notifications", label: "Bildirimler", icon: Bell, status: "delivery_status", date: "created_at" }
];

type FieldDef = {
  name: string;
  label: string;
  type?: "text" | "number" | "date" | "select" | "textarea";
  required?: boolean;
  options?: Array<{ value: string; label: string }>;
};

const moduleFields: Record<string, FieldDef[]> = {
  tasks: [
    { name: "task_type", label: "Görev Tipi", required: true },
    { name: "related_vehicle_id", label: "Araç ID", type: "number" },
    {
      name: "priority",
      label: "Öncelik",
      type: "select",
      options: [
        { value: "low", label: "Düşük" },
        { value: "medium", label: "Orta" },
        { value: "high", label: "Yüksek" },
        { value: "critical", label: "Kritik" }
      ]
    },
    { name: "due_date", label: "Termin", type: "date" },
    { name: "description", label: "Açıklama", type: "textarea" }
  ],
  maintenances: [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "last_maintenance_km", label: "Son Bakım KM", type: "number" },
    { name: "next_maintenance_km", label: "Sonraki Bakım KM", type: "number" },
    { name: "maintenance_date", label: "Bakım Tarihi", type: "date" },
    { name: "service_company", label: "Servis" },
    { name: "maintenance_type", label: "Bakım Tipi" },
    { name: "total_cost", label: "Toplam Tutar", type: "number" },
    {
      name: "maintenance_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "planned", label: "Planlı" },
        { value: "scheduled", label: "Randevulu" },
        { value: "completed", label: "Tamamlandı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "description", label: "Açıklama", type: "textarea" }
  ],
  insurance: [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    {
      name: "policy_type",
      label: "Poliçe Tipi",
      type: "select",
      required: true,
      options: [
        { value: "trafik", label: "Trafik" },
        { value: "kasko", label: "Kasko" },
        { value: "imm", label: "İMM" },
        { value: "ferdi_kaza", label: "Ferdi Kaza" }
      ]
    },
    { name: "policy_number", label: "Poliçe No", required: true },
    { name: "insurance_company", label: "Sigorta Şirketi" },
    { name: "agency_name", label: "Acente" },
    { name: "start_date", label: "Başlangıç", type: "date" },
    { name: "end_date", label: "Bitiş", type: "date" },
    { name: "amount", label: "Tutar", type: "number" },
    {
      name: "renewal_status",
      label: "Yenileme Durumu",
      type: "select",
      options: [
        { value: "active", label: "Aktif" },
        { value: "approaching", label: "Yaklaşıyor" },
        { value: "renewing", label: "Yenileniyor" },
        { value: "ended", label: "Bitti" }
      ]
    }
  ],
  expenses: [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    {
      name: "expense_type",
      label: "Gider Tipi",
      type: "select",
      required: true,
      options: [
        { value: "fuel", label: "Yakıt" },
        { value: "insurance", label: "Sigorta" },
        { value: "maintenance", label: "Bakım" },
        { value: "tax", label: "Vergi" },
        { value: "fine", label: "Ceza" },
        { value: "tire", label: "Lastik" },
        { value: "service", label: "Servis" }
      ]
    },
    { name: "amount", label: "Tutar", type: "number", required: true },
    { name: "invoice_number", label: "Fatura No" },
    {
      name: "payment_status",
      label: "Ödeme Durumu",
      type: "select",
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "paid", label: "Ödendi" },
        { value: "overdue", label: "Gecikti" }
      ]
    },
    { name: "expense_date", label: "Gider Tarihi", type: "date", required: true },
    { name: "note", label: "Not", type: "textarea" }
  ],
  damages: [
    { name: "vehicle_id", label: "Araç ID", type: "number", required: true },
    { name: "damage_date", label: "Hasar Tarihi", type: "date", required: true },
    { name: "damage_type", label: "Hasar Tipi" },
    { name: "estimated_cost", label: "Tahmini Tutar", type: "number" },
    { name: "actual_cost", label: "Gerçekleşen Tutar", type: "number" },
    { name: "insurance_claim_no", label: "Hasar Dosya No" },
    {
      name: "damage_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "expertise", label: "Ekspertiz" },
        { value: "insurance", label: "Sigorta" },
        { value: "repaired", label: "Onarıldı" },
        { value: "closed", label: "Kapandı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "description", label: "Açıklama", type: "textarea" }
  ],
  notifications: [
    { name: "notification_type", label: "Bildirim Tipi", required: true },
    { name: "receiver_user_id", label: "Alıcı Kullanıcı ID", type: "number" },
    { name: "related_vehicle_id", label: "Araç ID", type: "number" },
    {
      name: "sent_via",
      label: "Kanal",
      type: "select",
      required: true,
      options: [
        { value: "system", label: "Sistem" },
        { value: "whatsapp", label: "WhatsApp" },
        { value: "email", label: "Email" }
      ]
    },
    {
      name: "delivery_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "queued", label: "Kuyrukta" },
        { value: "sent", label: "Gönderildi" },
        { value: "failed", label: "Hatalı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "message", label: "Mesaj", type: "textarea", required: true }
  ]
};

const moduleEditFields: Record<string, FieldDef[]> = {
  tasks: [
    {
      name: "task_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "in_progress", label: "Devam Ediyor" },
        { value: "completed", label: "Tamamlandı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    {
      name: "priority",
      label: "Öncelik",
      type: "select",
      options: [
        { value: "low", label: "Düşük" },
        { value: "medium", label: "Orta" },
        { value: "high", label: "Yüksek" },
        { value: "critical", label: "Kritik" }
      ]
    },
    { name: "assigned_user_id", label: "Atanan Kullanıcı ID", type: "number" },
    { name: "assigned_department_id", label: "Departman ID", type: "number" },
    { name: "due_date", label: "Termin", type: "date" },
    { name: "description", label: "Açıklama", type: "textarea" }
  ],
  maintenances: [
    {
      name: "maintenance_status",
      label: "Durum",
      type: "select",
      required: true,
      options: [
        { value: "planned", label: "Planlı" },
        { value: "scheduled", label: "Randevulu" },
        { value: "completed", label: "Tamamlandı" },
        { value: "cancelled", label: "İptal" }
      ]
    }
  ],
  insurance: [
    {
      name: "renewal_status",
      label: "Yenileme Durumu",
      type: "select",
      required: true,
      options: [
        { value: "active", label: "Aktif" },
        { value: "approaching", label: "Yaklaşıyor" },
        { value: "renewing", label: "Yenileniyor" },
        { value: "ended", label: "Bitti" }
      ]
    }
  ],
  expenses: [
    {
      name: "payment_status",
      label: "Ödeme Durumu",
      type: "select",
      required: true,
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "paid", label: "Ödendi" },
        { value: "overdue", label: "Gecikti" },
        { value: "cancelled", label: "İptal" }
      ]
    }
  ],
  damages: [
    {
      name: "damage_status",
      label: "Durum",
      type: "select",
      options: [
        { value: "open", label: "Açık" },
        { value: "expertise", label: "Ekspertiz" },
        { value: "insurance", label: "Sigorta" },
        { value: "repaired", label: "Onarıldı" },
        { value: "closed", label: "Kapandı" },
        { value: "cancelled", label: "İptal" }
      ]
    },
    { name: "estimated_cost", label: "Tahmini Tutar", type: "number" },
    { name: "actual_cost", label: "Gerçekleşen Tutar", type: "number" },
    { name: "insurance_claim_no", label: "Hasar Dosya No" },
    { name: "description", label: "Açıklama", type: "textarea" }
  ],
  notifications: [
    {
      name: "delivery_status",
      label: "Durum",
      type: "select",
      required: true,
      options: [
        { value: "pending", label: "Bekliyor" },
        { value: "queued", label: "Kuyrukta" },
        { value: "sent", label: "Gönderildi" },
        { value: "failed", label: "Hatalı" },
        { value: "cancelled", label: "İptal" }
      ]
    }
  ]
};

export function TrackingPage() {
  const [activeKey, setActiveKey] = useState(modules[0].key);
  const [createOpen, setCreateOpen] = useState(false);
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [editingRow, setEditingRow] = useState<OperationRecord | null>(null);
  const [editValues, setEditValues] = useState<Record<string, string>>({});
  const queryClient = useQueryClient();
  const queries = useQueries({
    queries: modules.map((module) => ({
      queryKey: ["records", module.path],
      queryFn: () => api.records(module.path)
    }))
  });

  const activeIndex = modules.findIndex((module) => module.key === activeKey);
  const activeModule = modules[activeIndex];
  const activeQuery = queries[activeIndex];
  const activeRows = activeQuery.data ?? [];
  const firstError = queries.find((query) => query.isError)?.error;
  const createFields = moduleFields[activeKey] ?? [];
  const editFields = moduleEditFields[activeKey] ?? [];
  const createMutation = useMutation({
    mutationFn: (payload: Record<string, unknown>) => api.createRecord(activeModule.path, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", activeModule.path] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setFormValues({});
      setCreateOpen(false);
    }
  });
  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string | number; payload: Record<string, unknown> }) =>
      api.updateRecord(activeModule.path, id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", activeModule.path] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setEditingRow(null);
      setEditValues({});
    }
  });
  const deleteMutation = useMutation({
    mutationFn: (id: string | number) => api.deleteRecord(activeModule.path, id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", activeModule.path] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    }
  });
  const readMutation = useMutation({
    mutationFn: (id: string | number) => api.markNotificationRead(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", activeModule.path] });
    }
  });

  if (queries.some((query) => query.isLoading)) return <LoadingBlock />;
  if (firstError) return <ErrorBlock error={firstError} />;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Takip</h1>
        <p className="mt-1 text-sm text-muted-foreground">Görev, bakım, poliçe, gider, hasar ve bildirim kayıtları.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-6">
        {modules.map((module, index) => (
          <button key={module.key} className="text-left" type="button" onClick={() => setActiveKey(module.key)}>
            <MetricCard
              icon={module.icon}
              label={module.label}
              value={(queries[index].data ?? []).length}
              helper={activeKey === module.key ? "Seçili modül" : "Toplam kayıt"}
              tone={activeKey === module.key ? "success" : "default"}
            />
          </button>
        ))}
      </div>

      <Panel
        title={activeModule.label}
        action={
          <div className="flex flex-wrap gap-2">
            <Button size="sm" onClick={() => setCreateOpen((value) => !value)}>
              Yeni Kayıt
            </Button>
              {modules.map((module) => (
              <Button
                key={module.key}
                size="sm"
                variant={activeKey === module.key ? "primary" : "secondary"}
                onClick={() => {
                  setActiveKey(module.key);
                  setCreateOpen(false);
                  setEditingRow(null);
                  setFormValues({});
                  setEditValues({});
                }}
              >
                {module.label}
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
              const payload = buildPayload(createFields, formValues);
              createMutation.mutate(payload);
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              {createFields.map((field) => (
                <RecordField
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
                <h2 className="text-sm font-semibold">Kayıt Düzenle #{String(editingRow.id)}</h2>
                <p className="mt-1 text-xs text-muted-foreground">
                  Bu modülde backend’in kabul ettiği güncelleme alanları gösteriliyor.
                </p>
              </div>
              <Button type="button" size="sm" variant="secondary" onClick={() => setEditingRow(null)}>
                Kapat
              </Button>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              {editFields.map((field) => (
                <RecordField
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
        <div className="grid gap-3 lg:grid-cols-2">
          {activeRows.map((row) => (
            <RecordCard
              key={String(row.id ?? JSON.stringify(row))}
              row={row}
              module={activeModule}
              canMutate={editFields.length > 0}
              canDelete={activeKey !== "notifications"}
              canMarkRead={activeKey === "notifications" && !row.read_at}
              onEdit={() => {
                setCreateOpen(false);
                setEditingRow(row);
                setEditValues(valuesFromRow(editFields, row));
              }}
              onDelete={async () => {
                const recordId = row.id;
                if (typeof recordId !== "string" && typeof recordId !== "number") return;
                const confirmed = await appConfirm(`${activeModule.label} #${String(recordId)} arşivlensin mi?`, {
                  title: "Kayıt arşivle",
                  confirmLabel: "Arşivle",
                  tone: "danger"
                });
                if (confirmed) deleteMutation.mutate(recordId);
              }}
              onMarkRead={() => {
                const recordId = row.id;
                if (typeof recordId !== "string" && typeof recordId !== "number") return;
                readMutation.mutate(recordId);
              }}
            />
          ))}
          {activeRows.length === 0 && (
            <div className="rounded-md border border-border bg-background px-3 py-8 text-center text-sm text-muted-foreground">
              {activeModule.label} kaydı yok.
            </div>
          )}
        </div>
      </Panel>
    </div>
  );
}

function RecordField({
  field,
  value,
  onChange
}: {
  field: FieldDef;
  value: string;
  onChange: (value: string) => void;
}) {
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
          type={field.type ?? "text"}
          required={field.required}
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
    payload[field.name] = field.type === "number" ? Number(value) : value;
  }
  return payload;
}

function RecordCard({
  row,
  module,
  canMutate,
  canDelete,
  canMarkRead,
  onEdit,
  onDelete,
  onMarkRead
}: {
  row: OperationRecord;
  module: { label: string; status: string; date: string };
  canMutate: boolean;
  canDelete: boolean;
  canMarkRead: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onMarkRead: () => void;
}) {
  const title = pick(row, ["plate", "task_type", "policy_number", "expense_type", "damage_type", "notification_type"]) ?? `${module.label} #${row.id ?? "-"}`;
  const amount = pick(row, ["amount", "total_cost", "estimated_cost", "gross_premium"]);
  const dateValue = row[module.date];
  const status = row[module.status];

  return (
    <article className="rounded-lg border border-border bg-white px-4 py-3 shadow-panel">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h2 className="truncate text-sm font-semibold">{humanize(String(title))}</h2>
          <p className="mt-1 text-xs text-muted-foreground">
            #{row.id ?? "-"} / {formatRecordDate(String(dateValue ?? ""))}
          </p>
        </div>
        <StatusBadge value={status ? String(status) : null} />
      </div>
      <div className="mt-3 grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
        <Info label="Araç" value={String(row.plate ?? row.vehicle_id ?? "-")} />
        <Info label="Tutar" value={amount ? formatMoney(amount as string | number) : "-"} />
        <Info label="Öncelik" value={humanize(String(row.priority ?? "-"))} />
        <Info label="Güncelleme" value={formatDateTime(String(row.updated_at ?? row.created_at ?? ""))} />
      </div>
      {(canMutate || canDelete || canMarkRead) && (
        <div className="mt-3 flex justify-end gap-2">
          {canMutate && (
            <Button size="sm" variant="secondary" onClick={onEdit}>
              <Pencil size={14} />
              Düzenle
            </Button>
          )}
          {canMarkRead && (
            <Button size="sm" variant="secondary" onClick={onMarkRead}>
              Okundu
            </Button>
          )}
          {canDelete && (
            <Button size="sm" variant="danger" onClick={onDelete}>
              <Trash2 size={14} />
              Arşivle
            </Button>
          )}
        </div>
      )}
    </article>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0">
      <span className="text-muted-foreground">{label}: </span>
      <span className="font-medium text-foreground">{value}</span>
    </div>
  );
}

function pick(row: OperationRecord, keys: string[]) {
  for (const key of keys) {
    const value = row[key];
    if (value !== null && value !== undefined && value !== "") return value;
  }
  return null;
}

function formatRecordDate(value: string) {
  if (!value) return "-";
  return value.includes("T") ? formatDateTime(value) : formatDate(value);
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
