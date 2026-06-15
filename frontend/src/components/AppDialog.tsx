import { AlertTriangle, Info, MessageSquareText } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";

type DialogKind = "alert" | "confirm" | "prompt";

type DialogRequest = {
  id: number;
  kind: DialogKind;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  defaultValue?: string;
  tone?: "default" | "danger";
  resolve: (value: boolean | string | null) => void;
};

type DialogOptions = {
  title?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  defaultValue?: string;
  tone?: "default" | "danger";
};

let nextId = 1;
let listeners: Array<(request: DialogRequest) => void> = [];

export function appAlert(message: string, options: DialogOptions = {}) {
  return openDialog("alert", message, options) as Promise<boolean>;
}

export function appConfirm(message: string, options: DialogOptions = {}) {
  return openDialog("confirm", message, options) as Promise<boolean>;
}

export function appPrompt(message: string, options: DialogOptions = {}) {
  return openDialog("prompt", message, options) as Promise<string | null>;
}

function openDialog(kind: DialogKind, message: string, options: DialogOptions) {
  return new Promise((resolve) => {
    const request: DialogRequest = {
      id: nextId++,
      kind,
      title: options.title ?? defaultTitle(kind),
      message,
      confirmLabel: options.confirmLabel,
      cancelLabel: options.cancelLabel,
      defaultValue: options.defaultValue,
      tone: options.tone,
      resolve
    };
    listeners.forEach((listener) => listener(request));
  });
}

export function AppDialogHost() {
  const [queue, setQueue] = useState<DialogRequest[]>([]);
  const active = queue[0];
  const [promptValue, setPromptValue] = useState("");

  useEffect(() => {
    const listener = (request: DialogRequest) => setQueue((current) => [...current, request]);
    listeners = [...listeners, listener];
    return () => {
      listeners = listeners.filter((item) => item !== listener);
    };
  }, []);

  useEffect(() => {
    setPromptValue(active?.defaultValue ?? "");
  }, [active?.id, active?.defaultValue]);

  if (!active) return null;

  const close = (value: boolean | string | null) => {
    active.resolve(value);
    setQueue((current) => current.slice(1));
  };
  const Icon = active.kind === "prompt" ? MessageSquareText : active.tone === "danger" ? AlertTriangle : Info;
  const primaryVariant = active.tone === "danger" ? "danger" : "primary";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 px-4">
      <div className="w-full max-w-md rounded-lg border border-border bg-white shadow-xl">
        <div className="flex items-start gap-3 border-b border-border px-5 py-4">
          <div className={`mt-0.5 rounded-md p-2 ${active.tone === "danger" ? "bg-red-50 text-red-700" : "bg-slate-100 text-slate-700"}`}>
            <Icon size={18} />
          </div>
          <div>
            <h2 className="text-base font-semibold text-foreground">{active.title}</h2>
            <p className="mt-1 text-sm leading-6 text-muted-foreground">{active.message}</p>
          </div>
        </div>
        {active.kind === "prompt" ? (
          <div className="px-5 py-4">
            <Input value={promptValue} onChange={(event) => setPromptValue(event.target.value)} autoFocus />
          </div>
        ) : null}
        <div className="flex justify-end gap-2 px-5 py-4">
          {active.kind !== "alert" ? (
            <Button type="button" variant="secondary" onClick={() => close(null)}>
              {active.cancelLabel ?? "Vazgeç"}
            </Button>
          ) : null}
          <Button
            type="button"
            variant={primaryVariant}
            onClick={() => close(active.kind === "prompt" ? promptValue : true)}
          >
            {active.confirmLabel ?? (active.kind === "alert" ? "Tamam" : "Onayla")}
          </Button>
        </div>
      </div>
    </div>
  );
}

function defaultTitle(kind: DialogKind) {
  if (kind === "confirm") return "Onay gerekli";
  if (kind === "prompt") return "Bilgi gerekli";
  return "Uyarı";
}
