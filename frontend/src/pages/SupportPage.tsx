import { createColumnHelper, type ColumnDef } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, BookOpen, Bot, CheckCircle2, Clock, FileUp, Headset, MessageSquare, Paperclip, RefreshCw, Trash2, Wrench } from "lucide-react";
import type { Dispatch, SetStateAction } from "react";
import { useMemo, useState } from "react";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api, uploadFile } from "../lib/api";
import type { DepartmentRow, FileDocumentRow, SupportKnowledgeBaseRow, SupportTicketEventRow, SupportTicketRow, UserRow } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";

const ticketColumn = createColumnHelper<SupportTicketRow>();
const eventColumn = createColumnHelper<SupportTicketEventRow>();
const kbColumn = createColumnHelper<SupportKnowledgeBaseRow>();

const priorities = ["low", "medium", "high", "critical"];
const statuses = ["open", "in_progress", "waiting_user", "resolved", "closed", "cancelled"];
const sources = ["web", "mobile", "whatsapp", "system"];
const supportFileTypes = [
  { value: "support_screenshot", label: "Ekran Görüntüsü" },
  { value: "support_video", label: "Destek Videosu" },
  { value: "support_log", label: "Log Dosyası" },
  { value: "support_document", label: "Destek Dokümanı" }
];

export function SupportPage() {
  const queryClient = useQueryClient();
  const [filters, setFilters] = useState<Record<string, string>>({ ticket_status: "open" });
  const [createValues, setCreateValues] = useState<Record<string, string>>({ priority: "medium", source_channel: "web", category: "it_support" });
  const [editValues, setEditValues] = useState<Record<string, string>>({});
  const [kbValues, setKbValues] = useState<Record<string, string>>({ category: "it_support", tags: "it,destek" });
  const [kbSearch, setKbSearch] = useState("");
  const [selectedTicket, setSelectedTicket] = useState<SupportTicketRow | null>(null);
  const [aiMessage, setAiMessage] = useState<string | null>(null);
  const [replyMessage, setReplyMessage] = useState("");
  const [replyStatus, setReplyStatus] = useState<string | null>(null);
  const [supportFileType, setSupportFileType] = useState("support_screenshot");
  const [supportFileNote, setSupportFileNote] = useState("");
  const [supportFile, setSupportFile] = useState<File | null>(null);
  const [supportFileInputKey, setSupportFileInputKey] = useState(0);

  const queryString = useMemo(() => buildQueryString({ ...filters, limit: "300" }), [filters]);
  const kbQueryString = useMemo(() => buildQueryString({ q: kbSearch, is_published: "true", limit: "50" }), [kbSearch]);
  const ticketsQuery = useQuery({ queryKey: ["support-tickets", queryString], queryFn: () => api.supportTickets(queryString) });
  const summaryQuery = useQuery({ queryKey: ["support-summary"], queryFn: api.supportSummary });
  const knowledgeQuery = useQuery({ queryKey: ["support-knowledge-base", kbQueryString], queryFn: () => api.supportKnowledgeBase(kbQueryString) });
  const usersQuery = useQuery({ queryKey: ["users", "support"], queryFn: () => api.users() });
  const departmentsQuery = useQuery({ queryKey: ["departments", "support"], queryFn: api.departments });
  const eventsQuery = useQuery({
    queryKey: ["support-ticket-events", selectedTicket?.id],
    queryFn: () => api.supportTicketEvents(selectedTicket!.id),
    enabled: selectedTicket !== null
  });
  const filesQuery = useQuery({
    queryKey: ["support-ticket-files", selectedTicket?.id],
    queryFn: () => api.files(`module_name=support_tickets&entity_id=${selectedTicket!.id}&limit=30`),
    enabled: selectedTicket !== null
  });

  const refresh = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["support-tickets"] }),
      queryClient.invalidateQueries({ queryKey: ["support-summary"] }),
      queryClient.invalidateQueries({ queryKey: ["support-ticket-events"] }),
      queryClient.invalidateQueries({ queryKey: ["support-ticket-files"] }),
      queryClient.invalidateQueries({ queryKey: ["support-knowledge-base"] }),
      queryClient.invalidateQueries({ queryKey: ["dashboard"] })
    ]);
  };

  const createMutation = useMutation({
    mutationFn: () => api.createSupportTicket(buildCreatePayload(createValues)),
    onSuccess: async () => {
      await refresh();
      setCreateValues({ priority: "medium", source_channel: "web", category: "it_support" });
    }
  });
  const updateMutation = useMutation({
    mutationFn: () => api.updateSupportTicket(selectedTicket!.id, buildUpdatePayload(editValues)),
    onSuccess: async (ticket) => {
      await refresh();
      setSelectedTicket(ticket);
      setEditValues(ticketToEditValues(ticket));
    }
  });
  const aiMutation = useMutation({
    mutationFn: (analysisType: string) => api.createSupportTicketAiAnalysis(selectedTicket!.id, analysisType),
    onSuccess: async (job) => {
      const label = job.analysis_type === "support_reply_draft" ? "AI cevap taslağı" : "AI analiz";
      setAiMessage(`${label} job #${job.id} oluşturuldu. Sonucu AI Yönetimi ekranından takip edebilirsiniz.`);
      await queryClient.invalidateQueries({ queryKey: ["ai-jobs"] });
    }
  });
  const replyMutation = useMutation({
    mutationFn: () =>
      api.replySupportTicket(selectedTicket!.id, {
        message: replyMessage,
        next_status: "waiting_user",
        event_note: "WhatsApp yanıtı hazırlandı"
      }),
    onSuccess: async (result) => {
      await refresh();
      setSelectedTicket(result.ticket);
      setEditValues(ticketToEditValues(result.ticket));
      setReplyStatus(`WhatsApp yanıt kaydı #${result.notification.id} oluşturuldu. Durum: ${humanize(result.notification.delivery_status)}.`);
    }
  });
  const kbMutation = useMutation({
    mutationFn: () => api.createSupportKnowledgeBase(buildKnowledgeBasePayload(kbValues, selectedTicket)),
    onSuccess: async () => {
      await refresh();
      setKbValues({ category: "it_support", tags: "it,destek" });
    }
  });
  const uploadSupportFileMutation = useMutation({
    mutationFn: () => {
      if (!selectedTicket) throw new Error("Talep seçilmedi");
      if (!supportFile) throw new Error("Dosya seçilmedi");
      return uploadFile({
        module_name: "support_tickets",
        entity_id: selectedTicket.id,
        file_type: supportFileType,
        note: supportFileNote,
        file: supportFile
      });
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["support-ticket-files", selectedTicket?.id] }),
        queryClient.invalidateQueries({ queryKey: ["files"] })
      ]);
      setSupportFile(null);
      setSupportFileNote("");
      setSupportFileInputKey((current) => current + 1);
    }
  });
  const deleteSupportFileMutation = useMutation({
    mutationFn: (id: string | number) => api.deleteFile(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["support-ticket-files", selectedTicket?.id] }),
        queryClient.invalidateQueries({ queryKey: ["files"] })
      ]);
    }
  });

  const tickets = ticketsQuery.data ?? [];
  const summary = summaryQuery.data;
  const knowledgeItems = knowledgeQuery.data ?? [];
  const supportFiles = filesQuery.data ?? [];

  const users = usersQuery.data ?? [];
  const departments = departmentsQuery.data ?? [];

  const columns = useMemo<ColumnDef<SupportTicketRow, any>[]>(
    () => [
      ticketColumn.accessor("ticket_no", { header: "Talep" }),
      ticketColumn.accessor("title", { header: "Başlık", cell: (info) => <span className="font-semibold text-primary">{info.getValue()}</span> }),
      ticketColumn.accessor("source_channel", { header: "Kaynak", cell: (info) => humanize(info.getValue()) }),
      ticketColumn.accessor("priority", { header: "Öncelik", cell: (info) => <StatusBadge value={info.getValue()} /> }),
      ticketColumn.accessor("ticket_status", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue()} /> }),
      ticketColumn.accessor("sla_status", { header: "SLA", cell: (info) => <StatusBadge value={info.getValue()} /> }),
      ticketColumn.accessor("sla_resolution_due_at", { header: "Çözüm Hedefi", cell: (info) => info.getValue() ? formatDateTime(info.getValue()) : "-" }),
      ticketColumn.accessor("reporter_phone", { header: "Telefon", cell: (info) => info.getValue() ?? "-" }),
      ticketColumn.accessor("assigned_user_name", { header: "Atanan", cell: (info) => info.getValue() ?? "-" }),
      ticketColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) }),
      ticketColumn.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => {
              setSelectedTicket(info.row.original);
              setEditValues(ticketToEditValues(info.row.original));
              setAiMessage(null);
              setReplyMessage(defaultReplyMessage(info.row.original));
              setReplyStatus(null);
            }}
          >
            Aç
          </Button>
        )
      })
    ],
    []
  );

  const eventColumns = [
    eventColumn.accessor("event_type", { header: "İşlem", cell: (info) => humanize(info.getValue()) }),
    eventColumn.accessor("note", { header: "Not", cell: (info) => info.getValue() ?? "-" }),
    eventColumn.accessor("old_status", { header: "Eski", cell: (info) => info.getValue() ?? "-" }),
    eventColumn.accessor("new_status", { header: "Yeni", cell: (info) => info.getValue() ?? "-" }),
    eventColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) })
  ];

  const kbColumns = [
    kbColumn.accessor("title", { header: "Başlık", cell: (info) => <span className="font-semibold text-primary">{info.getValue()}</span> }),
    kbColumn.accessor("category", { header: "Kategori", cell: (info) => humanize(info.getValue()) }),
    kbColumn.accessor("source_ticket_no", { header: "Kaynak", cell: (info) => info.getValue() ?? "-" }),
    kbColumn.accessor("view_count", { header: "Okunma" }),
    kbColumn.display({
      id: "solution",
      header: "Çözüm",
      cell: (info) => <span className="line-clamp-2 text-sm text-muted-foreground">{info.row.original.solution}</span>
    })
  ];

  const loading = ticketsQuery.isLoading || summaryQuery.isLoading || knowledgeQuery.isLoading || usersQuery.isLoading || departmentsQuery.isLoading;
  const error = ticketsQuery.error ?? summaryQuery.error ?? knowledgeQuery.error ?? usersQuery.error ?? departmentsQuery.error;
  if (loading) return <LoadingBlock />;
  if (error) return <ErrorBlock error={error} />;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">IT Destek</h1>
        <p className="mt-1 text-sm text-muted-foreground">Mobil, web ve WhatsApp üzerinden gelen destek taleplerinin takip ve çözüm ekranı.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-6">
        <MetricCard icon={Headset} label="Açık" value={summary?.open_count ?? 0} helper={`${summary?.unassigned_count ?? 0} atanmamış`} tone={(summary?.open_count ?? 0) > 0 ? "warning" : "success"} />
        <MetricCard icon={Wrench} label="İşlemde" value={summary?.in_progress_count ?? 0} helper={`${summary?.waiting_user_count ?? 0} kullanıcı bekliyor`} />
        <MetricCard icon={MessageSquare} label="WhatsApp" value={summary?.whatsapp_open_count ?? 0} helper="Açık WP kaynaklı" />
        <MetricCard icon={AlertTriangle} label="SLA geciken" value={(summary?.response_overdue_count ?? 0) + (summary?.resolution_overdue_count ?? 0)} helper={`${summary?.resolution_overdue_count ?? 0} çözüm`} tone={(summary?.response_overdue_count ?? 0) + (summary?.resolution_overdue_count ?? 0) > 0 ? "danger" : "default"} />
        <MetricCard icon={Clock} label="SLA yaklaşan" value={summary?.due_soon_count ?? 0} helper="4 saat içinde" tone={(summary?.due_soon_count ?? 0) > 0 ? "warning" : "default"} />
        <MetricCard icon={CheckCircle2} label="Bugün çözülen" value={summary?.resolved_today_count ?? 0} helper={`${summary?.closed_count ?? 0} toplam çözülen`} tone="success" />
      </div>

      <div className="grid gap-5 xl:grid-cols-[1fr_420px]">
        <div className="space-y-4">
          <Panel title="Yeni Destek Talebi">
            <form
              className="grid gap-3 md:grid-cols-4"
              onSubmit={(event) => {
                event.preventDefault();
                createMutation.mutate();
              }}
            >
              <Field label="Başlık" name="title" required values={createValues} setValues={setCreateValues} />
              <SelectField label="Öncelik" name="priority" options={priorities} values={createValues} setValues={setCreateValues} />
              <SelectField label="Kaynak" name="source_channel" options={sources} values={createValues} setValues={setCreateValues} />
              <Field label="Telefon" name="reporter_phone" values={createValues} setValues={setCreateValues} />
              <TextareaField label="Açıklama" name="description" required values={createValues} setValues={setCreateValues} />
              <div className="flex items-end md:col-span-4">
                <Button type="submit" disabled={createMutation.isPending}>
                  <RefreshCw size={16} />
                  Talebi Kaydet
                </Button>
              </div>
              {createMutation.error && <FormError error={createMutation.error} />}
            </form>
          </Panel>

          <Panel title="Talepler">
            <div className="mb-3 grid gap-3 md:grid-cols-5">
              <Field label="Ara" name="q" values={filters} setValues={setFilters} />
              <SelectField label="Durum" name="ticket_status" options={statuses} values={filters} setValues={setFilters} />
              <SelectField label="Öncelik" name="priority" options={priorities} values={filters} setValues={setFilters} />
              <SelectField label="Kaynak" name="source_channel" options={sources} values={filters} setValues={setFilters} />
              <SelectField label="SLA" name="overdue_only" options={["true"]} values={filters} setValues={setFilters} />
              <Field label="Telefon" name="reporter_phone" values={filters} setValues={setFilters} />
            </div>
            <DataTable data={tickets} columns={columns} emptyText="Destek talebi yok." />
          </Panel>
        </div>

        <Panel title={selectedTicket ? `${selectedTicket.ticket_no} Detay` : "Talep Detayı"}>
          {!selectedTicket ? (
            <div className="rounded-md border border-border bg-muted/20 px-3 py-6 text-center text-sm text-muted-foreground">
              Detay görmek için bir destek talebi açın.
            </div>
          ) : (
            <div className="space-y-4">
              <div className="rounded-md border border-border bg-muted/20 px-3 py-3">
                <div className="text-sm font-semibold">{selectedTicket.title}</div>
                <div className="mt-2 whitespace-pre-wrap text-sm text-muted-foreground">{selectedTicket.description}</div>
                <div className="mt-2 text-xs text-muted-foreground">
                  {humanize(selectedTicket.source_channel)} / {selectedTicket.reporter_name ?? "-"} / {selectedTicket.reporter_phone ?? "-"}
                </div>
                <div className="mt-3 grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
                  <InfoLine label="SLA" value={humanize(selectedTicket.sla_status)} />
                  <InfoLine label="İlk Cevap" value={formatDateTime(selectedTicket.sla_response_due_at)} />
                  <InfoLine label="Çözüm Hedefi" value={formatDateTime(selectedTicket.sla_resolution_due_at)} />
                  <InfoLine label="İlk Cevap Zamanı" value={formatDateTime(selectedTicket.first_response_at)} />
                  <InfoLine label="Eskalasyon" value={String(selectedTicket.escalation_level)} />
                  <InfoLine label="Memnuniyet" value={selectedTicket.satisfaction_score ? `${selectedTicket.satisfaction_score}/5` : "-"} />
                </div>
              </div>
              <form
                className="grid gap-3"
                onSubmit={(event) => {
                  event.preventDefault();
                  updateMutation.mutate();
                }}
              >
                <SelectField label="Durum" name="ticket_status" options={statuses} values={editValues} setValues={setEditValues} />
                <SelectField label="Öncelik" name="priority" options={priorities} values={editValues} setValues={setEditValues} />
                <UserSelect users={users} values={editValues} setValues={setEditValues} />
                <DepartmentSelect departments={departments} values={editValues} setValues={setEditValues} />
                <TextareaField label="Çözüm Notu" name="resolution_note" values={editValues} setValues={setEditValues} />
                <Field label="Memnuniyet Puanı" name="satisfaction_score" values={editValues} setValues={setEditValues} />
                <TextareaField label="Memnuniyet Notu" name="satisfaction_note" values={editValues} setValues={setEditValues} />
                <TextareaField label="İşlem Notu" name="event_note" values={editValues} setValues={setEditValues} />
                <label className="block md:col-span-4">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">WhatsApp Yanıtı</span>
                  <textarea
                    className="min-h-24 w-full rounded-md border border-input bg-white px-3 py-2 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                    value={replyMessage}
                    onChange={(event) => setReplyMessage(event.target.value)}
                  />
                </label>
                <div className="flex justify-end">
                  <div className="flex flex-wrap justify-end gap-2">
                    <Button type="button" variant="secondary" disabled={aiMutation.isPending} onClick={() => aiMutation.mutate("support_ticket_triage")}>
                      <Bot size={16} />
                      AI Analiz
                    </Button>
                    <Button type="button" variant="secondary" disabled={aiMutation.isPending} onClick={() => aiMutation.mutate("support_reply_draft")}>
                      <Bot size={16} />
                      AI Cevap Taslağı
                    </Button>
                    <Button type="button" variant="secondary" disabled={replyMutation.isPending || !selectedTicket.reporter_phone || !replyMessage.trim()} onClick={() => replyMutation.mutate()}>
                      <MessageSquare size={16} />
                      WhatsApp Yanıtı Hazırla
                    </Button>
                    <Button type="submit" disabled={updateMutation.isPending}>
                      Güncelle
                    </Button>
                  </div>
                </div>
                {aiMessage && (
                  <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                    {aiMessage}
                  </div>
                )}
                {replyStatus && (
                  <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                    {replyStatus}
                  </div>
                )}
                {aiMutation.error && <FormError error={aiMutation.error} />}
                {replyMutation.error && <FormError error={replyMutation.error} />}
                {updateMutation.error && <FormError error={updateMutation.error} />}
              </form>
              {eventsQuery.isLoading ? (
                <LoadingBlock />
              ) : eventsQuery.error ? (
                <ErrorBlock error={eventsQuery.error} />
              ) : (
                <>
                  <SupportFilesBlock
                    files={supportFiles}
                    isLoading={filesQuery.isLoading}
                    error={filesQuery.error}
                    fileType={supportFileType}
                    note={supportFileNote}
                    inputKey={supportFileInputKey}
                    isUploading={uploadSupportFileMutation.isPending}
                    uploadError={uploadSupportFileMutation.error}
                    deleteError={deleteSupportFileMutation.error}
                    isDeleting={deleteSupportFileMutation.isPending}
                    setFileType={setSupportFileType}
                    setNote={setSupportFileNote}
                    setFile={setSupportFile}
                    onUpload={() => uploadSupportFileMutation.mutate()}
                    onDelete={(id) => deleteSupportFileMutation.mutate(id)}
                  />
                  <DataTable data={eventsQuery.data ?? []} columns={eventColumns} emptyText="İşlem geçmişi yok." />
                </>
              )}
            </div>
          )}
        </Panel>
      </div>

      <div className="grid gap-5 xl:grid-cols-[420px_1fr]">
        <Panel title="Bilgi Bankasına Ekle" description="Çözülen veya tekrar eden destek taleplerinden standart çözüm kaydı oluşturulur.">
          <form
            className="grid gap-3"
            onSubmit={(event) => {
              event.preventDefault();
              kbMutation.mutate();
            }}
          >
            <Field label="Başlık" name="title" required values={kbValues} setValues={setKbValues} />
            <Field label="Kategori" name="category" required values={kbValues} setValues={setKbValues} />
            <TextareaField label="Problem" name="problem" required values={kbValues} setValues={setKbValues} />
            <TextareaField label="Çözüm" name="solution" required values={kbValues} setValues={setKbValues} />
            <Field label="Etiketler" name="tags" values={kbValues} setValues={setKbValues} />
            <div className="flex flex-wrap gap-2">
              <Button type="button" variant="secondary" disabled={!selectedTicket} onClick={() => setKbValues(ticketToKnowledgeValues(selectedTicket, editValues))}>
                <BookOpen size={16} />
                Seçili Talepten Doldur
              </Button>
              <Button type="submit" disabled={kbMutation.isPending}>
                Kaydet
              </Button>
            </div>
            {kbMutation.error && <FormError error={kbMutation.error} />}
          </form>
        </Panel>

        <Panel title="Bilgi Bankası">
          <div className="mb-3 grid gap-3 md:grid-cols-[1fr_160px]">
            <Field label="Çözüm Ara" name="q" values={{ q: kbSearch }} setValues={(updater) => {
              const next = typeof updater === "function" ? updater({ q: kbSearch }) : updater;
              setKbSearch(next.q ?? "");
            }} />
            <div className="flex items-end">
              <Button type="button" variant="secondary" className="w-full" onClick={() => setKbSearch("")}>
                Temizle
              </Button>
            </div>
          </div>
          <DataTable data={knowledgeItems} columns={kbColumns} emptyText="Bilgi bankası kaydı yok." />
        </Panel>
      </div>
    </div>
  );
}

