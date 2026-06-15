import {
  AlertTriangle,
  Car,
  ClipboardCheck,
  FileWarning,
  Gauge,
  Headset,
  PackageSearch,
  ReceiptText,
  ShieldCheck,
  Wrench
} from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import { formatDateTime, formatMoney, humanize } from "../lib/utils";

export function DashboardPage() {
  const query = useQuery({ queryKey: ["dashboard"], queryFn: api.dashboard });

  if (query.isLoading) return <LoadingBlock />;
  if (query.isError) return <ErrorBlock error={query.error} />;
  const data = query.data!;

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold">Dashboard</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Son güncelleme: {formatDateTime(data.generated_at)}
          </p>
        </div>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          icon={Car}
          label="Toplam araç"
          value={data.vehicles.total_count}
          helper={`${data.vehicles.active_count} aktif, ${data.vehicles.assigned_count} atanmış`}
        />
        <MetricCard
          icon={ClipboardCheck}
          label="Açık görev"
          value={data.tasks.open_count}
          helper={`${data.tasks.critical_count} kritik, ${data.tasks.overdue_count} gecikmiş`}
          tone={data.tasks.overdue_count > 0 ? "danger" : "default"}
        />
        <MetricCard
          icon={ShieldCheck}
          label="Biten poliçe riski"
          value={data.policies.ending_in_30_days + data.policies.ended_count}
          helper={`${data.policies.active_count} aktif poliçe`}
          tone={data.policies.ended_count > 0 ? "danger" : "warning"}
        />
        <MetricCard
          icon={ReceiptText}
          label="Bu ay gider"
          value={formatMoney(data.expenses.current_month_total)}
          helper={`${data.expenses.pending_count} bekleyen ödeme`}
        />
      </div>

      <div className="grid gap-5 xl:grid-cols-[1fr_420px]">
        <Panel title="Operasyon Durumu">
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            <MetricCard
              icon={Wrench}
              label="Planlı bakım"
              value={data.maintenances.planned_count + data.maintenances.scheduled_count}
              helper={`${data.maintenances.completed_count} tamamlandı`}
            />
            <MetricCard
              icon={FileWarning}
              label="Açık hasar"
              value={data.damages.open_count}
              helper={`Tahmini açık maliyet: ${formatMoney(data.damages.estimated_open_cost)}`}
              tone={data.damages.open_count > 0 ? "warning" : "default"}
            />
            <MetricCard
              icon={Gauge}
              label="Yakıt / yıkama"
              value={formatMoney(data.operations.current_month_fuel_total)}
              helper={`Yıkama: ${formatMoney(data.operations.current_month_wash_total)}`}
            />
            <MetricCard
              icon={PackageSearch}
              label="Stok değeri"
              value={formatMoney(data.inventory.total_stock_value)}
              helper={`${data.inventory.total_products} ürün kartı`}
            />
            <MetricCard
              icon={PackageSearch}
              label="Kritik stok"
              value={data.inventory.critical_stock_count + data.inventory.out_of_stock_count}
              helper={`${data.inventory.out_of_stock_count} tükenen, ${data.inventory.expiring_lots_30_days} SKT riski`}
              tone={data.inventory.out_of_stock_count > 0 ? "danger" : data.inventory.critical_stock_count > 0 ? "warning" : "default"}
            />
            <MetricCard
              icon={PackageSearch}
              label="Bekleyen sevkiyat"
              value={data.inventory.pending_shipments}
              helper={`${data.inventory.pending_purchase_requests} satın alma talebi`}
              tone={data.inventory.pending_shipments > 0 ? "warning" : "default"}
            />
            <MetricCard
              icon={Headset}
              label="IT destek"
              value={data.support.open_count + data.support.in_progress_count}
              helper={`${data.support.whatsapp_count} WhatsApp, ${data.support.resolution_overdue_count} SLA geciken`}
              tone={data.support.resolution_overdue_count > 0 || data.support.critical_count > 0 ? "danger" : data.support.open_count > 0 ? "warning" : "default"}
            />
          </div>
        </Panel>

        <Panel title="Uyarılar">
          {data.warnings.length === 0 ? (
            <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-4 text-sm text-emerald-700">
              Kritik uyarı yok.
            </div>
          ) : (
            <div className="space-y-2">
              {data.warnings.map((warning) => (
                <div
                  key={warning.warning_type}
                  className="flex items-center justify-between gap-3 rounded-md border border-border px-3 py-2"
                >
                  <div className="flex min-w-0 items-center gap-2">
                    <AlertTriangle className="h-4 w-4 shrink-0 text-amber-600" />
                    <span className="truncate text-sm">{warning.title}</span>
                  </div>
                  <span className="rounded-md bg-muted px-2 py-1 text-xs font-semibold">{warning.count}</span>
                </div>
              ))}
            </div>
          )}
        </Panel>
      </div>

      <Panel title="Son Sistem Hareketleri">
        <div className="divide-y divide-border">
          {data.recent_activity.map((activity) => (
            <div key={`${activity.table_name}-${activity.record_id}-${activity.created_at}`} className="flex flex-wrap items-center justify-between gap-3 py-2 text-sm">
              <div>
                <span className="font-medium">{humanize(activity.table_name)}</span>
                <span className="text-muted-foreground"> / {humanize(activity.action_type)}</span>
              </div>
              <div className="text-xs text-muted-foreground">{formatDateTime(activity.created_at)}</div>
            </div>
          ))}
          {data.recent_activity.length === 0 && (
            <div className="py-6 text-center text-sm text-muted-foreground">Henüz audit kaydı yok.</div>
          )}
        </div>
      </Panel>
    </div>
  );
}
