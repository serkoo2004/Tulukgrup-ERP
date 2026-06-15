import { createColumnHelper } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil, Plus, Shield, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { appConfirm } from "../components/AppDialog";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { MobilePermissionInput, UserCreateInput, UserRow, UserUpdateInput } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";

const columnHelper = createColumnHelper<UserRow>();
const fallbackRoles = [
  { key: "admin", label: "Admin" },
  { key: "manager", label: "Yönetici" },
  { key: "operation", label: "Operasyon" },
  { key: "accounting", label: "Muhasebe" },
  { key: "user", label: "Kullanıcı" }
];
const mobilePermissionKeys = [
  { key: "dashboard", label: "Mobil Panel" },
  { key: "inventory_view", label: "Stok Görüntüleme" },
  { key: "inventory_order", label: "Ürün Siparişi" },
  { key: "vehicle_view", label: "Araç Görüntüleme" },
  { key: "vehicle_fault", label: "Araç Arıza Bildirimi" },
  { key: "km_log", label: "KM Girişi" },
  { key: "support_ticket", label: "Destek Talebi" },
  { key: "media_upload", label: "Foto/Video Yükleme" }
];

export function UsersPage() {
  const [formOpen, setFormOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserRow | null>(null);
  const [values, setValues] = useState<Record<string, string>>({
    role: "user",
    password: "ChangeMe123!"
  });
  const [permissionValues, setPermissionValues] = useState<Record<string, string | boolean>>({
    permission_key: "dashboard",
    can_view: true,
    can_create: false,
    can_update: false,
    can_approve: false
  });
  const queryClient = useQueryClient();
  const query = useQuery({ queryKey: ["users"], queryFn: api.users });
  const rolesQuery = useQuery({ queryKey: ["roles"], queryFn: api.roles });
  const companiesQuery = useQuery({ queryKey: ["companies"], queryFn: api.companies });
  const departmentsQuery = useQuery({ queryKey: ["departments"], queryFn: api.departments });
  const mobilePermissionsQuery = useQuery({
    queryKey: ["user-mobile-permissions", editingUser?.id],
    queryFn: () => api.userMobilePermissions(editingUser!.id),
    enabled: Boolean(editingUser)
  });
  const roles = rolesQuery.data?.length ? rolesQuery.data : fallbackRoles;
  const createMutation = useMutation({
    mutationFn: (payload: UserCreateInput) => api.createUser(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["users"] });
      setValues({ role: "user", password: "ChangeMe123!" });
      setPermissionValues(defaultMobilePermissionValues());
      setFormOpen(false);
    }
  });
  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: UserUpdateInput }) => api.updateUser(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["users"] });
      resetForm();
    }
  });
  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.deleteUser(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["users"] });
    }
  });
  const mobilePermissionMutation = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: MobilePermissionInput }) =>
      api.upsertUserMobilePermission(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["user-mobile-permissions", editingUser?.id] });
    }
  });
  const columns = useMemo(
    () => [
      columnHelper.accessor("full_name", {
        header: "Ad Soyad",
        cell: (info) => <span className="font-medium">{info.getValue()}</span>
      }),
      columnHelper.accessor("email", {
        header: "E-posta"
      }),
      columnHelper.accessor("role", {
        header: "Rol",
        cell: (info) => humanize(info.getValue())
      }),
      columnHelper.accessor("phone", {
        header: "Telefon",
        cell: (info) => info.getValue() ?? "-"
      }),
      columnHelper.accessor("is_active", {
        header: "Durum",
        cell: (info) => <StatusBadge value={info.getValue() ? "active" : "passive"} />
      }),
      columnHelper.accessor("updated_at", {
        header: "Güncelleme",
        cell: (info) => formatDateTime(info.getValue())
      }),
      columnHelper.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => (
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                const row = info.row.original;
                setEditingUser(row);
                setFormOpen(true);
                setValues({
                  full_name: row.full_name,
                  role: row.role,
                  phone: row.phone ?? "",
                  extension: row.extension ?? "",
                  company_id: row.company_id ? String(row.company_id) : "",
                  department_id: row.department_id ? String(row.department_id) : "",
                  is_active: String(row.is_active)
                });
                setPermissionValues(defaultMobilePermissionValues());
              }}
            >
              <Pencil size={14} />
              Düzenle
            </Button>
            <Button
              size="sm"
              variant="danger"
              onClick={async () => {
                const confirmed = await appConfirm(`${info.row.original.full_name} pasife alınsın mı?`, {
                  title: "Kullanıcıyı pasife al",
                  confirmLabel: "Pasife al",
                  tone: "danger"
                });
                if (confirmed) deleteMutation.mutate(info.row.original.id);
              }}
              disabled={deleteMutation.isPending}
            >
              <Trash2 size={14} />
              Pasife Al
            </Button>
          </div>
        )
      })
    ],
    [deleteMutation]
  );

  const resetForm = () => {
    setValues({ role: "user", password: "ChangeMe123!" });
    setPermissionValues(defaultMobilePermissionValues());
    setEditingUser(null);
    setFormOpen(false);
  };

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold">Kullanıcılar</h1>
          <p className="mt-1 text-sm text-muted-foreground">Rol bazlı yetki ve aktif/pasif kullanıcı görünümü.</p>
        </div>
        <Button
          onClick={() => {
            if (formOpen && !editingUser) {
              resetForm();
            } else {
              setEditingUser(null);
              setValues({ role: "user", password: "ChangeMe123!" });
              setPermissionValues(defaultMobilePermissionValues());
              setFormOpen(true);
            }
          }}
        >
          <Plus size={17} />
          Yeni Kullanıcı
        </Button>
      </div>
      {formOpen && (
        <Panel title={editingUser ? "Kullanıcı Düzenle" : "Yeni Kullanıcı"}>
          <form
            className="space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              if (editingUser) {
                updateMutation.mutate({ id: editingUser.id, payload: buildUserUpdatePayload(values) });
              } else {
                createMutation.mutate(buildUserPayload(values));
              }
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              <UserField label="Ad Soyad" name="full_name" required values={values} setValues={setValues} />
              {!editingUser && (
                <>
                  <UserField label="E-posta" name="email" required type="email" values={values} setValues={setValues} />
                  <UserField label="Şifre" name="password" required type="password" values={values} setValues={setValues} />
                </>
              )}
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Rol</span>
                <select
                  className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                  value={values.role ?? "user"}
                  onChange={(event) => setValues((current) => ({ ...current, role: event.target.value }))}
                >
                  {roles.map((role) => (
                    <option key={role.key} value={role.key}>
                      {role.label}
                    </option>
                  ))}
                </select>
              </label>
              <UserField label="Telefon" name="phone" values={values} setValues={setValues} />
              <UserField label="Dahili" name="extension" values={values} setValues={setValues} />
              <UserField label="Şirket ID" name="company_id" type="number" values={values} setValues={setValues} />
              <UserField label="Departman ID" name="department_id" type="number" values={values} setValues={setValues} />
              {editingUser && (
                <label className="block">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">Durum</span>
                  <select
                    className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                    value={values.is_active ?? "true"}
                    onChange={(event) => setValues((current) => ({ ...current, is_active: event.target.value }))}
                  >
                    <option value="true">Aktif</option>
                    <option value="false">Pasif</option>
                  </select>
                </label>
              )}
            </div>
            {(createMutation.isError || updateMutation.isError) && (
              <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {(createMutation.error ?? updateMutation.error) instanceof Error
                  ? ((createMutation.error ?? updateMutation.error) as Error).message
                  : "Kullanıcı kaydedilemedi"}
              </div>
            )}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={resetForm}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
                Kaydet
              </Button>
            </div>
          </form>
        </Panel>
      )}
      {formOpen && editingUser && (
        <Panel title="Mobil Uygulama Yetkileri">
          <div className="mb-4 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-muted-foreground">
            Bu alan mobil uygulamadaki ekran ve işlem yetkilerini belirler. Şirket/departman boş bırakılırsa yetki global çalışır.
          </div>
          <form
            className="grid gap-3 lg:grid-cols-6"
            onSubmit={(event) => {
              event.preventDefault();
              mobilePermissionMutation.mutate({
                id: editingUser.id,
                payload: buildMobilePermissionPayload(permissionValues)
              });
            }}
          >
            <label className="block lg:col-span-2">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Yetki</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={String(permissionValues.permission_key ?? "dashboard")}
                onChange={(event) =>
                  setPermissionValues((current) => ({ ...current, permission_key: event.target.value }))
                }
              >
                {mobilePermissionKeys.map((permission) => (
                  <option key={permission.key} value={permission.key}>
                    {permission.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Şirket</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={String(permissionValues.company_id ?? "")}
                onChange={(event) =>
                  setPermissionValues((current) => ({ ...current, company_id: event.target.value }))
                }
              >
                <option value="">Global</option>
                {(companiesQuery.data ?? []).map((company) => (
                  <option key={company.id} value={company.id}>
                    {company.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Departman</span>
              <select
                className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                value={String(permissionValues.department_id ?? "")}
                onChange={(event) =>
                  setPermissionValues((current) => ({ ...current, department_id: event.target.value }))
                }
              >
                <option value="">Global</option>
                {(departmentsQuery.data ?? []).map((department) => (
                  <option key={department.id} value={department.id}>
                    {department.name}
                  </option>
                ))}
              </select>
            </label>
            <div className="flex flex-wrap items-end gap-3 lg:col-span-2">
              <PermissionCheckbox label="Gör" name="can_view" values={permissionValues} setValues={setPermissionValues} />
              <PermissionCheckbox label="Ekle" name="can_create" values={permissionValues} setValues={setPermissionValues} />
              <PermissionCheckbox label="Düzenle" name="can_update" values={permissionValues} setValues={setPermissionValues} />
              <PermissionCheckbox label="Onayla" name="can_approve" values={permissionValues} setValues={setPermissionValues} />
              <Button type="submit" disabled={mobilePermissionMutation.isPending}>
                <Shield size={16} />
                Yetki Kaydet
              </Button>
            </div>
          </form>
          {mobilePermissionMutation.isError && (
            <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {mobilePermissionMutation.error instanceof Error
                ? mobilePermissionMutation.error.message
                : "Mobil yetki kaydedilemedi"}
            </div>
          )}
          <div className="mt-4 overflow-x-auto">
            <table className="w-full min-w-[760px] text-left text-sm">
              <thead className="border-b border-border text-xs uppercase text-muted-foreground">
                <tr>
                  <th className="py-2">Yetki</th>
                  <th>Şirket</th>
                  <th>Departman</th>
                  <th>Gör</th>
                  <th>Ekle</th>
                  <th>Düzenle</th>
                  <th>Onayla</th>
                </tr>
              </thead>
              <tbody>
                {mobilePermissionsQuery.isLoading ? (
                  <tr>
                    <td className="py-3 text-muted-foreground" colSpan={7}>Yetkiler yükleniyor...</td>
                  </tr>
                ) : (mobilePermissionsQuery.data ?? []).length === 0 ? (
                  <tr>
                    <td className="py-3 text-muted-foreground" colSpan={7}>Mobil yetki tanımı yok.</td>
                  </tr>
                ) : (
                  (mobilePermissionsQuery.data ?? []).map((permission) => (
                    <tr key={`${permission.permission_key}-${permission.company_id ?? "all"}-${permission.department_id ?? "all"}`} className="border-b border-border">
                      <td className="py-2 font-medium">{mobilePermissionLabel(permission.permission_key)}</td>
                      <td>{permission.company_name ?? "Global"}</td>
                      <td>{permission.department_name ?? "Global"}</td>
                      <td>{permission.can_view ? "Evet" : "Hayır"}</td>
                      <td>{permission.can_create ? "Evet" : "Hayır"}</td>
                      <td>{permission.can_update ? "Evet" : "Hayır"}</td>
                      <td>{permission.can_approve ? "Evet" : "Hayır"}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </Panel>
      )}
      <Panel title="Kullanıcı Listesi">
        {deleteMutation.isError && (
          <div className="mb-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
            {deleteMutation.error instanceof Error ? deleteMutation.error.message : "Kullanıcı pasife alınamadı"}
          </div>
        )}
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

function PermissionCheckbox({
  label,
  name,
  values,
  setValues
}: {
  label: string;
  name: string;
  values: Record<string, string | boolean>;
  setValues: React.Dispatch<React.SetStateAction<Record<string, string | boolean>>>;
}) {
  return (
    <label className="flex h-10 items-center gap-2 text-sm font-medium text-muted-foreground">
      <input
        type="checkbox"
        checked={Boolean(values[name])}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.checked }))}
      />
      {label}
    </label>
  );
}

function UserField({
  label,
  name,
  type = "text",
  required,
  values,
  setValues
}: {
  label: string;
  name: string;
  type?: string;
  required?: boolean;
  values: Record<string, string>;
  setValues: React.Dispatch<React.SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">
        {label}
        {required ? " *" : ""}
      </span>
      <Input
        required={required}
        type={type}
        value={values[name] ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))}
      />
    </label>
  );
}

function buildUserPayload(values: Record<string, string>): UserCreateInput {
  const payload: UserCreateInput = {
    email: values.email,
    full_name: values.full_name,
    password: values.password,
    role: values.role
  };
  if (values.phone) payload.phone = values.phone;
  if (values.extension) payload.extension = values.extension;
  if (values.company_id) payload.company_id = Number(values.company_id);
  if (values.department_id) payload.department_id = Number(values.department_id);
  return payload;
}

function buildUserUpdatePayload(values: Record<string, string>): UserUpdateInput {
  const payload: UserUpdateInput = {};
  if (values.full_name) payload.full_name = values.full_name;
  if (values.role) payload.role = values.role;
  if (values.phone) payload.phone = values.phone;
  if (values.extension) payload.extension = values.extension;
  if (values.company_id) payload.company_id = Number(values.company_id);
  if (values.department_id) payload.department_id = Number(values.department_id);
  if (values.is_active) payload.is_active = values.is_active === "true";
  return payload;
}

function buildMobilePermissionPayload(values: Record<string, string | boolean>): MobilePermissionInput {
  return {
    permission_key: String(values.permission_key ?? "dashboard"),
    company_id: values.company_id ? Number(values.company_id) : null,
    department_id: values.department_id ? Number(values.department_id) : null,
    can_view: Boolean(values.can_view),
    can_create: Boolean(values.can_create),
    can_update: Boolean(values.can_update),
    can_approve: Boolean(values.can_approve)
  };
}

function mobilePermissionLabel(key: string) {
  return mobilePermissionKeys.find((permission) => permission.key === key)?.label ?? key;
}

function defaultMobilePermissionValues(): Record<string, string | boolean> {
  return {
    permission_key: "dashboard",
    can_view: true,
    can_create: false,
    can_update: false,
    can_approve: false
  };
}