function SupportFilesBlock({
  files,
  isLoading,
  error,
  fileType,
  note,
  inputKey,
  isUploading,
  uploadError,
  deleteError,
  isDeleting,
  setFileType,
  setNote,
  setFile,
  onUpload,
  onDelete
}: {
  files: FileDocumentRow[];
  isLoading: boolean;
  error: unknown;
  fileType: string;
  note: string;
  inputKey: number;
  isUploading: boolean;
  uploadError: unknown;
  deleteError: unknown;
  isDeleting: boolean;
  setFileType: (value: string) => void;
  setNote: (value: string) => void;
  setFile: (file: File | null) => void;
  onUpload: () => void;
  onDelete: (id: number) => void;
}) {
  return (
    <div className="space-y-3 rounded-md border border-border bg-muted/10 px-3 py-3">
      <div className="flex items-center gap-2 text-sm font-semibold">
        <Paperclip size={16} />
        Talep Ekleri
      </div>
      <form
        className="grid gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          onUpload();
        }}
      >
        <select
          className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
          value={fileType}
          onChange={(event) => setFileType(event.target.value)}
        >
          {supportFileTypes.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <Input key={inputKey} required type="file" onChange={(event) => setFile(event.target.files?.[0] ?? null)} />
        <Input placeholder="Not" value={note} onChange={(event) => setNote(event.target.value)} />
        <Button type="submit" disabled={isUploading}>
          <FileUp size={16} />
          Ek Yükle
        </Button>
        {uploadError ? <FormError error={uploadError} /> : null}
        {deleteError ? <FormError error={deleteError} /> : null}
      </form>
      {isLoading ? (
        <LoadingBlock />
      ) : error ? (
        <ErrorBlock error={error} />
      ) : files.length === 0 ? (
        <div className="rounded-md border border-dashed border-border bg-white px-3 py-4 text-center text-sm text-muted-foreground">
          Bu talebe bağlı ek dosya yok.
        </div>
      ) : (
        <div className="space-y-2">
          {files.map((file) => (
            <div key={file.id} className="flex items-center justify-between gap-3 rounded-md border border-border bg-white px-3 py-2">
              <div className="min-w-0">
                <div className="truncate text-sm font-medium">{file.original_name}</div>
                <div className="text-xs text-muted-foreground">
                  {humanize(file.file_type)} / {Math.max(1, Math.round(file.file_size / 1024)).toLocaleString("tr-TR")} KB / {formatDateTime(file.created_at)}
                </div>
              </div>
              <Button type="button" size="sm" variant="danger" disabled={isDeleting} onClick={() => onDelete(file.id)}>
                <Trash2 size={14} />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

type FieldProps = {
  label: string;
  name: string;
  required?: boolean;
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
};

function Field({ label, name, required, values, setValues }: FieldProps) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <Input required={required} value={values[name] ?? ""} onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))} />
    </label>
  );
}

function TextareaField({ label, name, required, values, setValues }: FieldProps) {
  return (
    <label className="block md:col-span-4">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <textarea
        required={required}
        className="min-h-24 w-full rounded-md border border-input bg-white px-3 py-2 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values[name] ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))}
      />
    </label>
  );
}

function SelectField({ label, name, options, values, setValues }: FieldProps & { options: string[] }) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values[name] ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))}
      >
        <option value="">Tümü</option>
        {options.map((option) => (
          <option key={option} value={option}>
            {option === "true" ? "Sadece geciken" : humanize(option)}
          </option>
        ))}
      </select>
    </label>
  );
}

