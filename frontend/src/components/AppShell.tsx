import {
  BarChart3,
  Bot,
  Building2,
  Car,
  ClipboardList,
  FileSpreadsheet,
  FolderOpen,
  Headset,
  PackageSearch,
  Upload,
  LogOut,
  Menu,
  Settings,
  UserRound,
  Users
} from "lucide-react";
import { useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../modules/auth/AuthProvider";
import { AppDialogHost } from "./AppDialog";
import { GlobalSearch } from "./GlobalSearch";
import { Button } from "./ui/Button";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: BarChart3 },
  { to: "/vehicles", label: "Araçlar", icon: Car },
  { to: "/operations", label: "Operasyon", icon: ClipboardList },
  { to: "/tracking", label: "Takip", icon: ClipboardList },
  { to: "/inventory", label: "Stok", icon: PackageSearch },
  { to: "/files", label: "Dosyalar", icon: FolderOpen },
  { to: "/imports", label: "Import", icon: Upload },
  { to: "/reports", label: "Raporlar", icon: FileSpreadsheet },
  { to: "/support", label: "IT Destek", icon: Headset },
  { to: "/ai", label: "AI", icon: Bot },
  { to: "/account", label: "Hesabım", icon: UserRound },
  { to: "/users", label: "Kullanıcılar", icon: Users },
  { to: "/organization", label: "Organizasyon", icon: Building2 },
  { to: "/system", label: "Sistem", icon: Settings }
];

export function AppShell() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const { logout, user } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate("/login", { replace: true });
  };

  return (
    <div className="min-h-screen bg-background">
      <aside
        className={`fixed inset-y-0 left-0 z-30 w-64 border-r border-border bg-white transition-transform lg:translate-x-0 ${
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <div className="flex h-16 items-center border-b border-border px-5">
          <div>
            <div className="text-sm font-semibold text-foreground">Tuluklar ERP</div>
            <div className="text-xs text-muted-foreground">Filo operasyon paneli</div>
          </div>
        </div>
        <nav className="space-y-1 p-3">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              onClick={() => setSidebarOpen(false)}
              className={({ isActive }) =>
                `flex h-10 items-center gap-3 rounded-md px-3 text-sm font-medium ${
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`
              }
            >
              <item.icon size={18} aria-hidden="true" />
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>

      {sidebarOpen && (
        <button
          className="fixed inset-0 z-20 bg-black/20 lg:hidden"
          aria-label="Menüyü kapat"
          type="button"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <div className="lg:pl-64">
        <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b border-border bg-white px-4 lg:px-6">
          <div className="flex min-w-0 items-center gap-3">
            <Button
              variant="ghost"
              size="icon"
              className="lg:hidden"
              aria-label="Menüyü aç"
              onClick={() => setSidebarOpen(true)}
            >
              <Menu size={18} />
            </Button>
            <GlobalSearch />
          </div>
          <div className="flex items-center gap-3">
            <div className="hidden text-right sm:block">
              <div className="text-sm font-medium">{user?.full_name ?? "Sistem Admin"}</div>
              <div className="text-xs text-muted-foreground">{user?.role ?? "admin"}</div>
            </div>
            <Button variant="secondary" size="icon" aria-label="Çıkış yap" onClick={handleLogout}>
              <LogOut size={17} />
            </Button>
          </div>
        </header>
        <main className="mx-auto w-full max-w-[1500px] px-4 py-5 lg:px-6">
          <Outlet />
        </main>
      </div>
      <AppDialogHost />
    </div>
  );
}
