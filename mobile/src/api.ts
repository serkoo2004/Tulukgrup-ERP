import type {
  AuthSession,
  CurrentUser,
  DashboardResponse,
  FileDocument,
  InventoryAlert,
  InventoryDashboard,
  InventoryProduct,
  InventoryPurchaseRequest,
  KmLog,
  SupportTicket,
  Vehicle
} from "./types";

const fallbackBaseUrl = "http://127.0.0.1:8080";

export function defaultApiBaseUrl() {
  const envUrl = process.env.EXPO_PUBLIC_API_BASE_URL;
  return envUrl && envUrl.length > 0 ? envUrl : fallbackBaseUrl;
}

type RequestOptions = RequestInit & {
  auth?: boolean;
};

export class ApiClient {
  constructor(
    private readonly tokenProvider: () => string | null,
    private readonly baseUrlProvider: () => string = defaultApiBaseUrl
  ) {}

  async request<T>(path: string, options: RequestOptions = {}) {
    const headers = new Headers(options.headers);
    if (!headers.has("Content-Type") && options.body && !isFormData(options.body)) {
      headers.set("Content-Type", "application/json");
    }

    if (options.auth !== false) {
      const token = this.tokenProvider();
      if (token) headers.set("Authorization", `Bearer ${token}`);
    }

    const response = await fetch(`${this.baseUrlProvider().replace(/\/$/, "")}${path}`, { ...options, headers });
    if (!response.ok) {
      let detail = `${response.status} ${response.statusText}`;
      try {
        const payload = await response.json();
        detail = payload.error ?? payload.message ?? detail;
      } catch {
        // keep default detail
      }
      throw new Error(detail);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json() as Promise<T>;
  }

  login(email: string, password: string) {
    return this.request<AuthSession>("/api/v1/auth/login", {
      method: "POST",
      auth: false,
      body: JSON.stringify({ email, password })
    });
  }

  me() {
    return this.request<CurrentUser>("/api/v1/auth/me");
  }

  dashboard() {
    return this.request<DashboardResponse>("/api/v1/dashboard");
  }

  inventoryDashboard() {
    return this.request<InventoryDashboard>("/api/v1/inventory/dashboard");
  }

  inventoryProducts() {
    return this.request<InventoryProduct[]>("/api/v1/inventory/products?include_inactive=true&limit=200");
  }

  inventoryAlerts() {
    return this.request<InventoryAlert[]>("/api/v1/inventory/alerts");
  }

  inventoryPurchaseRequests() {
    return this.request<InventoryPurchaseRequest[]>("/api/v1/inventory/purchase-requests");
  }

  createInventoryProduct(payload: Record<string, unknown>) {
    return this.request<InventoryProduct>("/api/v1/inventory/products", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  createInventoryMovement(payload: Record<string, unknown>) {
    return this.request("/api/v1/inventory/movements", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  createInventoryPurchaseRequest(payload: Record<string, unknown>) {
    return this.request<InventoryPurchaseRequest>("/api/v1/inventory/purchase-requests", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  vehicles() {
    return this.request<Vehicle[]>("/api/v1/vehicles?status=all&limit=100");
  }

  createKmLog(payload: Record<string, unknown>) {
    return this.request<KmLog>("/api/v1/tracking/km-logs", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  supportTickets() {
    return this.request<SupportTicket[]>("/api/v1/support/tickets?limit=80");
  }

  createSupportTicket(payload: Record<string, unknown>) {
    return this.request<SupportTicket>("/api/v1/support/tickets", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  uploadFile(payload: {
    module_name: string;
    entity_id: number;
    file_type: string;
    note?: string;
    uri: string;
    name: string;
    mimeType: string;
  }) {
    const formData = new FormData();
    formData.append("module_name", payload.module_name);
    formData.append("entity_id", String(payload.entity_id));
    formData.append("file_type", payload.file_type);
    if (payload.note) formData.append("note", payload.note);
    formData.append("file", {
      uri: payload.uri,
      name: payload.name,
      type: payload.mimeType
    } as unknown as Blob);

    return this.request<FileDocument>("/api/v1/files/upload", {
      method: "POST",
      body: formData
    });
  }
}

function isFormData(body: BodyInit) {
  return typeof FormData !== "undefined" && body instanceof FormData;
}
