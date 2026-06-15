export type AuthSession = {
  access_token: string;
  refresh_token: string;
  token_type: string;
};

export type MobilePermission = {
  permission_key: string;
  company_id: number | null;
  department_id: number | null;
  can_view: boolean;
  can_create: boolean;
  can_update: boolean;
  can_approve: boolean;
};

export type CurrentUser = {
  id: number;
  email: string;
  full_name: string;
  role: string;
  company_id: number | null;
  department_id: number | null;
  is_active: boolean;
  mobile_permissions: MobilePermission[];
};

export type MobileTab = "dashboard" | "inventory" | "support" | "vehicles" | "account";

export type DashboardResponse = {
  generated_at: string;
  vehicles: {
    total_count: number;
    active_count: number;
    assigned_count: number;
  };
  tasks: {
    open_count: number;
    critical_count: number;
    overdue_count: number;
  };
  inventory: InventoryDashboard;
  support: {
    open_count: number;
    in_progress_count: number;
    waiting_user_count: number;
    critical_count: number;
    whatsapp_count: number;
    response_overdue_count: number;
    resolution_overdue_count: number;
    due_soon_count: number;
  };
  warnings: Array<{
    warning_type: string;
    severity: string;
    title: string;
    count: number;
  }>;
};

export type InventoryDashboard = {
  total_products: number;
  total_stock_value: string | number | null;
  critical_stock_count: number;
  out_of_stock_count: number;
  pending_shipments: number;
  pending_purchase_requests: number;
  expiring_lots_30_days: number;
};

export type InventoryProduct = {
  id: number;
  product_name: string;
  category: string | null;
  sub_category: string | null;
  brand: string | null;
  main_unit: string;
  package_unit: string | null;
  package_multiplier: string | number | null;
  current_stock: string | number;
  minimum_stock: string | number;
  critical_stock: string | number | null;
  is_active: boolean;
};

export type InventoryAlert = {
  product_id: number;
  product_name: string;
  current_stock: string | number;
  minimum_stock: string | number;
  critical_stock: string | number | null;
  alert_level: string;
  suggested_order_quantity: string | number;
};

export type InventoryPurchaseRequest = {
  id: number;
  product_id: number;
  product_name: string;
  requested_quantity: string | number;
  approved_quantity: string | number | null;
  request_status: string;
  reason: string | null;
  note: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type Vehicle = {
  id: number;
  plate: string;
  brand: string;
  model: string;
  model_year: number | null;
  status: string;
  user_name?: string | null;
  department_name?: string | null;
};

export type SupportTicket = {
  id: number;
  ticket_no: string;
  title: string;
  description: string;
  category: string;
  priority: string;
  ticket_status: string;
  source_channel: string;
  reporter_name: string | null;
  reporter_phone: string | null;
  assigned_user_name: string | null;
  assigned_department_name: string | null;
  resolution_note: string | null;
  sla_response_due_at: string | null;
  sla_resolution_due_at: string | null;
  first_response_at: string | null;
  escalation_level: number;
  satisfaction_score: number | null;
  sla_status: string;
  created_at: string;
  updated_at: string;
};

export type KmLog = {
  id: number;
  vehicle_id: number;
  km: number;
  entry_type: string;
  image_path: string | null;
  ocr_result: unknown;
  verification_status: string;
  device_info: string | null;
  created_at: string;
  created_by: number | null;
};

export type FileDocument = {
  id: number;
  module_name: string;
  entity_id: number;
  file_type: string;
  original_name: string;
  storage_path: string;
  mime_type: string | null;
  file_size: number;
  note: string | null;
  created_at: string;
};
