import { cn, humanize } from "../lib/utils";

const toneMap: Record<string, string> = {
  active: "border-emerald-200 bg-emerald-50 text-emerald-700",
  completed: "border-emerald-200 bg-emerald-50 text-emerald-700",
  paid: "border-emerald-200 bg-emerald-50 text-emerald-700",
  open: "border-amber-200 bg-amber-50 text-amber-700",
  pending: "border-amber-200 bg-amber-50 text-amber-700",
  planned: "border-amber-200 bg-amber-50 text-amber-700",
  scheduled: "border-sky-200 bg-sky-50 text-sky-700",
  critical: "border-red-200 bg-red-50 text-red-700",
  overdue: "border-red-200 bg-red-50 text-red-700",
  ended: "border-red-200 bg-red-50 text-red-700",
  passive: "border-zinc-200 bg-zinc-50 text-zinc-700",
  sold: "border-zinc-200 bg-zinc-50 text-zinc-700",
  cancelled: "border-zinc-200 bg-zinc-50 text-zinc-700"
};

export function StatusBadge({ value }: { value?: string | null }) {
  const normalized = value ?? "";
  return (
    <span
      className={cn(
        "inline-flex min-h-6 items-center rounded-md border px-2 text-xs font-medium",
        toneMap[normalized] ?? "border-border bg-muted text-muted-foreground"
      )}
    >
      {humanize(value)}
    </span>
  );
}
