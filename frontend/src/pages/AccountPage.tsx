import { useMutation } from "@tanstack/react-query";
import { KeyRound, UserRound } from "lucide-react";
import { useState } from "react";
import { MetricCard } from "../components/MetricCard";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import { useAuth } from "../modules/auth/AuthProvider";

export function AccountPage() {
  const { user, logout } = useAuth();
  const [values, setValues] = useState({ current_password: "", new_password: "", confirm_password: "" });
  const [success, setSuccess] = useState(false);

  const mutation = useMutation({
    mutationFn: async () => {
      if (values.new_password.length < 8) throw new Error("Yeni şifre en az 8 karakter olmalı");
      if (values.new_password !== values.confirm_password) throw new Error("Yeni şifre tekrarı eşleşmiyor");
      return api.changePassword({
        current_password: values.current_password,
        new_password: values.new_password
      });
    },
    onSuccess: async () => {
      setSuccess(true);
      setValues({ current_password: "", new_password: "", confirm_password: "" });
      await logout();
    }
  });

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Hesabım</h1>
        <p className="mt-1 text-sm text-muted-foreground">Oturum ve şifre güvenliği.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <MetricCard icon={UserRound} label="Kullanıcı" value={user?.full_name ?? "-"} helper={user?.email ?? "-"} />
        <MetricCard icon={KeyRound} label="Rol" value={user?.role ?? "-"} helper={user?.is_active ? "Aktif hesap" : "Pasif hesap"} />
        <MetricCard icon={KeyRound} label="Şifre Kuralı" value="8+" helper="Minimum karakter" />
      </div>

      <Panel title="Şifre Değiştir">
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            setSuccess(false);
            mutation.mutate();
          }}
        >
          <div className="grid gap-3 md:grid-cols-3">
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Mevcut Şifre</span>
              <Input
                required
                type="password"
                value={values.current_password}
                onChange={(event) => setValues((current) => ({ ...current, current_password: event.target.value }))}
              />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Yeni Şifre</span>
              <Input
                required
                minLength={8}
                type="password"
                value={values.new_password}
                onChange={(event) => setValues((current) => ({ ...current, new_password: event.target.value }))}
              />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-muted-foreground">Yeni Şifre Tekrar</span>
              <Input
                required
                minLength={8}
                type="password"
                value={values.confirm_password}
                onChange={(event) => setValues((current) => ({ ...current, confirm_password: event.target.value }))}
              />
            </label>
          </div>

          {mutation.isError && (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {mutation.error instanceof Error ? mutation.error.message : "Şifre değiştirilemedi"}
            </div>
          )}
          {success && (
            <div className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
              Şifre değiştirildi. Güvenlik nedeniyle tekrar giriş yapılması gerekiyor.
            </div>
          )}

          <div className="flex justify-end">
            <Button type="submit" disabled={mutation.isPending}>
              Şifreyi Değiştir
            </Button>
          </div>
        </form>
      </Panel>
    </div>
  );
}
