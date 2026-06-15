import type {
  AiCapability,
  AiJob,
  AiPromptTemplate,
  AiProviderStatus,
  AiRecommendation,
  AiNotificationDraft,
  AiTaskDraft,
  AssignmentRow,
  AuditLogRow,
  CompanyRow,
  CurrentUser,
  DashboardResponse,
  DepartmentRow,
  FileDocumentRow,
  ImportJobRow,
  ImportErrorRow,
  ImportStagingRow,
  ImportValidationSummary,
  InventoryAlertRow,
  InventoryCountDetail,
  InventoryCountRow,
  InventoryDashboardRow,
  InventoryLotRow,
  InventoryMovementRow,
  InventoryProductRow,
  InventoryPurchaseRequestRow,
  InventoryPurchaseSuggestionRow,
  InventoryShipmentDetail,
  InventoryShipmentRow,
  KmLogRow,
  LoginResponse,
  OperationRecord,
  SearchResult,
  SettingRow,
  RoleRow,
  MobilePermissionInput,
  MobilePermissionRow,
  SupportTicketEventRow,
  SupportTicketRow,
  SupportTicketReplyResponse,
  SupportKnowledgeBaseRow,
  SupportSummary,
  SystemLogRow,
  UserCreateInput,
  UserUpdateInput,
  UserRow,
  Vehicle,
  VehicleCreateInput,
  VehicleSellInput,
  VehicleUpdateInput,
  VehicleProfile
} from "./types";

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ??
  `${window.location.protocol}//${window.location.hostname}:8080`;
const ACCESS_TOKEN_KEY = "tuluklar_access_token";
const REFRESH_TOKEN_KEY = "tuluklar_refresh_token";

export function getAccessToken() {
  return localStorage.getItem(ACCESS_TOKEN_KEY);
}

