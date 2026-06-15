export type LoginResponse = {
  access_token: string;
  refresh_token: string;
  token_type: string;
};

export type CurrentUser = {
  id: number;
  email: string;
  full_name: string;
  role: string;
  company_id: number | null;
  department_id: number | null;
  is_active: boolean;
  mobile_permissions?: MobilePermissionRow[];
};

export type UserRow = CurrentUser & {
  phone: string | null;
  extension: string | null;
  company_id: number | null;
  department_id: number | null;
  created_at: string;
  updated_at: string;
};

export type RoleRow = {
  key: string;
  label: string;
};

export type UserCreateInput = {
  email: string;
  full_name: string;
  password: string;
  role: string;
  phone?: string;
  extension?: string;
  company_id?: number;
  department_id?: number;
};

export type UserUpdateInput = Partial<Omit<UserCreateInput, "email" | "password">> & {
  is_active?: boolean;
};

export type MobilePermissionRow = {
  id?: number;
  user_id?: number;
  company_id: number | null;
  company_name?: string | null;
  department_id: number | null;
  department_name?: string | null;
  permission_key: string;
  can_view: boolean;
  can_create: boolean;
  can_update: boolean;
  can_approve: boolean;
  created_at?: string;
  updated_at?: string;
  created_by?: number | null;
};

export type MobilePermissionInput = {
  company_id?: number | null;
  department_id?: number | null;
  permission_key: string;
  can_view?: boolean;
  can_create?: boolean;
  can_update?: boolean;
  can_approve?: boolean;
};

