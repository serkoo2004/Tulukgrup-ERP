import { Navigate, Route, Routes } from "react-router-dom";
import { lazy, Suspense } from "react";
import type { ReactNode } from "react";
import { AppShell } from "./components/AppShell";
import { LoadingBlock } from "./components/LoadState";
import { ProtectedRoute } from "./modules/auth/ProtectedRoute";
import { LoginPage } from "./pages/LoginPage";

const AiPage = lazy(() => import("./pages/AiPage").then((module) => ({ default: module.AiPage })));
const AccountPage = lazy(() => import("./pages/AccountPage").then((module) => ({ default: module.AccountPage })));
const DashboardPage = lazy(() => import("./pages/DashboardPage").then((module) => ({ default: module.DashboardPage })));
const FilesPage = lazy(() => import("./pages/FilesPage").then((module) => ({ default: module.FilesPage })));
const ImportsPage = lazy(() => import("./pages/ImportsPage").then((module) => ({ default: module.ImportsPage })));
const InventoryPage = lazy(() => import("./pages/InventoryPage").then((module) => ({ default: module.InventoryPage })));
const OperationsPage = lazy(() => import("./pages/OperationsPage").then((module) => ({ default: module.OperationsPage })));
const OrganizationPage = lazy(() => import("./pages/OrganizationPage").then((module) => ({ default: module.OrganizationPage })));
const ReportsPage = lazy(() => import("./pages/ReportsPage").then((module) => ({ default: module.ReportsPage })));
const SupportPage = lazy(() => import("./pages/SupportPage").then((module) => ({ default: module.SupportPage })));
const SystemPage = lazy(() => import("./pages/SystemPage").then((module) => ({ default: module.SystemPage })));
const TrackingPage = lazy(() => import("./pages/TrackingPage").then((module) => ({ default: module.TrackingPage })));
const UsersPage = lazy(() => import("./pages/UsersPage").then((module) => ({ default: module.UsersPage })));
const VehicleDetailPage = lazy(() => import("./pages/VehicleDetailPage").then((module) => ({ default: module.VehicleDetailPage })));
const VehiclesPage = lazy(() => import("./pages/VehiclesPage").then((module) => ({ default: module.VehiclesPage })));

function Page({ children }: { children: ReactNode }) {
  return <Suspense fallback={<LoadingBlock />}>{children}</Suspense>;
}

export function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/"
        element={
          <ProtectedRoute>
            <AppShell />
          </ProtectedRoute>
        }
      >
        <Route index element={<Navigate to="/dashboard" replace />} />
        <Route path="dashboard" element={<Page><DashboardPage /></Page>} />
        <Route path="vehicles" element={<Page><VehiclesPage /></Page>} />
        <Route path="vehicles/:id" element={<Page><VehicleDetailPage /></Page>} />
        <Route path="operations" element={<Page><OperationsPage /></Page>} />
        <Route path="tracking" element={<Page><TrackingPage /></Page>} />
        <Route path="files" element={<Page><FilesPage /></Page>} />
        <Route path="imports" element={<Page><ImportsPage /></Page>} />
        <Route path="inventory" element={<Page><InventoryPage /></Page>} />
        <Route path="reports" element={<Page><ReportsPage /></Page>} />
        <Route path="support" element={<Page><SupportPage /></Page>} />
        <Route path="ai" element={<Page><AiPage /></Page>} />
        <Route path="account" element={<Page><AccountPage /></Page>} />
        <Route path="users" element={<Page><UsersPage /></Page>} />
        <Route path="organization" element={<Page><OrganizationPage /></Page>} />
        <Route path="system" element={<Page><SystemPage /></Page>} />
      </Route>
      <Route path="*" element={<Navigate to="/dashboard" replace />} />
    </Routes>
  );
}
