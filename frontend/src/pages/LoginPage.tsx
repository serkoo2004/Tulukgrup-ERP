import { zodResolver } from "@hookform/resolvers/zod";
import { AlertCircle, LogIn } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { Navigate, useLocation, useNavigate } from "react-router-dom";
import { z } from "zod";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { useAuth } from "../modules/auth/AuthProvider";

const schema = z.object({
  email: z.string().email("Geçerli e-posta girin"),
  password: z.string().min(1, "Şifre zorunlu")
});

type LoginForm = z.infer<typeof schema>;

export function LoginPage() {
  const { isAuthenticated, login } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [error, setError] = useState<string | null>(null);
  const from = (location.state as { from?: { pathname?: string } } | null)?.from?.pathname ?? "/dashboard";

  const form = useForm<LoginForm>({
    resolver: zodResolver(schema),
    defaultValues: {
      email: "admin@tuluklar.local",
      password: "Admin12345!"
    }
  });

  if (isAuthenticated) {
    return <Navigate to={from} replace />;
  }

  const submit = form.handleSubmit(async (values) => {
    setError(null);
    try {
      await login(values.email, values.password);
      navigate(from, { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Giriş yapılamadı");
    }
  });

  return (
    <main className="grid min-h-screen grid-cols-1 bg-background lg:grid-cols-[minmax(420px,520px)_1fr]">
      <section className="flex items-center border-r border-border bg-white px-6 py-10 lg:px-12">
        <div className="w-full max-w-sm">
          <div className="mb-8">
            <div className="text-sm font-semibold text-primary">Tuluklar ERP</div>
            <h1 className="mt-2 text-2xl font-semibold text-foreground">Yönetim paneli girişi</h1>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">
              Filo, operasyon, rapor ve AI destekli takip ekranlarına erişim.
            </p>
          </div>

          <form className="space-y-4" onSubmit={submit}>
            <label className="block">
              <span className="mb-1 block text-sm font-medium">E-posta</span>
              <Input autoComplete="email" {...form.register("email")} />
              {form.formState.errors.email && (
                <span className="mt-1 block text-xs text-red-600">{form.formState.errors.email.message}</span>
              )}
            </label>
            <label className="block">
              <span className="mb-1 block text-sm font-medium">Şifre</span>
              <Input type="password" autoComplete="current-password" {...form.register("password")} />
              {form.formState.errors.password && (
                <span className="mt-1 block text-xs text-red-600">
                  {form.formState.errors.password.message}
                </span>
              )}
            </label>

            {error && (
              <div className="flex items-center gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                <AlertCircle size={16} />
                {error}
              </div>
            )}

            <Button className="w-full" disabled={form.formState.isSubmitting} type="submit">
              <LogIn size={17} />
              Giriş Yap
            </Button>
          </form>
        </div>
      </section>
      <section className="hidden bg-[linear-gradient(180deg,#f8fafc_0%,#eef2f2_100%)] p-10 lg:block">
        <div className="grid h-full grid-rows-[auto_1fr_auto]">
          <div className="text-right text-xs text-muted-foreground">Local PostgreSQL / Backend API</div>
          <div className="flex items-center justify-center">
            <div className="w-full max-w-3xl rounded-lg border border-border bg-white p-6 shadow-panel">
              <div className="grid grid-cols-3 gap-3">
                {["Araç envanteri", "Operasyon takip", "Excel rapor", "AI öneri", "Bakım / poliçe", "Audit log"].map(
                  (label) => (
                    <div key={label} className="rounded-md border border-border bg-background px-4 py-5">
                      <div className="text-sm font-semibold">{label}</div>
                      <div className="mt-2 h-2 rounded bg-muted" />
                      <div className="mt-2 h-2 w-2/3 rounded bg-muted" />
                    </div>
                  )
                )}
              </div>
            </div>
          </div>
          <div className="text-xs text-muted-foreground">Web panel responsive hazırlandı; mobil uygulama ayrı faz.</div>
        </div>
      </section>
    </main>
  );
}