export function getRefreshToken() {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

export function setTokens(tokens: LoginResponse) {
  localStorage.setItem(ACCESS_TOKEN_KEY, tokens.access_token);
  localStorage.setItem(REFRESH_TOKEN_KEY, tokens.refresh_token);
}

export function clearTokens() {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

type RequestOptions = RequestInit & {
  auth?: boolean;
};

export async function apiRequest<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers = new Headers(options.headers);
  if (!headers.has("Content-Type") && options.body) {
    headers.set("Content-Type", "application/json");
  }

  if (options.auth !== false) {
    const token = getAccessToken();
    if (token) headers.set("Authorization", `Bearer ${token}`);
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers
  });

  if (!response.ok) {
    let detail = `${response.status} ${response.statusText}`;
    try {
      const payload = await response.json();
      detail = payload.error ?? payload.message ?? detail;
    } catch {
      // ignore non-json error bodies
    }
    throw new Error(detail);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

export const api = {
  login: (email: string, password: string) =>
    apiRequest<LoginResponse>("/api/v1/auth/login", {
      method: "POST",
      auth: false,
      body: JSON.stringify({ email, password })
    }),
  logout: (refresh_token: string) =>
    apiRequest<{ message: string }>("/api/v1/auth/logout", {
      method: "POST",
      body: JSON.stringify({ refresh_token })
    }),
  changePassword: (payload: Record<string, unknown>) =>
    apiRequest<{ message: string }>("/api/v1/auth/change-password", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  me: () => apiRequest<CurrentUser>("/api/v1/auth/me"),
  users: () => apiRequest<UserRow[]>("/api/v1/users?include_inactive=true"),
  roles: () => apiRequest<RoleRow[]>("/api/v1/users/roles"),
  createUser: (payload: UserCreateInput) =>
    apiRequest<UserRow>("/api/v1/users", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateUser: (id: string | number, payload: UserUpdateInput) =>
    apiRequest<UserRow>(`/api/v1/users/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  userMobilePermissions: (id: string | number) =>
    apiRequest<MobilePermissionRow[]>(`/api/v1/users/${id}/mobile-permissions`),
  upsertUserMobilePermission: (id: string | number, payload: MobilePermissionInput) =>
    apiRequest<MobilePermissionRow>(`/api/v1/users/${id}/mobile-permissions`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  deleteUser: (id: string | number) =>
    apiRequest<UserRow>(`/api/v1/users/${id}`, {
      method: "DELETE"
    }),
  companies: () => apiRequest<CompanyRow[]>("/api/v1/organization/companies?include_inactive=true"),
  createCompany: (payload: Record<string, unknown>) =>
    apiRequest<CompanyRow>("/api/v1/organization/companies", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateCompany: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<CompanyRow>(`/api/v1/organization/companies/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  departments: () => apiRequest<DepartmentRow[]>("/api/v1/organization/departments?include_inactive=true"),
  createDepartment: (payload: Record<string, unknown>) =>
    apiRequest<DepartmentRow>("/api/v1/organization/departments", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateDepartment: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<DepartmentRow>(`/api/v1/organization/departments/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  dashboard: () => apiRequest<DashboardResponse>("/api/v1/dashboard"),
  search: (q: string) => apiRequest<SearchResult[]>(`/api/v1/search?q=${encodeURIComponent(q)}&limit=12`),
  vehicles: (query = "status=all") => apiRequest<Vehicle[]>(`/api/v1/vehicles?${query}`),
  createVehicle: (payload: VehicleCreateInput) =>
    apiRequest<Vehicle>("/api/v1/vehicles", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateVehicle: (id: string | number, payload: VehicleUpdateInput) =>
    apiRequest<Vehicle>(`/api/v1/vehicles/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  deleteVehicle: (id: string | number) =>
    apiRequest<Vehicle>(`/api/v1/vehicles/${id}`, {
      method: "DELETE"
    }),
  sellVehicle: (id: string | number, payload: VehicleSellInput) =>
    apiRequest<unknown>(`/api/v1/vehicles/${id}/sell`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  vehicleProfile: (id: string) => apiRequest<VehicleProfile>(`/api/v1/vehicles/${id}/profile`),
  inventoryDashboard: () => apiRequest<InventoryDashboardRow>("/api/v1/inventory/dashboard"),
  inventoryProducts: (query = "limit=100") => apiRequest<InventoryProductRow[]>(`/api/v1/inventory/products?${query}`),
  createInventoryProduct: (payload: Record<string, unknown>) =>
    apiRequest<InventoryProductRow>("/api/v1/inventory/products", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateInventoryProduct: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<InventoryProductRow>(`/api/v1/inventory/products/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  deleteInventoryProduct: (id: string | number) =>
    apiRequest<InventoryProductRow>(`/api/v1/inventory/products/${id}`, {
      method: "DELETE"
    }),
  inventoryMovements: (query = "limit=100") => apiRequest<InventoryMovementRow[]>(`/api/v1/inventory/movements?${query}`),
  createInventoryMovement: (payload: Record<string, unknown>) =>
    apiRequest<InventoryMovementRow>("/api/v1/inventory/movements", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  cancelInventoryMovement: (id: string | number, reason: string) =>
    apiRequest<InventoryMovementRow>(`/api/v1/inventory/movements/${id}/cancel`, {
      method: "POST",
      body: JSON.stringify({ reason })
    }),
  inventoryLots: (query = "limit=100") => apiRequest<InventoryLotRow[]>(`/api/v1/inventory/lots?${query}`),
  createInventoryLot: (payload: Record<string, unknown>) =>
    apiRequest<InventoryLotRow>("/api/v1/inventory/lots", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  inventoryShipments: () => apiRequest<InventoryShipmentRow[]>("/api/v1/inventory/shipments"),
  inventoryShipment: (id: string | number) => apiRequest<InventoryShipmentDetail>(`/api/v1/inventory/shipments/${id}`),
  createInventoryShipment: (payload: Record<string, unknown>) =>
    apiRequest<InventoryShipmentRow>("/api/v1/inventory/shipments", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  approveInventoryShipment: (id: string | number) =>
    apiRequest<InventoryShipmentRow>(`/api/v1/inventory/shipments/${id}/approve`, {
      method: "POST"
    }),
  inventoryCounts: () => apiRequest<InventoryCountRow[]>("/api/v1/inventory/counts"),
  inventoryCount: (id: string | number) => apiRequest<InventoryCountDetail>(`/api/v1/inventory/counts/${id}`),
  createInventoryCount: (payload: Record<string, unknown>) =>
    apiRequest<InventoryCountRow>("/api/v1/inventory/counts", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  completeInventoryCount: (id: string | number) =>
    apiRequest<InventoryCountRow>(`/api/v1/inventory/counts/${id}/complete`, {
      method: "POST"
    }),
  inventoryAlerts: () => apiRequest<InventoryAlertRow[]>("/api/v1/inventory/alerts"),
  inventoryPurchaseRequests: () => apiRequest<InventoryPurchaseRequestRow[]>("/api/v1/inventory/purchase-requests"),
  createInventoryPurchaseRequest: (payload: Record<string, unknown>) =>
    apiRequest<InventoryPurchaseRequestRow>("/api/v1/inventory/purchase-requests", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateInventoryPurchaseRequest: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<InventoryPurchaseRequestRow>(`/api/v1/inventory/purchase-requests/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  inventoryPurchaseSuggestions: () =>
    apiRequest<InventoryPurchaseSuggestionRow[]>("/api/v1/inventory/purchase-suggestions"),
  kmLogs: (query = "limit=100") => apiRequest<KmLogRow[]>(`/api/v1/tracking/km-logs?${query}`),
  vehicleKmLogs: (vehicleId: string | number, query = "limit=100") =>
    apiRequest<KmLogRow[]>(`/api/v1/tracking/vehicles/${vehicleId}/km-logs?${query}`),
  createKmLog: (payload: Record<string, unknown>) =>
    apiRequest<KmLogRow>("/api/v1/tracking/km-logs", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  verifyKmLog: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<KmLogRow>(`/api/v1/tracking/km-logs/${id}/verify`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  vehicleAssignments: (vehicleId: string | number) =>
    apiRequest<AssignmentRow[]>(`/api/v1/assignments/vehicles/${vehicleId}?limit=100`),
  assignVehicle: (payload: Record<string, unknown>) =>
    apiRequest<AssignmentRow>("/api/v1/assignments", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  releaseAssignment: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<AssignmentRow>(`/api/v1/assignments/${id}/release`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  operations: (type: string, limit = 20) =>
    apiRequest<OperationRecord[]>(`/api/v1/operations/${type}?limit=${limit}`),
  createOperation: (type: string, payload: Record<string, unknown>) =>
    apiRequest<OperationRecord>(`/api/v1/operations/${type}`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateOperation: (type: string, id: string | number, payload: Record<string, unknown>) =>
    apiRequest<OperationRecord>(`/api/v1/operations/${type}/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  deleteOperation: (type: string, id: string | number) =>
    apiRequest<OperationRecord>(`/api/v1/operations/${type}/${id}`, {
      method: "DELETE"
    }),
  records: (path: string) => apiRequest<OperationRecord[]>(`/api/v1/${path}`),
  createRecord: (path: string, payload: Record<string, unknown>) =>
    apiRequest<OperationRecord>(`/api/v1/${path}`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateRecord: (path: string, id: string | number, payload: Record<string, unknown>) =>
    apiRequest<OperationRecord>(`/api/v1/${path}/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  deleteRecord: (path: string, id: string | number) =>
    apiRequest<OperationRecord>(`/api/v1/${path}/${id}`, {
      method: "DELETE"
    }),
  markNotificationRead: (id: string | number) =>
    apiRequest<OperationRecord>(`/api/v1/notifications/${id}/read`, {
      method: "POST"
    }),
  settings: () => apiRequest<SettingRow[]>("/api/v1/settings"),
  upsertSetting: (payload: Record<string, unknown>) =>
    apiRequest<SettingRow>("/api/v1/settings", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateSetting: (key: string, payload: Record<string, unknown>) =>
    apiRequest<SettingRow>(`/api/v1/settings/${encodeURIComponent(key)}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  supportTickets: (query = "limit=100") => apiRequest<SupportTicketRow[]>(`/api/v1/support/tickets?${query}`),
  supportTicket: (id: string | number) => apiRequest<SupportTicketRow>(`/api/v1/support/tickets/${id}`),
  createSupportTicket: (payload: Record<string, unknown>) =>
    apiRequest<SupportTicketRow>("/api/v1/support/tickets", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateSupportTicket: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<SupportTicketRow>(`/api/v1/support/tickets/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  replySupportTicket: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<SupportTicketReplyResponse>(`/api/v1/support/tickets/${id}/reply`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  supportSummary: () => apiRequest<SupportSummary>("/api/v1/support/summary"),
  supportTicketEvents: (id: string | number) => apiRequest<SupportTicketEventRow[]>(`/api/v1/support/tickets/${id}/events`),
  supportKnowledgeBase: (query = "limit=50") => apiRequest<SupportKnowledgeBaseRow[]>(`/api/v1/support/knowledge-base?${query}`),
  createSupportKnowledgeBase: (payload: Record<string, unknown>) =>
    apiRequest<SupportKnowledgeBaseRow>("/api/v1/support/knowledge-base", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateSupportKnowledgeBase: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<SupportKnowledgeBaseRow>(`/api/v1/support/knowledge-base/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  systemLogs: () => apiRequest<SystemLogRow[]>("/api/v1/system-logs?limit=80"),
  createSystemLog: (payload: Record<string, unknown>) =>
    apiRequest<SystemLogRow>("/api/v1/system-logs", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  auditLogs: () => apiRequest<AuditLogRow[]>("/api/v1/logs/audit?limit=80"),
  files: (query = "limit=80") => apiRequest<FileDocumentRow[]>(`/api/v1/files?${query}`),
  deleteFile: (id: string | number) =>
    apiRequest<FileDocumentRow>(`/api/v1/files/${id}`, {
      method: "DELETE"
    }),
  importJobs: () => apiRequest<ImportJobRow[]>("/api/v1/imports/jobs?limit=80"),
  createImportJob: (payload: Record<string, unknown>) =>
    apiRequest<ImportJobRow>("/api/v1/imports/jobs", {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  updateImportJob: (jobId: number, payload: Record<string, unknown>) =>
    apiRequest<ImportJobRow>(`/api/v1/imports/jobs/${jobId}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  stageImportRows: (jobId: number, rows: unknown[]) =>
    apiRequest<unknown[]>(`/api/v1/imports/jobs/${jobId}/rows`, {
      method: "POST",
      body: JSON.stringify({ rows })
    }),
  validateImportJob: (jobId: number) =>
    apiRequest<ImportValidationSummary>(`/api/v1/imports/jobs/${jobId}/validate`, {
      method: "POST"
    }),
  importRows: (jobId: number) => apiRequest<ImportStagingRow[]>(`/api/v1/imports/jobs/${jobId}/rows`),
  importErrors: (jobId: number) => apiRequest<ImportErrorRow[]>(`/api/v1/imports/jobs/${jobId}/errors`),
  aiProvider: () => apiRequest<AiProviderStatus>("/api/v1/ai/provider"),
  aiCapabilities: () => apiRequest<AiCapability[]>("/api/v1/ai/capabilities"),
  aiRecommendations: () => apiRequest<AiRecommendation[]>("/api/v1/ai/recommendations"),
  aiPrompts: () => apiRequest<AiPromptTemplate[]>("/api/v1/ai/prompt-templates"),
  aiJobs: () => apiRequest<AiJob[]>("/api/v1/ai/jobs?limit=12"),
  updateAiJob: (id: string | number, payload: Record<string, unknown>) =>
    apiRequest<AiJob>(`/api/v1/ai/jobs/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload)
    }),
  approveAiJob: (id: string | number) =>
    apiRequest<AiJob>(`/api/v1/ai/jobs/${id}/approve`, {
      method: "POST"
    }),
  aiTaskDraft: (id: string | number) => apiRequest<AiTaskDraft>(`/api/v1/ai/jobs/${id}/task-draft`),
  aiNotificationDraft: (id: string | number) =>
    apiRequest<AiNotificationDraft>(`/api/v1/ai/jobs/${id}/notification-draft`),
  createVehicleAiAnalysis: (id: string, analysis_type = "vehicle_risk_summary") =>
    apiRequest<AiJob>(`/api/v1/ai/vehicles/${id}/analyze`, {
      method: "POST",
      body: JSON.stringify({
        analysis_type,
        requires_human_approval: true,
        note: "Frontend araç profilinden başlatıldı"
      })
    }),
  supportTicketAiContext: (id: string | number) => apiRequest<unknown>(`/api/v1/ai/support/tickets/${id}/context`),
  createSupportTicketAiAnalysis: (id: string | number, analysis_type = "support_ticket_triage") =>
    apiRequest<AiJob>(`/api/v1/ai/support/tickets/${id}/analyze`, {
      method: "POST",
      body: JSON.stringify({
        analysis_type,
        requires_human_approval: true,
        note: "Frontend IT destek ekranından başlatıldı"
      })
    })
};

export async function downloadReport(path: string, fileName: string) {
  const headers = new Headers();
  const token = getAccessToken();
  if (token) headers.set("Authorization", `Bearer ${token}`);

  const response = await fetch(`${API_BASE_URL}${path}`, { headers });
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }

  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

export async function uploadFile(payload: {
  module_name: string;
  entity_id: number;
  file_type: string;
  note?: string;
  file: File;
}) {
  const formData = new FormData();
  formData.set("module_name", payload.module_name);
  formData.set("entity_id", String(payload.entity_id));
  formData.set("file_type", payload.file_type);
  if (payload.note) formData.set("note", payload.note);
  formData.set("file", payload.file);

  const headers = new Headers();
  const token = getAccessToken();
  if (token) headers.set("Authorization", `Bearer ${token}`);

  const response = await fetch(`${API_BASE_URL}/api/v1/files/upload`, {
    method: "POST",
    headers,
    body: formData
  });

  if (!response.ok) {
    let detail = `${response.status} ${response.statusText}`;
    try {
      const error = await response.json();
      detail = error.error ?? error.message ?? detail;
    } catch {
      // ignore non-json error bodies
    }
    throw new Error(detail);
  }

  return response.json() as Promise<FileDocumentRow>;
}
