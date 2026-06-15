import { createColumnHelper } from "@tanstack/react-table";
import { useMutation, useQueries, useQueryClient } from "@tanstack/react-query";
import { Pencil, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { AuditLogRow, SettingRow, SystemLogRow } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";

const settingColumnHelper = createColumnHelper<SettingRow>();

const logColumns = [
  createColumnHelper<SystemLogRow>().accessor("service_name", { header: "Servis" }),
  createColumnHelper<SystemLogRow>().accessor("severity", {
    header: "Seviye",
    cell: (info) => <StatusBadge value={info.getValue()} />
  }),
  createColumnHelper<SystemLogRow>().accessor("message", {
    header: "Mesaj",
    cell: (info) => <span className="line-clamp-1">{info.getValue()}</span>
  }),
  createColumnHelper<SystemLogRow>().accessor("created_at", {
    header: "Tarih",
    cell: (info) => formatDateTime(info.getValue())
  })
];

const auditColumns = [
  createColumnHelper<AuditLogRow>().accessor("table_name", {
    header: "Tablo",
    cell: (info) => humanize(info.getValue())
  }),
  createColumnHelper<AuditLogRow>().accessor("record_id", {
    header: "Kayıt",
    cell: (info) => info.getValue() ?? "-"
  }),
  createColumnHelper<AuditLogRow>().accessor("action_type", {
    header: "İşlem",
    cell: (info) => <StatusBadge value={info.getValue()} />
  }),
  createColumnHelper<AuditLogRow>().accessor("created_by", {
    header: "Kullanıcı",
    cell: (info) => info.getValue() ?? "-"
  }),
  createColumnHelper<AuditLogRow>().accessor("created_at", {
    header: "Tarih",
    cell: (info) => formatDateTime(info.getValue())
  })
];

export function SystemPage() {
  const [settingFormOpen, setSettingFormOpen] = useState(false);
  const [editingSettingKey, setEditingSettingKey] = useState<string | null>(null);
  const [settingValues, setSettingValues] = useState<Record<string, string>>({
    setting_key: "",
    setting_value: "true",
    description: "",
    is_active: "true"
  });
  const [logValues, setLogValues] = useState<Record<string, string>>({
    service_name: "frontend",
    severity: "info",
    message: "",
    context: "{}"
  });
  const queryClient = useQueryClient();
  const [settingsQuery, logsQuery, auditQuery] = useQueries({
    queries: [
      { queryKey: ["settings"], queryFn: api.settings },
      { queryKey: ["system-logs"], queryFn: api.systemLogs },
      { queryKey: ["audit-logs"], queryFn: api.auditLogs }
    ]
  });

  const loading = [settingsQuery, logsQuery, auditQuery].some((query) => query.isLoading);
  const error = [settingsQuery, logsQuery, auditQuery].find((query) => query.isError)?.error;
  const settingMutation = useMutation({
    mutationFn: () => {
      const payload = buildSettingPayload(settingValues);
      return editingSettingKey
        ? api.updateSetting(editingSettingKey, payload)
        : api.upsertSetting({
            setting_key: settingValues.setting_key,
            ...payload
          });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
      setSettingFormOpen(false);
      setEditingSettingKey(null);
      setSettingValues({ setting_key: "", setting_value: "true", description: "", is_active: "true" });
    }
  });
  const logMutation = useMutation({
    mutationFn: () => api.createSystemLog(buildSystemLogPayload(logValues)),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["system-logs"] });
      setLogValues({ service_name: "frontend", severity: "info", message: "", context: "{}" });
    }
  });
  const settingColumns = useMemo(
    () => [
      settingColumnHelper.accessor("setting_key", {
        header: "Ayar",
        cell: (info) => <span className="font-medium">{info.getValue()}</span>
      }),
      settingColumnHelper.accessor("setting_value", {
        header: "Değer",
        cell: (info) => <span className="line-clamp-1">{JSON.stringify(info.getValue())}</span>
      }),
      settingColumnHelper.accessor("description", {
        header: "Açıklama",
        cell: (info) => info.getValue() ?? "-"
      }),
      settingColumnHelper.accessor("is_active", {
        header: "Durum",
        cell: (info) => <StatusBadge value={info.getValue() ? "active" : "passive"} />
      }),
      settingColumnHelper.accessor("updated_at", {
        header: "Güncelleme",
        cell: (info) => formatDateTime(info.getValue())
      }),
      settingColumnHelper.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => {
              const row = info.row.original;
              setEditingSettingKey(row.setting_key);
              setSettingValues({
                setting_key: row.setting_key,
                setting_value: JSON.stringify(row.setting_value, null, 2),
                description: row.description ?? "",
                is_active: String(row.is_active)
              });
              setSettingFormOpen(true);
            }}
          >
            <Pencil size={14} />
            Düzenle
          </Button>
        )
      })
    ],
    []
  );

  if (loading) return <LoadingBlock />;
  if (error) return <ErrorBlock error={error} />;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Sistem</h1>
        <p className="mt-1 text-sm text-muted-foreground">Ayarlar, hata logları ve audit izleri.</p>
      </div>

      <Panel
        title="Ayarlar"
        action={
          <Button
            size="sm"
            onClick={() => {
              setEditingSettingKey(null);
              setSettingValues({ setting_key: "", setting_value: "true", description: "", is_active: "true" });
              setSettingFormOpen((value) => !value);
            }}
          >
            <Plus size={15} />
            Yeni Ayar
          </Button>
        }
      >
        {settingFormOpen && (
          <form
            className="mb-4 rounded-lg border border-border bg-background p-4"
            onSubmit={(event) => {
              event.preventDefault();
              settingMutation.mutate();
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Ayar Anahtarı</span>
                <Input
                  disabled={Boolean(editingSettingKey)}
                  required
                  value={settingValues.setting_key}
                  onChange={(event) => setSettingValues((current) => ({ ...current, setting_key: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Aktif</span>
                <select
                  className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                  value={settingValues.is_active}
                  onChange={(event) => setSettingValues((current) => ({ ...current, is_active: event.target.value }))}
                >
                  <option value="true">Aktif</option>
                  <option value="false">Pasif</option>
                </select>
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Açıklama</span>
                <Input
                  value={settingValues.description}
                  onChange={(event) => setSettingValues((current) => ({ ...current, description: event.target.value }))}
                />
              </label>
            </div>
            <label className="mt-3 block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Değer JSON</span>
              <textarea
                className="min-h-28 w-full rounded-md border border-input bg-white px-3 py-2 font-mono text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={settingValues.setting_value}
                onChange={(event) => setSettingValues((current) => ({ ...current, setting_value: event.target.value }))}
              />
            </label>
            {settingMutation.isError && (
              <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {settingMutation.error instanceof Error ? settingMutation.error.message : "Ayar kaydedilemedi"}
              </div>
            )}
            <div className="mt-4 flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setSettingFormOpen(false)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={settingMutation.isPending}>
                Kaydet
              </Button>
            </div>
          </form>
        )}
        <DataTable data={settingsQuery.data ?? []} columns={settingColumns} emptyText="Ayar kaydı yok." />
      </Panel>

      <div className="grid gap-5 xl:grid-cols-2">
        <Panel title="Sistem Hata Logları">
          <form
            className="mb-4 rounded-lg border border-border bg-background p-4"
            onSubmit={(event) => {
              event.preventDefault();
              logMutation.mutate();
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Servis</span>
                <Input
                  required
                  value={logValues.service_name}
                  onChange={(event) => setLogValues((current) => ({ ...current, service_name: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Seviye</span>
                <select
                  className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                  value={logValues.severity}
                  onChange={(event) => setLogValues((current) => ({ ...current, severity: event.target.value }))}
                >
                  <option value="debug">Debug</option>
                  <option value="info">Info</option>
                  <option value="warning">Warning</option>
                  <option value="error">Error</option>
                  <option value="critical">Critical</option>
                </select>
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Mesaj</span>
                <Input
                  required
                  value={logValues.message}
                  onChange={(event) => setLogValues((current) => ({ ...current, message: event.target.value }))}
                />
              </label>
            </div>
            <label className="mt-3 block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Context JSON</span>
              <textarea
                className="min-h-20 w-full rounded-md border border-input bg-white px-3 py-2 font-mono text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={logValues.context}
                onChange={(event) => setLogValues((current) => ({ ...current, context: event.target.value }))}
              />
            </label>
            {logMutation.isError && (
              <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {logMutation.error instanceof Error ? logMutation.error.message : "Log kaydedilemedi"}
              </div>
            )}
            <div className="mt-4 flex justify-end">
              <Button type="submit" disabled={logMutation.isPending}>
                Log Kaydet
              </Button>
            </div>
          </form>
          <DataTable data={logsQuery.data ?? []} columns={logColumns} emptyText="Sistem logu yok." />
        </Panel>
        <Panel title="Audit Logları">
          <DataTable data={auditQuery.data ?? []} columns={auditColumns} emptyText="Audit kaydı yok." />
        </Panel>
      </div>
    </div>
  );
}

function buildSettingPayload(values: Record<string, string>) {
  let parsedValue: unknown;
  try {
    parsedValue = JSON.parse(values.setting_value);
  } catch {
    parsedValue = values.setting_value;
  }

  return {
    setting_value: parsedValue,
    description: values.description || undefined,
    is_active: values.is_active === "true"
  };
}

function buildSystemLogPayload(values: Record<string, string>) {
  let context: unknown;
  try {
    context = JSON.parse(values.context);
  } catch {
    context = { raw: values.context };
  }

  return {
    service_name: values.service_name,
    severity: values.severity,
    message: values.message,
    context
  };
}
