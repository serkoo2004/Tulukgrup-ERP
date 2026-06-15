import { AlertCircle, Loader2 } from "lucide-react";

export function LoadingBlock({ label = "Veriler yükleniyor" }: { label?: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center gap-2 rounded-lg border border-border bg-white text-sm text-muted-foreground">
      <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
      {label}
    </div>
  );
}

export function ErrorBlock({ error }: { error: unknown }) {
  const message = error instanceof Error ? error.message : "Beklenmeyen hata";
  return (
    <div className="flex min-h-40 items-center justify-center gap-2 rounded-lg border border-red-200 bg-red-50 px-4 text-sm text-red-700">
      <AlertCircle className="h-4 w-4" aria-hidden="true" />
      {message}
    </div>
  );
}
