import { useQuery } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../lib/api";
import { humanize } from "../lib/utils";
import { StatusBadge } from "./StatusBadge";

export function GlobalSearch() {
  const [q, setQ] = useState("");
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const normalized = q.trim();
  const query = useQuery({
    queryKey: ["global-search", normalized],
    queryFn: () => api.search(normalized),
    enabled: normalized.length >= 2
  });

  const goToResult = (type: string, id: number) => {
    setOpen(false);
    setQ("");
    if (type === "vehicle") {
      navigate(`/vehicles/${id}`);
      return;
    }
    if (type === "support_ticket" || type === "support_knowledge") {
      navigate("/support");
      return;
    }
    navigate("/dashboard");
  };

  return (
    <div className="relative hidden md:block">
      <div className="flex h-10 min-w-[320px] items-center gap-2 rounded-md border border-border bg-muted px-3 text-sm">
        <Search size={16} className="text-muted-foreground" />
        <input
          value={q}
          onChange={(event) => {
            setQ(event.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          className="w-full bg-transparent outline-none placeholder:text-muted-foreground"
          placeholder="Araç, plaka, kullanıcı, poliçe, destek, çözüm"
        />
      </div>

      {open && normalized.length >= 2 && (
        <div className="absolute left-0 top-12 z-40 w-[420px] rounded-lg border border-border bg-white shadow-lg">
          <div className="max-h-[420px] overflow-y-auto p-2">
            {query.isLoading && <div className="px-3 py-4 text-sm text-muted-foreground">Aranıyor...</div>}
            {query.isError && <div className="px-3 py-4 text-sm text-red-700">Arama yapılamadı.</div>}
            {query.data?.map((result) => (
              <button
                key={`${result.result_type}-${result.id}`}
                className="flex w-full items-center justify-between gap-3 rounded-md px-3 py-2 text-left hover:bg-muted"
                type="button"
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => goToResult(result.result_type, result.id)}
              >
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{result.title}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {humanize(result.result_type)} / {result.subtitle ?? "-"}
                  </div>
                </div>
                <StatusBadge value={result.status} />
              </button>
            ))}
            {query.data?.length === 0 && (
              <div className="px-3 py-4 text-sm text-muted-foreground">Sonuç bulunamadı.</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
