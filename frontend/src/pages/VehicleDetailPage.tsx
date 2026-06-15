import { Bot, ClipboardList, FileArchive, Gauge, ReceiptText, Save, ShieldCheck, Trash2, UserRound, Wrench } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { appConfirm } from "../components/AppDialog";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { AssignmentRow, KmLogRow, UserRow, VehicleSellInput, VehicleUpdateInput } from "../lib/types";
import { formatDate, formatDateTime, formatMoney, humanize } from "../lib/utils";

export function VehicleDetailPage() {
  const { id = "" } = useParams();
  const queryClient = useQueryClient();
  const [editOpen, setEditOpen] = useState(false);
  const [sellOpen, setSellOpen] = useState(false);
  const [editValues, setEditValues] = useState<Record<string, string>>({});
  const [kmValues, setKmValues] = useState<Record<string, string>>({ km: "", entry_type: "manual", device_info: "" });
  const [assignValues, setAssignValues] = useState<Record<string, string>>({ user_id: "", note: "" });
  const [sellValues, setSellValues] = useState<Record<string, string>>({
    sold_date: new Date().toISOString().slice(0, 10),
    sold_reason: "satis",
    company_exit_reason: "satis"
  });
  const query = useQuery({
    queryKey: ["vehicle-profile", id],
    queryFn: () => api.vehicleProfile(id),
    enabled: Boolean(id)
  });
  const kmLogsQuery = useQuery({
    queryKey: ["vehicle-km-logs", id],
    queryFn: () => api.vehicleKmLogs(id),
    enabled: Boolean(id)
  });
  const assignmentsQuery = useQuery({
    queryKey: ["vehicle-assignments", id],
    queryFn: () => api.vehicleAssignments(id),
    enabled: Boolean(id)
  });
  const usersQuery = useQuery({
    queryKey: ["users"],
    queryFn: api.users
  });

  const aiMutation = useMutation({
    mutationFn: () => api.createVehicleAiAnalysis(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["ai-jobs"] });
    }
  });

  const updateMutation = useMutation({
    mutationFn: (payload: VehicleUpdateInput) => api.updateVehicle(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
      setEditOpen(false);
    }
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteVehicle(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    }
  });

  const sellMutation = useMutation({
    mutationFn: (payload: VehicleSellInput) => api.sellVehicle(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setSellOpen(false);
    }
  });
  const createKmMutation = useMutation({
    mutationFn: () =>
      api.createKmLog({
        vehicle_id: Number(id),
        km: Number(kmValues.km),
        entry_type: kmValues.entry_type || "manual",
        device_info: kmValues.device_info || undefined
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-km-logs", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setKmValues({ km: "", entry_type: "manual", device_info: "" });
    }
  });
  const verifyKmMutation = useMutation({
    mutationFn: ({ logId, verification_status }: { logId: number; verification_status: string }) =>
      api.verifyKmLog(logId, { verification_status }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-km-logs", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
    }
  });
  const assignMutation = useMutation({
    mutationFn: () =>
      api.assignVehicle({
        vehicle_id: Number(id),
        user_id: Number(assignValues.user_id),
        note: assignValues.note || undefined
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-assignments", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
      setAssignValues({ user_id: "", note: "" });
    }
  });
  const releaseMutation = useMutation({
    mutationFn: ({ assignmentId, note }: { assignmentId: number; note?: string }) =>
      api.releaseAssignment(assignmentId, { note }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["vehicle-assignments", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicle-profile", id] });
      await queryClient.invalidateQueries({ queryKey: ["vehicles"] });
    }
  });

  useEffect(() => {
    const vehicle = query.data?.vehicle;
    if (!vehicle) return;
    setEditValues({
      brand: vehicle.brand ?? "",
      model: vehicle.model ?? "",
      model_year: vehicle.model_year ? String(vehicle.model_year) : "",
      vehicle_type: vehicle.vehicle_type ?? "",
      fuel_type: vehicle.fuel_type ?? "",
      transmission: vehicle.transmission ?? "",
      chassis_no: vehicle.chassis_no ?? "",
      engine_no: vehicle.engine_no ?? "",
      warranty_status: vehicle.warranty_status ?? "",
      warranty_end: vehicle.warranty_end ?? "",
      tasitmatik_company: vehicle.tasitmatik_company ?? "",
      spare_key_location: vehicle.spare_key_location ?? "",
      has_hgs: String(vehicle.has_hgs),
      has_mobiliz: String(vehicle.has_mobiliz),
      has_kopilot: String(vehicle.has_kopilot),
      has_k2: String(vehicle.has_k2)
    });
  }, [query.data]);

  if (query.isLoading) return <LoadingBlock />;
  if (query.isError) return <ErrorBlock error={query.error} />;
  const profile = query.data!;
  const vehicle = profile.vehicle;
  const kmLogs = kmLogsQuery.data ?? profile.recent_km_logs;
  const assignments = assignmentsQuery.data ?? [];
  const users = usersQuery.data ?? [];

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <h1 className="text-xl font-semibold">{vehicle.plate}</h1>
            <StatusBadge value={vehicle.status} />
          </div>
          <p className="mt-1 text-sm text-muted-foreground">
            {vehicle.brand} {vehicle.model} {vehicle.model_year ?? ""}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="secondary" onClick={() => setEditOpen((value) => !value)}>
            Düzenle
          </Button>
          <Button variant="secondary" onClick={() => setSellOpen((value) => !value)} disabled={vehicle.status === "sold"}>
            Satışa Çıkar
          </Button>
          <Button onClick={() => aiMutation.mutate()} disabled={aiMutation.isPending}>
            <Bot size={17} />
            AI Analiz Başlat
          </Button>
        </div>
      </div>

      {editOpen && (
        <Panel title="Araç Düzenle">
          <form
            className="space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              updateMutation.mutate(buildVehicleUpdate(editValues));
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              {[
                ["brand", "Marka"],
                ["model", "Model"],
                ["model_year", "Model Yılı", "number"],
                ["vehicle_type", "Araç Tipi"],
                ["fuel_type", "Yakıt Tipi"],
                ["transmission", "Vites"],
                ["chassis_no", "Şasi No"],
                ["engine_no", "Motor No"],
                ["warranty_status", "Garanti Durumu"],
                ["warranty_end", "Garanti Bitişi", "date"],
                ["tasitmatik_company", "Taşıtmatik Firma"],
                ["spare_key_location", "Yedek Anahtar"]
              ].map(([key, label, type]) => (
                <label key={key} className="block">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
                  <Input
                    type={type ?? "text"}
                    value={editValues[key] ?? ""}
                    onChange={(event) => setEditValues((current) => ({ ...current, [key]: event.target.value }))}
                  />
                </label>
              ))}
            </div>
            <div className="grid gap-2 sm:grid-cols-4">
              {[
                ["has_hgs", "HGS"],
                ["has_mobiliz", "Mobiliz"],
                ["has_kopilot", "Kopilot"],
                ["has_k2", "K2"]
              ].map(([key, label]) => (
                <label key={key} className="flex h-10 items-center gap-2 rounded-md border border-border bg-background px-3 text-sm">
                  <input
                    type="checkbox"
                    checked={editValues[key] === "true"}
                    onChange={(event) => setEditValues((current) => ({ ...current, [key]: String(event.target.checked) }))}
                  />
                  {label}
                </label>
              ))}
            </div>
            {updateMutation.isError && (
              <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {updateMutation.error instanceof Error ? updateMutation.error.message : "Araç güncellenemedi"}
              </div>
            )}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setEditOpen(false)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                <Save size={17} />
                Kaydet
              </Button>
            </div>
          </form>
        </Panel>
      )}

      {sellOpen && (
        <Panel title="Araç Satış / Çıkış">
          <form
            className="space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              sellMutation.mutate(buildSellPayload(sellValues));
            }}
          >
            <div className="grid gap-3 md:grid-cols-3">
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Satış Tarihi</span>
                <Input
                  required
                  type="date"
                  value={sellValues.sold_date ?? ""}
                  onChange={(event) => setSellValues((current) => ({ ...current, sold_date: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Satış Nedeni</span>
                <Input
                  required
                  value={sellValues.sold_reason ?? ""}
                  onChange={(event) => setSellValues((current) => ({ ...current, sold_reason: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Çıkış Nedeni</span>
                <Input
                  required
                  value={sellValues.company_exit_reason ?? ""}
                  onChange={(event) => setSellValues((current) => ({ ...current, company_exit_reason: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Satış Tutarı</span>
                <Input
                  type="number"
                  value={sellValues.sold_price ?? ""}
                  onChange={(event) => setSellValues((current) => ({ ...current, sold_price: event.target.value }))}
                />
              </label>
              <label className="block md:col-span-2">
                <span className="mb-1 block text-xs font-medium text-muted-foreground">Alıcı Bilgisi</span>
                <Input
                  value={sellValues.buyer_info ?? ""}
                  onChange={(event) => setSellValues((current) => ({ ...current, buyer_info: event.target.value }))}
                />
              </label>
            </div>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Not</span>
              <Input
                value={sellValues.note ?? ""}
                onChange={(event) => setSellValues((current) => ({ ...current, note: event.target.value }))}
              />
            </label>
            {sellMutation.isError && (
              <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                {sellMutation.error instanceof Error ? sellMutation.error.message : "Satış kaydı oluşturulamadı"}
              </div>
            )}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="secondary" onClick={() => setSellOpen(false)}>
                Vazgeç
              </Button>
              <Button type="submit" disabled={sellMutation.isPending}>
                Satışı Kaydet
              </Button>
            </div>
          </form>
        </Panel>
      )}

      {aiMutation.isSuccess && (
        <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          AI analiz işi oluşturuldu. AI ekranından job durumunu takip edebilirsin.
        </div>
      )}
      {aiMutation.isError && (
        <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {aiMutation.error instanceof Error ? aiMutation.error.message : "AI analiz başlatılamadı"}
        </div>
      )}
      {deleteMutation.isError && (
        <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {deleteMutation.error instanceof Error ? deleteMutation.error.message : "Araç arşivlenemedi"}
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard icon={Gauge} label="Son KM" value={profile.km_summary.last_km ?? "-"} helper={`${profile.km_summary.total_logs} KM kaydı`} />
        <MetricCard icon={Wrench} label="Bakım maliyeti" value={formatMoney(profile.maintenance_summary.total_cost)} helper={`${profile.maintenance_summary.total_records} bakım kaydı`} />
        <MetricCard icon={ReceiptText} label="Toplam gider" value={formatMoney(profile.expense_summary.total_amount)} helper={`${profile.expense_summary.total_records} gider kaydı`} />
        <MetricCard icon={ClipboardList} label="Açık operasyon" value={profile.operations_summary.open_inspection_count + profile.operations_summary.open_value_loss_count} helper={`${profile.operations_summary.fuel_record_count} yakıt kaydı`} />
      </div>

      <div className="grid gap-5 xl:grid-cols-[1fr_420px]">
        <Panel title="Araç Bilgileri">
          <dl className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {[
              ["Şasi No", vehicle.chassis_no],
              ["Motor No", vehicle.engine_no],
              ["Yakıt", humanize(vehicle.fuel_type)],
              ["Vites", humanize(vehicle.transmission)],
              ["Garanti", `${humanize(vehicle.warranty_status)} / ${formatDate(vehicle.warranty_end)}`],
              ["Yedek Anahtar", vehicle.spare_key_location],
              ["Taşıtmatik", vehicle.tasitmatik_company],
              ["HGS", vehicle.has_hgs ? "Var" : "Yok"],
              ["Mobiliz", vehicle.has_mobiliz ? "Var" : "Yok"],
              ["Kopilot", vehicle.has_kopilot ? "Var" : "Yok"],
              ["K2", vehicle.has_k2 ? "Var" : "Yok"]
            ].map(([label, value]) => (
              <div key={label} className="rounded-md border border-border bg-background px-3 py-2">
                <dt className="text-xs text-muted-foreground">{label}</dt>
                <dd className="mt-1 text-sm font-medium">{value || "-"}</dd>
              </div>
            ))}
          </dl>
        </Panel>

        <Panel title="Dosya / Poliçe / Hasar">
          <div className="grid gap-3">
            <MetricCard icon={ShieldCheck} label="Aktif poliçe" value={profile.policy_summary.active_policies} helper={`Son bitiş: ${formatDate(profile.policy_summary.latest_end_date)}`} />
            <MetricCard icon={FileArchive} label="Dosya sayısı" value={profile.file_summary.total_files} helper={`${profile.file_summary.expense_files} gider, ${profile.file_summary.damage_files} hasar dosyası`} />
            <MetricCard icon={Bot} label="AI analiz uygunluğu" value={profile.km_summary.suspicious_logs > 0 ? "İncelenmeli" : "Normal"} helper={`${profile.km_summary.suspicious_logs} şüpheli KM kaydı`} tone={profile.km_summary.suspicious_logs > 0 ? "warning" : "success"} />
          </div>
        </Panel>
      </div>

      <Panel title="Tehlikeli İşlem">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold">Aracı arşivle</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              Bu işlem aracı soft delete olarak arşivler. Satış akışı için “Satışa Çıkar” kullanılmalıdır.
            </p>
          </div>
          <Button
            variant="danger"
            onClick={async () => {
              const confirmed = await appConfirm(`${vehicle.plate} aracını arşivlemek istiyor musun?`, {
                title: "Araç arşivle",
                confirmLabel: "Arşivle",
                tone: "danger"
              });
              if (confirmed) deleteMutation.mutate();
            }}
            disabled={deleteMutation.isPending || vehicle.status === "sold"}
          >
            <Trash2 size={17} />
            Arşivle
          </Button>
        </div>
      </Panel>

      <div className="grid gap-5 xl:grid-cols-2">
        <Panel title="Son Görevler">
          <div className="divide-y divide-border">
            {profile.recent_tasks.map((task) => (
              <div key={task.id} className="py-3">
                <div className="flex items-center justify-between gap-3">
                  <div className="text-sm font-medium">{humanize(task.task_type)}</div>
                  <StatusBadge value={task.task_status} />
                </div>
                <div className="mt-1 text-xs text-muted-foreground">
                  {humanize(task.priority)} / Son tarih: {formatDate(task.due_date)}
                </div>
              </div>
            ))}
            {profile.recent_tasks.length === 0 && <div className="py-6 text-center text-sm text-muted-foreground">Görev kaydı yok.</div>}
          </div>
        </Panel>

        <Panel title="KM Yönetimi">
          <form
            className="mb-4 grid gap-3 md:grid-cols-[1fr_180px_1fr_auto]"
            onSubmit={(event) => {
              event.preventDefault();
              createKmMutation.mutate();
            }}
          >
            <Input
              required
              min="0"
              type="number"
              value={kmValues.km}
              onChange={(event) => setKmValues((current) => ({ ...current, km: event.target.value }))}
              placeholder="KM"
            />
            <select
              className="h-10 rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
              value={kmValues.entry_type}
              onChange={(event) => setKmValues((current) => ({ ...current, entry_type: event.target.value }))}
            >
              <option value="manual">Manuel</option>
              <option value="ocr">OCR</option>
              <option value="mobiliz_api">Mobiliz API</option>
              <option value="kopilot_api">Kopilot API</option>
            </select>
            <Input
              value={kmValues.device_info}
              onChange={(event) => setKmValues((current) => ({ ...current, device_info: event.target.value }))}
              placeholder="Cihaz / kaynak"
            />
            <Button type="submit" disabled={createKmMutation.isPending || vehicle.status === "sold"}>
              Kaydet
            </Button>
          </form>
          {createKmMutation.isError && <ErrorLine error={createKmMutation.error} />}
          <div className="divide-y divide-border">
            {kmLogs.map((log) => (
              <div key={log.id} className="flex items-center justify-between gap-3 py-3">
                <div>
                  <div className="text-sm font-medium">{log.km.toLocaleString("tr-TR")} KM</div>
                  <div className="text-xs text-muted-foreground">
                    {humanize(log.entry_type)} / {formatDateTime(log.created_at)}
                  </div>
                </div>
                <div className="flex flex-wrap items-center justify-end gap-2">
                  <StatusBadge value={log.verification_status} />
                  <Button
                    size="sm"
                    variant="secondary"
                    disabled={verifyKmMutation.isPending}
                    onClick={() => verifyKmMutation.mutate({ logId: log.id, verification_status: "verified" })}
                  >
                    Onayla
                  </Button>
                  <Button
                    size="sm"
                    variant="danger"
                    disabled={verifyKmMutation.isPending}
                    onClick={() => verifyKmMutation.mutate({ logId: log.id, verification_status: "rejected" })}
                  >
                    Reddet
                  </Button>
                </div>
              </div>
            ))}
            {kmLogs.length === 0 && <div className="py-6 text-center text-sm text-muted-foreground">KM kaydı yok.</div>}
          </div>
        </Panel>
      </div>

      <Panel title="Kullanıcı Atama">
        <form
          className="mb-4 grid gap-3 md:grid-cols-[280px_1fr_auto]"
          onSubmit={(event) => {
            event.preventDefault();
            assignMutation.mutate();
          }}
        >
          <select
            required
            className="h-10 rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
            value={assignValues.user_id}
            onChange={(event) => setAssignValues((current) => ({ ...current, user_id: event.target.value }))}
          >
            <option value="">Kullanıcı seç</option>
            {users
              .filter((user) => user.is_active)
              .map((user) => (
                <option key={user.id} value={user.id}>
                  {user.full_name} / {user.role}
                </option>
              ))}
          </select>
          <Input
            value={assignValues.note}
            onChange={(event) => setAssignValues((current) => ({ ...current, note: event.target.value }))}
            placeholder="Atama notu"
          />
          <Button type="submit" disabled={assignMutation.isPending || vehicle.status === "sold"}>
            <UserRound size={17} />
            Ata
          </Button>
        </form>
        {assignMutation.isError && <ErrorLine error={assignMutation.error} />}
        {releaseMutation.isError && <ErrorLine error={releaseMutation.error} />}
        <div className="divide-y divide-border">
          {assignments.map((assignment) => (
            <AssignmentLine
              key={assignment.id}
              assignment={assignment}
              users={users}
              onRelease={async () => {
                const confirmed = await appConfirm(`#${assignment.id} ataması kapatılsın mı?`, {
                  title: "Atamayı kapat",
                  confirmLabel: "Kapat"
                });
                if (confirmed) releaseMutation.mutate({ assignmentId: assignment.id, note: "Frontend araç detayından kapatıldı" });
              }}
              releasePending={releaseMutation.isPending}
            />
          ))}
          {assignments.length === 0 && <div className="py-6 text-center text-sm text-muted-foreground">Atama kaydı yok.</div>}
        </div>
      </Panel>
    </div>
  );
}

function AssignmentLine({
  assignment,
  users,
  onRelease,
  releasePending
}: {
  assignment: AssignmentRow;
  users: UserRow[];
  onRelease: () => void;
  releasePending: boolean;
}) {
  const user = users.find((item) => item.id === assignment.user_id);
  const active = !assignment.released_at;
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 py-3">
      <div>
        <div className="text-sm font-medium">{user?.full_name ?? `Kullanıcı #${assignment.user_id}`}</div>
        <div className="text-xs text-muted-foreground">
          Atama: {formatDateTime(assignment.assigned_at)} / Çıkış: {formatDateTime(assignment.released_at)}
        </div>
        {assignment.note && <div className="mt-1 text-xs text-muted-foreground">{assignment.note}</div>}
      </div>
      <div className="flex items-center gap-2">
        <StatusBadge value={active ? "active" : "released"} />
        {active && (
          <Button size="sm" variant="secondary" disabled={releasePending} onClick={onRelease}>
            Çıkış Ver
          </Button>
        )}
      </div>
    </div>
  );
}

function ErrorLine({ error }: { error: unknown }) {
  return (
    <div className="mb-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
      {error instanceof Error ? error.message : "İşlem tamamlanamadı"}
    </div>
  );
}

function buildVehicleUpdate(values: Record<string, string>): VehicleUpdateInput {
  const payload: VehicleUpdateInput = {};
  for (const [key, value] of Object.entries(values)) {
    if (value === "") continue;
    if (key === "model_year") {
      payload.model_year = Number(value);
    } else if (["has_hgs", "has_mobiliz", "has_kopilot", "has_k2"].includes(key)) {
      (payload as Record<string, unknown>)[key] = value === "true";
    } else {
      (payload as Record<string, unknown>)[key] = value;
    }
  }
  return payload;
}

function buildSellPayload(values: Record<string, string>): VehicleSellInput {
  const payload: VehicleSellInput = {
    sold_date: values.sold_date,
    sold_reason: values.sold_reason,
    company_exit_reason: values.company_exit_reason
  };
  if (values.sold_price) payload.sold_price = Number(values.sold_price);
  if (values.buyer_info) payload.buyer_info = values.buyer_info;
  if (values.note) payload.note = values.note;
  return payload;
}