export type SettingRow = {
  id: number;
  setting_key: string;
  setting_value: unknown;
  description: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type SystemLogRow = {
  id: number;
  service_name: string;
  severity: string;
  message: string;
  context: unknown;
  created_at: string;
};

export type FileDocumentRow = {
  id: number;
  module_name: string;
  entity_id: number;
  file_type: string;
  original_name: string;
  storage_path: string;
  mime_type: string | null;
  file_size: number;
  checksum_sha256: string | null;
  note: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type ImportJobRow = {
  id: number;
  target_module: string;
  source_file_id: number | null;
  column_mapping: unknown;
  import_status: string;
  total_rows: number;
  processed_rows: number;
  error_count: number;
  note: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type ImportValidationSummary = {
  import_job_id: number;
  total_rows: number;
  valid_rows: number;
  invalid_rows: number;
  error_count: number;
};

export type ImportErrorRow = {
  id: number;
  import_job_id: number;
  row_number: number | null;
  field_name: string | null;
  error_message: string;
  raw_data: unknown;
  created_at: string;
};

export type ImportStagingRow = {
  id: number;
  import_job_id: number;
  row_number: number;
  raw_data: unknown;
  normalized_data: unknown;
  row_status: string;
  error_count: number;
  created_at: string;
  updated_at: string;
};

export type AuditLogRow = {
  id: number;
  table_name: string;
  record_id: number | null;
  action_type: string;
  old_data: unknown;
  new_data: unknown;
  ip_address: string | null;
  user_agent: string | null;
  created_by: number | null;
  created_at: string;
};

export type CompanyRow = {
  id: number;
  name: string;
  tax_number: string | null;
  created_at: string;
  updated_at: string;
  is_active: boolean;
};

export type DepartmentRow = {
  id: number;
  company_id: number | null;
  name: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
};

export type KmLogRow = {
  id: number;
  vehicle_id: number;
  km: number;
  entry_type: string;
  image_path: string | null;
  ocr_result: unknown;
  verification_status: string;
  ip_address: string | null;
  device_info: string | null;
  created_at: string;
  created_by: number | null;
};

export type AssignmentRow = {
  id: number;
  vehicle_id: number;
  user_id: number;
  assigned_at: string;
  released_at: string | null;
  note: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type InventoryProductRow = {
  id: number;
  product_code: string;
  barcode: string | null;
  qr_code: string | null;
  product_name: string;
  category: string | null;
  sub_category: string | null;
  brand: string | null;
  description: string | null;
  main_unit: string;
  package_unit: string | null;
  package_multiplier: string | number | null;
  current_stock: string | number;
  available_stock: string | number;
  reserved_stock: string | number;
  minimum_stock: string | number;
  maximum_stock: string | number | null;
  critical_stock: string | number | null;
  safety_stock: string | number | null;
  unit_cost: string | number | null;
  is_lot_tracked: boolean;
  expiry_tracking: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type InventoryMovementRow = {
  id: number;
  product_id: number;
  product_name: string;
  product_code: string;
  lot_id: number | null;
  lot_number: string | null;
  movement_type: string;
  quantity: string | number;
  unit: string;
  unit_multiplier: string | number;
  base_quantity: string | number;
  previous_stock: string | number;
  next_stock: string | number;
  branch_id: number | null;
  branch_name: string | null;
  vehicle_id: number | null;
  plate: string | null;
  reference_table: string | null;
  reference_id: number | null;
  description: string | null;
  movement_at: string;
  cancelled_at: string | null;
  cancel_reason: string | null;
  created_at: string;
  created_by: number | null;
};

export type InventoryLotRow = {
  id: number;
  product_id: number;
  product_name: string;
  lot_number: string;
  production_date: string | null;
  expiry_date: string | null;
  supplier: string | null;
  quantity: string | number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type InventoryShipmentRow = {
  id: number;
  branch_id: number | null;
  branch_name: string | null;
  shipment_status: string;
  sender_user_id: number | null;
  approver_user_id: number | null;
  shipped_at: string | null;
  approved_at: string | null;
  note: string | null;
  item_count: number;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type InventoryShipmentItemRow = {
  id: number;
  shipment_id: number;
  product_id: number;
  product_name: string;
  quantity: string | number;
  unit: string;
  unit_multiplier: string | number;
  base_quantity: string | number;
};

export type InventoryShipmentDetail = {
  shipment: InventoryShipmentRow;
  items: InventoryShipmentItemRow[];
};

export type InventoryCountRow = {
  id: number;
  count_type: string;
  count_method: string;
  category: string | null;
  count_status: string;
  note: string | null;
  counted_at: string | null;
  item_count: number;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type InventoryCountItemRow = {
  id: number;
  count_id: number;
  product_id: number;
  product_name: string;
  system_stock: string | number;
  counted_stock: string | number;
  difference: string | number;
  reason: string | null;
};

export type InventoryCountDetail = {
  count: InventoryCountRow;
  items: InventoryCountItemRow[];
};

export type InventoryAlertRow = {
  product_id: number;
  product_code: string;
  product_name: string;
  current_stock: string | number;
  minimum_stock: string | number;
  critical_stock: string | number | null;
  alert_level: string;
  suggested_order_quantity: string | number;
};

export type InventoryPurchaseSuggestionRow = InventoryAlertRow & {
  monthly_consumption_3m: string | number;
  monthly_consumption_6m: string | number;
  monthly_consumption_12m: string | number;
  average_monthly_consumption: string | number;
  estimated_runout_date: string | null;
};

export type InventoryPurchaseRequestRow = {
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

export type InventoryDashboardRow = {
  total_products: number;
  total_stock_value: string | number | null;
  critical_stock_count: number;
  out_of_stock_count: number;
  pending_shipments: number;
  pending_purchase_requests: number;
  expiring_lots_30_days: number;
  recent_movements: InventoryMovementRow[];
  recent_notifications: InventoryNotificationRow[];
};

export type InventoryNotificationRow = {
  id: number;
  notification_type: string;
  message: string;
  sent_via: string;
  delivery_status: string;
  created_at: string;
  read_at: string | null;
};

export type NotificationRow = {
  id: number;
  notification_type: string;
  receiver_user_id: number | null;
  related_vehicle_id: number | null;
  message: string;
  sent_via: string;
  delivery_status: string;
  recipient_phone: string | null;
  external_message_id: string | null;
  provider_payload: unknown;
  delivery_error: string | null;
  sent_at: string | null;
  read_at: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type DashboardResponse = {
  generated_at: string;
  vehicles: {
    total_count: number;
    active_count: number;
    passive_count: number;
    sold_count: number;
    assigned_count: number;
    unassigned_count: number;
  };
  tasks: {
    open_count: number;
    critical_count: number;
    overdue_count: number;
  };
  policies: {
    active_count: number;
    ending_in_30_days: number;
    ended_count: number;
  };
  maintenances: {
    planned_count: number;
    scheduled_count: number;
    completed_count: number;
    upcoming_by_km_count: number;
  };
  expenses: {
    current_month_total: string | null;
    pending_count: number;
    overdue_count: number;
  };
  damages: {
    open_count: number;
    insurance_count: number;
    estimated_open_cost: string | null;
  };
  operations: {
    open_inspection_count: number;
    open_value_loss_count: number;
    current_month_fuel_total: string | null;
    current_month_wash_total: string | null;
    pending_quote_count: number;
  };
  inventory: InventoryDashboardRow;
  support: {
    open_count: number;
    in_progress_count: number;
    waiting_user_count: number;
    critical_count: number;
    whatsapp_count: number;
    response_overdue_count: number;
    resolution_overdue_count: number;
    due_soon_count: number;
    resolved_today_count: number;
  };
  warnings: Array<{
    warning_type: string;
    severity: string;
    title: string;
    count: number;
  }>;
  recent_activity: Array<{
    table_name: string;
    record_id: number | null;
    action_type: string;
    created_by: number | null;
    created_at: string;
  }>;
};

export type Vehicle = {
  id: number;
  plate: string;
  brand: string;
  model: string;
  model_year: number | null;
  vehicle_type: string | null;
  fuel_type: string | null;
  transmission: string | null;
  chassis_no: string | null;
  engine_no: string | null;
  warranty_status: string | null;
  warranty_end: string | null;
  has_hgs: boolean;
  has_mobiliz: boolean;
  has_kopilot: boolean;
  has_k2: boolean;
  tasitmatik_company: string | null;
  spare_key_location: string | null;
  status: string;
  company_id: number | null;
  department_id: number | null;
  user_id: number | null;
  created_at: string;
  updated_at: string;
  is_active: boolean;
};

export type VehicleCreateInput = {
  plate: string;
  brand: string;
  model: string;
  model_year?: number;
  vehicle_type?: string;
  fuel_type?: string;
  transmission?: string;
  chassis_no?: string;
  engine_no?: string;
  warranty_status?: string;
  warranty_end?: string;
  has_hgs?: boolean;
  has_mobiliz?: boolean;
  has_kopilot?: boolean;
  has_k2?: boolean;
  tasitmatik_company?: string;
  spare_key_location?: string;
  company_id?: number;
  department_id?: number;
  user_id?: number;
};

export type VehicleUpdateInput = Partial<Omit<VehicleCreateInput, "plate">>;

export type VehicleSellInput = {
  sold_date: string;
  sold_reason: string;
  sold_price?: number;
  buyer_info?: string;
  company_exit_reason: string;
  note?: string;
};

export type SearchResult = {
  result_type: string;
  id: number;
  title: string;
  subtitle: string | null;
  status: string | null;
  created_at: string | null;
};

export type VehicleProfile = {
  vehicle: Vehicle;
  km_summary: {
    total_logs: number;
    last_km: number | null;
    suspicious_logs: number;
  };
  maintenance_summary: {
    total_records: number;
    completed_records: number;
    total_cost: string | null;
  };
  policy_summary: {
    total_policies: number;
    active_policies: number;
    latest_end_date: string | null;
  };
  expense_summary: {
    total_records: number;
    total_amount: string | null;
  };
  damage_summary: {
    total_records: number;
    open_records: number;
    total_estimated_cost: string | null;
    total_actual_cost: string | null;
  };
  operations_summary: {
    inspection_count: number;
    open_inspection_count: number;
    value_loss_count: number;
    open_value_loss_count: number;
    fuel_record_count: number;
    wash_count: number;
    quote_count: number;
    pending_quote_count: number;
  };
  file_summary: {
    total_files: number;
    vehicle_files: number;
    expense_files: number;
    damage_files: number;
  };
  recent_tasks: Array<{
    id: number;
    task_type: string;
    priority: string;
    due_date: string | null;
    task_status: string;
    description: string | null;
    created_at: string;
  }>;
  recent_km_logs: Array<{
    id: number;
    km: number;
    entry_type: string;
    verification_status: string;
    created_at: string;
  }>;
};

export type OperationRecord = Record<string, string | number | boolean | null>;

export type AiProviderStatus = {
  provider: string;
  model: string;
  configured: boolean;
  execution_mode: string;
  base_url_configured: boolean;
  api_key_configured: boolean;
  timeout_seconds: number;
  supports_external_models: boolean;
  supports_local_models: boolean;
  required_env: string[];
  missing_env: string[];
  next_step: string;
};

export type AiCapability = {
  analysis_type: string;
  title: string;
  target_modules: string[];
  human_approval_required: boolean;
  description: string;
};

export type AiRecommendation = {
  recommendation_type: string;
  severity: string;
  title: string;
  detail: string;
  source_module: string;
  source_count: number;
  suggested_analysis_type: string;
};

export type AiPromptTemplate = {
  analysis_type: string;
  system_goal: string;
  required_context: string[];
  output_contract: string[];
};

export type AiJob = {
  id: number;
  analysis_type: string;
  related_vehicle_id: number | null;
  source_table: string | null;
  source_record_id: number | null;
  input_data: unknown;
  result_data: unknown;
  confidence_score: string | null;
  job_status: string;
  requires_human_approval: boolean;
  approved_by: number | null;
  approved_at: string | null;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type AiTaskDraft = {
  task_type: string;
  related_vehicle_id: number | null;
  priority: string;
  description: string;
  source_ai_job_id: number;
};

export type AiNotificationDraft = {
  notification_type: string;
  related_vehicle_id: number | null;
  sent_via: string;
  delivery_status: string;
  message: string;
  source_ai_job_id: number;
};

export type SupportTicketRow = {
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
  reporter_user_id: number | null;
  assigned_user_id: number | null;
  assigned_user_name: string | null;
  assigned_department_id: number | null;
  assigned_department_name: string | null;
  source_notification_id: number | null;
  external_message_id: string | null;
  resolution_note: string | null;
  resolved_at: string | null;
  closed_at: string | null;
  sla_response_due_at: string | null;
  sla_resolution_due_at: string | null;
  first_response_at: string | null;
  escalation_level: number;
  satisfaction_score: number | null;
  satisfaction_note: string | null;
  sla_status: string;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type SupportTicketEventRow = {
  id: number;
  ticket_id: number;
  event_type: string;
  note: string | null;
  old_status: string | null;
  new_status: string | null;
  created_at: string;
  created_by: number | null;
};

export type SupportSummary = {
  total_count: number;
  open_count: number;
  in_progress_count: number;
  waiting_user_count: number;
  resolved_today_count: number;
  closed_count: number;
  critical_count: number;
  whatsapp_open_count: number;
  unassigned_count: number;
  response_overdue_count: number;
  resolution_overdue_count: number;
  due_soon_count: number;
  avg_satisfaction: number | null;
  avg_resolution_hours: number | null;
};

export type SupportKnowledgeBaseRow = {
  id: number;
  title: string;
  category: string;
  problem: string;
  solution: string;
  tags: unknown;
  source_ticket_id: number | null;
  source_ticket_no: string | null;
  is_published: boolean;
  view_count: number;
  created_at: string;
  updated_at: string;
  created_by: number | null;
};

export type SupportTicketReplyResponse = {
  ticket: SupportTicketRow;
  notification: NotificationRow;
};