function UserSelect({ users, values, setValues }: { users: UserRow[]; values: Record<string, string>; setValues: Dispatch<SetStateAction<Record<string, string>>> }) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Atanan Kullanıcı</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.assigned_user_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, assigned_user_id: event.target.value }))}
      >
        <option value="">Seçiniz</option>
        {users.map((user) => (
          <option key={user.id} value={user.id}>
            {user.full_name}
          </option>
        ))}
      </select>
    </label>
  );
}

function DepartmentSelect({ departments, values, setValues }: { departments: DepartmentRow[]; values: Record<string, string>; setValues: Dispatch<SetStateAction<Record<string, string>>> }) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Atanan Birim</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.assigned_department_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, assigned_department_id: event.target.value }))}
      >
        <option value="">Seçiniz</option>
        {departments.map((department) => (
          <option key={department.id} value={department.id}>
            {department.name}
          </option>
        ))}
      </select>
    </label>
  );
}

function FormError({ error }: { error: unknown }) {
  return (
    <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 md:col-span-4">
      {error instanceof Error ? error.message : "İşlem tamamlanamadı"}
    </div>
  );
}

function InfoLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md bg-white px-2 py-1">
      <span className="font-medium text-foreground">{label}: </span>
      <span>{value}</span>
    </div>
  );
}

