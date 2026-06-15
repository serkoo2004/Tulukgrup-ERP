import { createColumnHelper } from "@tanstack/react-table";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { Vehicle, VehicleCreateInput } from "../lib/types";
import { formatDate, humanize } from "../lib/utils";

const columnHelper = createColumnHelper<Vehicle>();

const vehicleSchema = z.object({
  plate: z.string().min(2, "Plaka zorunlu"),
  brand: z.string().min(1, "Marka zorunlu"),
  model: z.string().min(1, "Model zorunlu"),
  model_year: z.coerce.number().int().min(1950).max(2100).optional().or(z.literal("")),
  vehicle_type: z.string().optional(),
  fuel_type: z.string().optional(),
  transmission: z.string().optional(),
  chassis_no: z.string().optional(),
  engine_no: z.string().optional(),
  warranty_status: z.string().optional(),
  warranty_end: z.string().optional(),
  has_hgs: z.boolean().optional(),
  has_mobiliz: z.boolean().optional(),
  has_kopilot: z.boolean().optional(),
  has_k2: z.boolean().optional(),
  tasitmatik_company: z.string().optional(),
  spare_key_location: z.string().optional()
});

type VehicleForm = z.infer<typeof vehicleSchema>;

export function VehiclesPage() {
  const [search, setSearch] = useState("");
  const [formOpen, setFormOpen] = useState(false);
  const queryClient = useQueryClient();
  const queryString = search.trim()
    ? `status=all&q=${encodeURIComponent(search.trim())}`
    : "status=all";
  const query = useQuery({
    queryKey: ["vehicles", queryString],
    queryFn: () => api.vehicles(queryString)
  });

  const form = useForm<VehicleForm>({
    resolver: zodResolver(vehicleSchema),
    defaultValues: {
      plate: "",
      brand: "",
      model: "",
      model_year: "",
      vehicle_type: "",
      fuel_type: "",
      transmission: "",
      chassis_no: "",
      engine_no: "",
      warranty_status: "",
      warranty_end: "",
      has_hgs: false,
      has_mobiliz: false,
      has_kopilot: false,
      has_k2: false,
      tasitmatik_company: "",
      spare_key_location: ""
    }
  });

  const createMutation = useMutation({
    mutationFn: (payload: VehicleCreateInput) => api.createVehicle(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      form.reset();
      setFormOpen(false);
    }
  });

  const submitVehicle = form.handleSubmit((values) => {
    const payload: VehicleCreateInput = Object.fromEntries(
      Object.entries(values).filter(([, value]) => value !== "" && value !== undefined)
    ) as VehicleCreateInput;
    if (typeof values.model_year === "number") {
      payload.model_year = values.model_year;
    }
    createMutation.mutate(payload);
  });

  const columns = useMemo(
    () => [
      columnHelper.accessor("plate", {
        header: "Plaka",
        cell: (info) => (
          <Link className="font-semibold text-primary hover:underline" to={`/vehicles/${info.row.original.id}`}>
            {info.getValue()}
          </Link>
        )
      }),
      columnHelper.accessor((row) => `${row.brand} ${row.model}`, {
        id: "vehicle",
        header: "Araç",
        cell: (info) => <span>{info.getValue()}</span>
      }),
      columnHelper.accessor("model_year", {
        header: "Yıl",
        cell: (info) => info.getValue() ?? "-"
      }),
      columnHelper.accessor("vehicle_type", {
        header: "Tip",
        cell: (info) => humanize(info.getValue())
      }),
      columnHelper.accessor("fuel_type", {
        header: "Yakıt",
        cell: (info) => humanize(info.getValue())
      }),
      columnHelper.accessor("warranty_end", {
        header: "Garanti",
        cell: (info) => formatDate(info.getValue())
      }),
      columnHelper.accessor("status", {
        header: "Durum",
        cell: (info) => <StatusBadge value={info.getValue()} />
      })
    ],
    []
  );

  return (
    <div className="space-y-5">
      <div>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold">Araçlar</h1>
            <p className="mt-1 text-sm text-muted-foreground">Filo envanteri ve araç profil kayıtları.</p>
          </div>
          <Button onClick={() => setFormOpen((value) => !value)}>
            <Plus size={17} />
            Yeni Araç
          </Button>
        </div>
      </div>
      {formOpen && (
        <Panel title="Yeni Araç Kaydı" description="Zorunlu alanlar plaka, marka ve modeldir. Diğer bilgiler daha sonra güncellenebilir.">
          <form className="space-y-4" onSubmit={submitVehicle}>
            <div className="grid gap-3 md:grid-cols-3">
              <Field label="Plaka" error={form.formState.errors.plate?.message}>
                <Input {...form.register("plate")} placeholder="34 ABC 123" />
              </Field>
              <Field label="Marka" error={form.formState.errors.brand?.message}>
                <Input {...form.register("brand")} placeholder="Ford" />
              </Field>
              <Field label="Model" error={form.formState.errors.model?.message}>
                <Input {...form.register("model")} placeholder="Transit" />
              </Field>
              <Field label="Model Yılı" error={form.formState.errors.model_year?.message}>
                <Input type="number" {...form.register("model_year")} placeholder="2024" />
              </Field>
              <Field label="Araç Tipi">
                <Input {...form.register("vehicle_type")} placeholder="panelvan" />
              </Field>
              <Field label="Yakıt Tipi">
                <Input {...form.register("fuel_type")} placeholder="dizel" />
              </Field>
              <Field label="Vites">
                <Input {...form.register("transmission")} placeholder="otomatik" />
              </Field>
              <Field label="Şasi No">
                <Input {...form.register("chassis_no")} />
              </Field>
              <Field label="Motor No">
                <Input {...form.register("engine_no")} />
              </Field>
              <Field label="Garanti Durumu">
                <Input {...form.register("warranty_status")} placeholder="devam ediyor" />
              </Field>
              <Field label="Garanti Bitişi">
                <Input type="date" {...form.register("warranty_end")} />
              </Field>
              <Field label="Taşıtmatik Firma">
                <Input {...form.register("tasitmatik_company")} />
              </Field>
              <Field label="Yedek Anahtar">
                <Input {...form.register("spare_key_location")} />
              </Field>
            </div>
            <div className="grid gap-2 sm:grid-cols-4">
              {[
                ["has_hgs", "HGS"],
                ["has_mobiliz", "Mobiliz"],
                ["has_kopilot", "Kopilot"],
                ["has_k2", "K2"]
              ].map(([key, label]) => (
                <label key={key} className="flex h-10 items-center gap-2 rounded-md border border-border bg-background px-3 text-sm">
                  <input type="checkbox" {...form.register(key as keyof VehicleForm)} />
                  {label}
                </label>
              ))}
            </div>
            {createMutation.isError && (
              <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {createMutation.error instanceof Error ? createMutation.error.message : "Araç oluşturulamadı"}
              </div>
            )}
            <div className="flex flex-wrap justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setFormOpen(false)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={createMutation.isPending}>
                Kaydet
              </Button>
            </div>
          </form>
        </Panel>
      )}
      <Panel
        title="Araç Listesi"
        action={
          <div className="relative w-full sm:w-72">
            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder="Plaka ara"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
            />
          </div>
        }
      >
        {query.isLoading ? (
          <LoadingBlock />
        ) : query.isError ? (
          <ErrorBlock error={query.error} />
        ) : (
          <DataTable data={query.data ?? []} columns={columns} />
        )}
      </Panel>
    </div>
  );
}

function Field({ label, error, children }: { label: string; error?: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      {children}
      {error && <span className="mt-1 block text-xs text-red-600">{error}</span>}
    </label>
  );
}
