import { Bot, CheckCircle2, CircleDashed, ClipboardList, Cpu, FileText, Pencil } from "lucide-react";
import { useMutation, useQueries, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type { AiJob } from "../lib/types";
import { formatDateTime, humanize } from "../lib/utils";

export function AiPage() {
  const [selectedJob, setSelectedJob] = useState<AiJob | null>(null);
  const [jobValues, setJobValues] = useState<Record<string, string>>({
    job_status: "completed",
    confidence_score: "",
    result_data: "{}"
  });
  const [draftResult, setDraftResult] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const [providerQuery, capabilitiesQuery, recommendationsQuery, promptsQuery, jobsQuery] = useQueries({
    queries: [
      { queryKey: ["ai-provider"], queryFn: api.aiProvider },
      { queryKey: ["ai-capabilities"], queryFn: api.aiCapabilities },
      { queryKey: ["ai-recommendations"], queryFn: api.aiRecommendations },
      { queryKey: ["ai-prompts"], queryFn: api.aiPrompts },
      { queryKey: ["ai-jobs"], queryFn: api.aiJobs }
    ]
  });
  const updateMutation = useMutation({
    mutationFn: () => {
      const payload: Record<string, unknown> = {
        job_status: jobValues.job_status
      };
      if (jobValues.confidence_score) payload.confidence_score = jobValues.confidence_score;
      if (jobValues.result_data.trim()) {
        try {
          payload.result_data = JSON.parse(jobValues.result_data);
        } catch {
          payload.result_data = { text: jobValues.result_data };
        }
      }
      return api.updateAiJob(selectedJob!.id, payload);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["ai-jobs"] });
      setSelectedJob(null);
    }
  });
  const approveMutation = useMutation({
    mutationFn: (id: number) => api.approveAiJob(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["ai-jobs"] });
    }
  });
  const draftMutation = useMutation({
    mutationFn: async ({ id, type }: { id: number; type: "task" | "notification" }) => {
      const draft = type === "task" ? await api.aiTaskDraft(id) : await api.aiNotificationDraft(id);
      return JSON.stringify(draft, null, 2);
    },
    onSuccess: (value) => setDraftResult(value)
  });

  const isLoading = [providerQuery, capabilitiesQuery, recommendationsQuery, promptsQuery, jobsQuery].some(
    (query) => query.isLoading
  );
  const firstError = [providerQuery, capabilitiesQuery, recommendationsQuery, promptsQuery, jobsQuery].find(
    (query) => query.isError
  )?.error;

  if (isLoading) return <LoadingBlock />;
  if (firstError) return <ErrorBlock error={firstError} />;

  const provider = providerQuery.data!;
  const capabilities = capabilitiesQuery.data ?? [];
  const recommendations = recommendationsQuery.data ?? [];
  const prompts = promptsQuery.data ?? [];
  const jobs = jobsQuery.data ?? [];

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">AI Yönetimi</h1>
        <p className="mt-1 text-sm text-muted-foreground">AI iş sözleşmeleri, öneriler, prompt şablonları ve onay akışı.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard icon={Cpu} label="Provider" value={provider.provider} helper={`${provider.model} / ${provider.execution_mode}`} tone={provider.configured ? "success" : "warning"} />
        <MetricCard icon={Bot} label="Yetenek" value={capabilities.length} helper="Tanımlı analiz tipi" />
        <MetricCard icon={CircleDashed} label="Öneri" value={recommendations.length} helper="Backend sinyallerinden üretildi" tone={recommendations.length > 0 ? "warning" : "success"} />
        <MetricCard icon={ClipboardList} label="AI job" value={jobs.length} helper="Son kayıtlar" />
      </div>

      <Panel title="AI Bağlantı Hazırlığı">
        <div className="grid gap-3 md:grid-cols-4">
          <ReadinessItem label="Durum" value={provider.configured ? "Hazır" : "Eksik ayar var"} tone={provider.configured ? "success" : "warning"} />
          <ReadinessItem label="Dış model" value={provider.supports_external_models ? "Destekli" : "Kapalı"} />
          <ReadinessItem label="Local model" value={provider.supports_local_models ? "Destekli" : "Kapalı"} />
          <ReadinessItem label="Timeout" value={`${provider.timeout_seconds} sn`} />
        </div>
        <div className="mt-3 grid gap-3 md:grid-cols-2">
          <div className="rounded-md border border-border bg-background px-3 py-3">
            <div className="text-xs font-semibold text-muted-foreground">Gerekli ayarlar</div>
            <div className="mt-2 flex flex-wrap gap-2">
              {provider.required_env.map((item) => (
                <span key={item} className="rounded-md border border-border bg-muted/30 px-2 py-1 text-xs">
                  {item}
                </span>
              ))}
            </div>
          </div>
          <div className="rounded-md border border-border bg-background px-3 py-3">
            <div className="text-xs font-semibold text-muted-foreground">Eksik ayarlar</div>
            <div className="mt-2 flex flex-wrap gap-2">
              {provider.missing_env.length === 0 ? (
                <span className="rounded-md border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs text-emerald-700">Eksik yok</span>
              ) : (
                provider.missing_env.map((item) => (
                  <span key={item} className="rounded-md border border-amber-200 bg-amber-50 px-2 py-1 text-xs text-amber-700">
                    {item}
                  </span>
                ))
              )}
            </div>
          </div>
        </div>
        <div className="mt-3 rounded-md border border-border bg-muted/20 px-3 py-2 text-sm text-muted-foreground">
          {provider.next_step}
        </div>
      </Panel>

      <div className="grid gap-5 xl:grid-cols-[1fr_420px]">
        <Panel title="Öneriler">
          <div className="space-y-2">
            {recommendations.map((item) => (
              <div key={`${item.recommendation_type}-${item.source_module}`} className="rounded-md border border-border px-3 py-3">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <h2 className="text-sm font-semibold">{item.title}</h2>
                    <p className="mt-1 text-sm text-muted-foreground">{item.detail}</p>
                  </div>
                  <StatusBadge value={item.severity} />
                </div>
                <div className="mt-2 text-xs text-muted-foreground">
                  {humanize(item.source_module)} / {item.source_count} kayıt / {item.suggested_analysis_type}
                </div>
              </div>
            ))}
            {recommendations.length === 0 && (
              <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-4 text-sm text-emerald-700">
                Şu an otomatik öneri oluşmadı.
              </div>
            )}
          </div>
        </Panel>

        <Panel title="Son AI İşleri">
          {selectedJob && (
            <form
              className="mb-3 rounded-lg border border-border bg-background p-3"
              onSubmit={(event) => {
                event.preventDefault();
                updateMutation.mutate();
              }}
            >
              <div className="mb-3 flex items-center justify-between gap-2">
                <div>
                  <h2 className="text-sm font-semibold">AI Job Düzenle #{selectedJob.id}</h2>
                  <p className="mt-1 text-xs text-muted-foreground">Durum, güven skoru ve sonuç verisi düzenlenir.</p>
                </div>
                <Button type="button" size="sm" variant="secondary" onClick={() => setSelectedJob(null)}>
                  Kapat
                </Button>
              </div>
              <div className="grid gap-2">
                <label className="block">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">Durum</span>
                  <select
                    className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                    value={jobValues.job_status}
                    onChange={(event) => setJobValues((current) => ({ ...current, job_status: event.target.value }))}
                  >
                    <option value="pending">Pending</option>
                    <option value="running">Running</option>
                    <option value="completed">Completed</option>
                    <option value="failed">Failed</option>
                    <option value="approved">Approved</option>
                    <option value="rejected">Rejected</option>
                  </select>
                </label>
                <label className="block">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">Güven Skoru</span>
                  <Input
                    type="number"
                    step="0.01"
                    value={jobValues.confidence_score}
                    onChange={(event) => setJobValues((current) => ({ ...current, confidence_score: event.target.value }))}
                  />
                </label>
                <label className="block">
                  <span className="mb-1 block text-xs font-medium text-muted-foreground">Sonuç JSON</span>
                  <textarea
                    className="min-h-28 w-full rounded-md border border-input bg-white px-3 py-2 font-mono text-xs outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
                    value={jobValues.result_data}
                    onChange={(event) => setJobValues((current) => ({ ...current, result_data: event.target.value }))}
                  />
                </label>
              </div>
              {updateMutation.isError && (
                <div className="mt-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                  {updateMutation.error instanceof Error ? updateMutation.error.message : "AI job güncellenemedi"}
                </div>
              )}
              <div className="mt-3 flex justify-end">
                <Button type="submit" disabled={updateMutation.isPending}>
                  Kaydet
                </Button>
              </div>
            </form>
          )}
          <div className="divide-y divide-border">
            {jobs.map((job) => (
              <div key={job.id} className="py-3">
                <div className="flex items-center justify-between gap-3">
                  <div className="text-sm font-medium">{humanize(job.analysis_type)}</div>
                  <StatusBadge value={job.job_status} />
                </div>
                <div className="mt-1 text-xs text-muted-foreground">
                  #{job.id} / Araç: {job.related_vehicle_id ?? "-"} / {formatDateTime(job.created_at)}
                </div>
                <div className="mt-2 flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      setSelectedJob(job);
                      setDraftResult(null);
                      setJobValues({
                        job_status: job.job_status,
                        confidence_score: job.confidence_score ?? "",
                        result_data: job.result_data ? JSON.stringify(job.result_data, null, 2) : "{}"
                      });
                    }}
                  >
                    <Pencil size={14} />
                    Düzenle
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => approveMutation.mutate(job.id)}>
                    Onayla
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => draftMutation.mutate({ id: job.id, type: "task" })}>
                    Görev Taslağı
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => draftMutation.mutate({ id: job.id, type: "notification" })}>
                    Bildirim Taslağı
                  </Button>
                </div>
              </div>
            ))}
            {jobs.length === 0 && <div className="py-6 text-center text-sm text-muted-foreground">AI job kaydı yok.</div>}
          </div>
          {(draftResult || draftMutation.isError || approveMutation.isError) && (
            <div className="mt-3 rounded-lg border border-border bg-background p-3">
              <h2 className="text-sm font-semibold">Aksiyon Çıktısı</h2>
              {draftMutation.isError || approveMutation.isError ? (
                <div className="mt-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                  {(draftMutation.error ?? approveMutation.error) instanceof Error
                    ? ((draftMutation.error ?? approveMutation.error) as Error).message
                    : "AI aksiyonu çalışmadı"}
                </div>
              ) : (
                <pre className="mt-2 max-h-80 overflow-auto rounded-md bg-white p-3 text-xs">{draftResult}</pre>
              )}
            </div>
          )}
        </Panel>
      </div>

      <div className="grid gap-5 xl:grid-cols-2">
        <Panel title="Analiz Yetenekleri">
          <div className="grid gap-3">
            {capabilities.map((capability) => (
              <div key={capability.analysis_type} className="rounded-md border border-border px-3 py-3">
                <div className="flex items-start gap-3">
                  <div className="mt-0.5 text-primary">
                    <CheckCircle2 size={17} aria-hidden="true" />
                  </div>
                  <div>
                    <h2 className="text-sm font-semibold">{capability.title}</h2>
                    <p className="mt-1 text-sm text-muted-foreground">{capability.description}</p>
                    <p className="mt-2 text-xs text-muted-foreground">
                      Modül: {capability.target_modules.join(", ")} / İnsan onayı:{" "}
                      {capability.human_approval_required ? "Zorunlu" : "Opsiyonel"}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Panel>

        <Panel title="Prompt Şablonları">
          <div className="space-y-3">
            {prompts.map((prompt) => (
              <div key={prompt.analysis_type} className="rounded-md border border-border px-3 py-3">
                <div className="flex items-start gap-3">
                  <FileText className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
                  <div>
                    <h2 className="text-sm font-semibold">{humanize(prompt.analysis_type)}</h2>
                    <p className="mt-1 text-sm text-muted-foreground">{prompt.system_goal}</p>
                    <p className="mt-2 text-xs text-muted-foreground">
                      Çıktı: {prompt.output_contract.slice(0, 2).join(", ")}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Panel>
      </div>
    </div>
  );
}

function ReadinessItem({ label, value, tone = "default" }: { label: string; value: string; tone?: "default" | "success" | "warning" }) {
  const toneClass =
    tone === "success"
      ? "border-emerald-200 bg-emerald-50 text-emerald-700"
      : tone === "warning"
        ? "border-amber-200 bg-amber-50 text-amber-700"
        : "border-border bg-background text-foreground";
  return (
    <div className={`rounded-md border px-3 py-3 ${toneClass}`}>
      <div className="text-xs font-medium opacity-75">{label}</div>
      <div className="mt-1 text-sm font-semibold">{value}</div>
    </div>
  );
}