function ticketToEditValues(ticket: SupportTicketRow) {
  return {
    ticket_status: ticket.ticket_status,
    priority: ticket.priority,
    assigned_user_id: ticket.assigned_user_id ? String(ticket.assigned_user_id) : "",
    assigned_department_id: ticket.assigned_department_id ? String(ticket.assigned_department_id) : "",
    resolution_note: ticket.resolution_note ?? "",
    satisfaction_score: ticket.satisfaction_score ? String(ticket.satisfaction_score) : "",
    satisfaction_note: ticket.satisfaction_note ?? "",
    event_note: ""
  };
}

function buildCreatePayload(values: Record<string, string>) {
  return cleanPayload(values, ["assigned_user_id", "assigned_department_id"]);
}

function buildUpdatePayload(values: Record<string, string>) {
  return cleanPayload(values, ["assigned_user_id", "assigned_department_id", "satisfaction_score"]);
}

function ticketToKnowledgeValues(ticket: SupportTicketRow | null, editValues: Record<string, string>) {
  if (!ticket) return { title: "", category: "it_support", problem: "", solution: "", tags: "it,destek" };
  return {
    title: ticket.title,
    category: ticket.category || "it_support",
    problem: ticket.description,
    solution: editValues.resolution_note || ticket.resolution_note || "",
    tags: [ticket.category, ticket.source_channel, ticket.priority].filter(Boolean).join(",")
  };
}

function defaultReplyMessage(ticket: SupportTicketRow) {
  return `Merhaba${ticket.reporter_name ? ` ${ticket.reporter_name}` : ""}, ${ticket.ticket_no} numaralı destek talebiniz üzerinde işlem başlatıldı. Gerekli kontrol yapıldıktan sonra sizi bilgilendireceğiz.`;
}

function buildKnowledgeBasePayload(values: Record<string, string>, ticket: SupportTicketRow | null) {
  return {
    title: values.title,
    category: values.category || "it_support",
    problem: values.problem,
    solution: values.solution,
    tags: values.tags
      ? values.tags.split(",").map((tag) => tag.trim()).filter(Boolean)
      : [],
    source_ticket_id: ticket?.id,
    is_published: true
  };
}

function cleanPayload(values: Record<string, string>, numericKeys: string[]) {
  const payload: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(values)) {
    if (value === "") continue;
    payload[key] = numericKeys.includes(key) ? Number(value) : value;
  }
  return payload;
}

function buildQueryString(values: Record<string, string>) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(values)) {
    if (value.trim()) params.set(key, value.trim());
  }
  return params.toString();
}
