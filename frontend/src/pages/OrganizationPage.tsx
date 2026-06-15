import { createColumnHelper, type ColumnDef } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Building2, Pencil, Power, Rows3 } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { CompanyRow, DepartmentRow } from "../lib/types";
import { formatDateTime } from "../lib/utils";

const companyColumn = createColumnHelper<CompanyRow>();
const departmentColumn = createColumnHelper<DepartmentRow>();

export function OrganizationPage() {
  const queryClient = useQueryClient();
  const companiesQuery = useQuery({ queryKey: ["companies"], queryFn: api.companies });
  const departmentsQuery = useQuery({ queryKey: ["departments"], queryFn: api.departments });
  const [companyForm, setCompanyForm] = useState({ id: "", name: "", tax_number: "", is_active: "true" });
  const [departmentForm, setDepartmentForm] = useState({ id: "", company_id: "", name: "", is_active: "true" });

  const companies = companiesQuery.data ?? [];
  const departments = departmentsQuery.data ?? [];

  const companyMutation = useMutation({
    mutationFn: () => {
      const payload = {
        name: companyForm.name,
        tax_number: companyForm.tax_number || undefined,
        is_active: companyForm.is_active === "true"
      };
      return companyForm.id ? api.updateCompany(companyForm.id, payload) : api.createCompany(payload);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["companies"] });
      setCompanyForm({ id: "", name: "", tax_number: "", is_active: "true" });
    }
  });

  const companyStatusMutation = useMutation({
    mutationFn: ({ id, is_active }: { id: number; is_active: boolean }) => api.updateCompany(id, { is_active }),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["companies"] })
  });

  const departmentMutation = useMutation({
    mutationFn: () => {
      const payload = {
        company_id: departmentForm.company_id ? Number(departmentForm.company_id) : undefined,
        name: departmentForm.name,
        is_active: departmentForm.is_active === "true"
      };
      return departmentForm.id ? api.updateDepartment(departmentForm.id, payload) : api.createDepartment(payload);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["departments"] });
      setDepartmentForm({ id: "", company_id: "", name: "", is_active: "true" });
    }
  });

  const departmentStatusMutation = useMutation({
    mutationFn: ({ id, is_active }: { id: number; is_active: boolean }) => api.updateDepartment(id, { is_active }),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["departments"] })
  });

  const companyColumns = useMemo<ColumnDef<CompanyRow, any>[]>(
    () => [
      companyColumn.accessor("name", {
        header: "Şirket",
        cell: (info) => <span className="font-semibold text-primary">{info.getValue()}</span>
      }),
      companyColumn.accessor("tax_number", { header: "Vergi No", cell: (info) => info.getValue() ?? "-" }),
      companyColumn.accessor("is_active", {
        header: "Durum",
        cell: (info) => <StatusBadge value={info.getValue() ? "active" : "passive"} />
      }),
      companyColumn.accessor("updated_at", { header: "Güncelleme", cell: (info) => formatDateTime(info.getValue()) }),
      companyColumn.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => {
          const row = info.row.original;
          return (
            <div className="flex justify-end gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  setCompanyForm({
                    id: String(row.id),
                    name: row.name,
                    tax_number: row.tax_number ?? "",
                    is_active: String(row.is_active)
                  })
                }
              >
                <Pencil size={14} />
                Düzenle
              </Button>
              <Button
                size="sm"
                variant={row.is_active ? "danger" : "secondary"}
                onClick={() => companyStatusMutation.mutate({ id: row.id, is_active: !row.is_active })}
              >
                <Power size={14} />
                {row.is_active ? "Pasif" : "Aktif"}
              </Button>
            </div>
          );
        }
      })
    ],
    [companyStatusMutation]
  );

  const departmentColumns = useMemo<ColumnDef<DepartmentRow, any>[]>(
    () => [
      departmentColumn.accessor("name", {
        header: "Departman",
        cell: (info) => <span className="font-semibold text-primary">{info.getValue()}</span>
      }),
      departmentColumn.accessor("company_id", {
        header: "Şirket",
        cell: (info) => companyName(companies, info.getValue())
      }),
      departmentColumn.accessor("is_active", {
        header: "Durum",
        cell: (info) => <StatusBadge value={info.getValue() ? "active" : "passive"} />
      }),
      departmentColumn.accessor("updated_at", { header: "Güncelleme", cell: (info) => formatDateTime(info.getValue()) }),
      departmentColumn.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => {
          const row = info.row.original;
          return (
            <div className="flex justify-end gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  setDepartmentForm({
                    id: String(row.id),
                    company_id: row.company_id ? String(row.company_id) : "",
                    name: row.name,
                    is_active: String(row.is_active)
                  })
                }
              >
                <Pencil size={14} />
                Düzenle
              </Button>
              <Button
                size="sm"
                variant={row.is_active ? "danger" : "secondary"}
                onClick={() => departmentStatusMutation.mutate({ id: row.id, is_active: !row.is_active })}
              >
                <Power size={14} />
                {row.is_active ? "Pasif" : "Aktif"}
              </Button>
            </div>
          );
        }
      })
    ],
    [companies, departmentStatusMutation]
  );

  const firstError = companiesQuery.error ?? departmentsQuery.error;
  if (companiesQuery.isLoading || departmentsQuery.isLoading) return <LoadingBlock />;
  if (firstError) return <ErrorBlock error={firstError} />;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Organizasyon</h1>
        <p className="mt-1 text-sm text-muted-foreground">Şirket ve departman tanımları.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        <MetricCard icon={Building2} label="Şirketler" value={companies.length} helper="Aktif ve pasif kayıtlar" />
        <MetricCard icon={Rows3} label="Departmanlar" value={departments.length} helper="Şirket ilişkili birimler" />
      </div>

      <Panel title={companyForm.id ? "Şirket Düzenle" : "Yeni Şirket"}>
        <form
          className="grid gap-3 md:grid-cols-[1fr_220px_160px_auto]"
          onSubmit={(event) => {
            event.preventDefault();
            companyMutation.mutate();
          }}
        >
          <Input required value={companyForm.name} onChange={(event) => setCompanyForm((current) => ({ ...current, name: event.target.value }))} placeholder="Şirket adı" />
          <Input value={companyForm.tax_number} onChange={(event) => setCompanyForm((current) => ({ ...current, tax_number: event.target.value }))} placeholder="Vergi no" />
          <select
            className="h-10 rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
            value={companyForm.is_active}
            onChange={(event) => setCompanyForm((current) => ({ ...current, is_active: event.target.value }))}
          >
            <option value="true">Aktif</option>
            <option value="false">Pasif</option>
          </select>
          <div className="flex gap-2">
            {companyForm.id && (
              <Button type="button" variant="secondary" onClick={() => setCompanyForm({ id: "", name: "", tax_number: "", is_active: "true" })}>
                Vazgeç
              </Button>
            )}
            <Button type="submit" disabled={companyMutation.isPending}>
              Kaydet
            </Button>
          </div>
        </form>
        {companyMutation.isError && <ErrorLine error={companyMutation.error} />}
      </Panel>

      <Panel title={departmentForm.id ? "Departman Düzenle" : "Yeni Departman"}>
        <form
          className="grid gap-3 md:grid-cols-[240px_1fr_160px_auto]"
          onSubmit={(event) => {
            event.preventDefault();
            departmentMutation.mutate();
          }}
        >
          <select
            className="h-10 rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
            value={departmentForm.company_id}
            onChange={(event) => setDepartmentForm((current) => ({ ...current, company_id: event.target.value }))}
          >
            <option value="">Şirket yok</option>
            {companies.map((company) => (
              <option key={company.id} value={company.id}>
                {company.name}
              </option>
            ))}
          </select>
          <Input required value={departmentForm.name} onChange={(event) => setDepartmentForm((current) => ({ ...current, name: event.target.value }))} placeholder="Departman adı" />
          <select
            className="h-10 rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
            value={departmentForm.is_active}
            onChange={(event) => setDepartmentForm((current) => ({ ...current, is_active: event.target.value }))}
          >
            <option value="true">Aktif</option>
            <option value="false">Pasif</option>
          </select>
          <div className="flex gap-2">
            {departmentForm.id && (
              <Button type="button" variant="secondary" onClick={() => setDepartmentForm({ id: "", company_id: "", name: "", is_active: "true" })}>
                Vazgeç
              </Button>
            )}
            <Button type="submit" disabled={departmentMutation.isPending}>
              Kaydet
            </Button>
          </div>
        </form>
        {departmentMutation.isError && <ErrorLine error={departmentMutation.error} />}
      </Panel>

      <Panel title="Şirket Listesi">
        <DataTable data={companies} columns={companyColumns} emptyText="Şirket kaydı yok." />
      </Panel>

      <Panel title="Departman Listesi">
        <DataTable data={departments} columns={departmentColumns} emptyText="Departman kaydı yok." />
      </Panel>
    </div>
  );
}

function companyName(companies: CompanyRow[], id: number | null) {
  if (!id) return "-";
  return companies.find((company) => company.id === id)?.name ?? `#${id}`;
}

function ErrorLine({ error }: { error: unknown }) {
  return (
    <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
      {error instanceof Error ? error.message : "İşlem tamamlanamadı"}
    </div>
  );
}
