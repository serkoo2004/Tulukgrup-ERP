import type { LucideIcon } from "lucide-react";
import { cn } from "../lib/utils";

type MetricCardProps = {
  label: string;
  value: string | number;
  helper?: string;
  icon: LucideIcon;
  tone?: "default" | "success" | "warning" | "danger";
};

const toneClass = {
  default: "bg-slate-100 text-slate-700",
  success: "bg-emerald-100 text-emerald-700",
  warning: "bg-amber-100 text-amber-700",
  danger: "bg-red-100 text-red-700"
};

export function MetricCard({ label, value, helper, icon: Icon, tone = "default" }: MetricCardProps) {
  return (
    <div className="rounded-lg border border-border bg-white p-4 shadow-panel">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-xs font-medium text-muted-foreground">{label}</p>
          <p className="mt-2 text-2xl font-semibold leading-none text-foreground">{value}</p>
        </div>
        <div className={cn("flex h-9 w-9 shrink-0 items-center justify-center rounded-md", toneClass[tone])}>
          <Icon size={18} aria-hidden="true" />
        </div>
      </div>
      {helper && <p className="mt-3 text-xs text-muted-foreground">{helper}</p>}
    </div>
  );
}
