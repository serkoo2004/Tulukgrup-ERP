use axum::{response::Html, Json};
use serde_json::{json, Value};

pub async fn swagger_ui() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

pub async fn openapi_json() -> Json<Value> {
    Json(openapi_spec())
}

fn openapi_spec() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Tuluklar ERP API",
            "version": "0.1.0",
            "description": "Rust backend API for Tuluklar ERP. Use /api/v1/auth/login first, then Authorize with the returned access_token as Bearer token."
        },
        "servers": [
            { "url": "http://127.0.0.1:8080", "description": "Local backend" },
            { "url": "http://localhost", "description": "Nginx / Docker gateway" }
        ],
        "tags": [
            { "name": "Health" },
            { "name": "Auth" },
            { "name": "Users" },
            { "name": "Organization" },
            { "name": "Vehicles" },
            { "name": "Assignments" },
            { "name": "Dashboard" },
            { "name": "Tracking" },
            { "name": "Maintenances" },
            { "name": "Insurance" },
            { "name": "Expenses" },
            { "name": "Damages" },
            { "name": "Tasks" },
            { "name": "Notifications" },
            { "name": "Operations" },
            { "name": "Files" },
            { "name": "Inventory" },
            { "name": "Reports" },
            { "name": "Imports" },
            { "name": "Settings" },
            { "name": "Search" },
            { "name": "System Logs" },
            { "name": "IT Support" },
            { "name": "AI" },
            { "name": "Audit" }
        ],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            },
            "schemas": schemas()
        },
        "paths": paths()
    })
}

fn schemas() -> Value {
    json!({
        "LoginRequest": {
            "type": "object",
            "required": ["email", "password"],
            "properties": {
                "email": { "type": "string", "example": "admin@tuluklar.local" },
                "password": { "type": "string", "example": "Admin12345!" }
            }
        },
        "LoginResponse": {
            "type": "object",
            "properties": {
                "access_token": { "type": "string" },
                "refresh_token": { "type": "string" },
                "token_type": { "type": "string", "example": "bearer" }
            }
        },
        "RefreshRequest": {
            "type": "object",
            "required": ["refresh_token"],
            "properties": {
                "refresh_token": { "type": "string" }
            }
        },
        "ChangePasswordRequest": {
            "type": "object",
            "required": ["current_password", "new_password"],
            "properties": {
                "current_password": { "type": "string" },
                "new_password": { "type": "string", "example": "NewPassword123!" }
            }
        },
        "CompanyCreate": {
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string", "example": "Tuluklar Group" },
                "tax_number": { "type": "string", "nullable": true, "example": "1234567890" }
            }
        },
        "DepartmentCreate": {
            "type": "object",
            "required": ["name"],
            "properties": {
                "company_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "name": { "type": "string", "example": "Operasyon" }
            }
        },
        "UserCreate": {
            "type": "object",
            "required": ["email", "full_name", "password", "role"],
            "properties": {
                "email": { "type": "string", "example": "operasyon@tuluklar.local" },
                "full_name": { "type": "string", "example": "Operasyon Kullanıcısı" },
                "password": { "type": "string", "example": "User12345!" },
                "role": { "type": "string", "enum": ["admin", "manager", "operation", "accounting", "user"], "example": "operation" },
                "phone": { "type": "string", "nullable": true },
                "extension": { "type": "string", "nullable": true },
                "company_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "department_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 }
            }
        },
        "UserUpdate": {
            "type": "object",
            "properties": {
                "full_name": { "type": "string", "nullable": true, "example": "Operasyon Sorumlusu" },
                "role": { "type": "string", "enum": ["admin", "manager", "operation", "accounting", "user"], "nullable": true },
                "phone": { "type": "string", "nullable": true },
                "extension": { "type": "string", "nullable": true },
                "company_id": { "type": "integer", "format": "int64", "nullable": true },
                "department_id": { "type": "integer", "format": "int64", "nullable": true },
                "is_active": { "type": "boolean", "nullable": true, "example": true }
            }
        },
        "MobilePermissionUpsert": {
            "type": "object",
            "required": ["permission_key"],
            "properties": {
                "company_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "department_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "permission_key": {
                    "type": "string",
                    "enum": ["dashboard", "inventory_view", "inventory_order", "vehicle_view", "vehicle_fault", "km_log", "support_ticket", "media_upload"],
                    "example": "inventory_order"
                },
                "can_view": { "type": "boolean", "nullable": true, "example": true },
                "can_create": { "type": "boolean", "nullable": true, "example": true },
                "can_update": { "type": "boolean", "nullable": true, "example": false },
                "can_approve": { "type": "boolean", "nullable": true, "example": false }
            }
        },
        "VehicleCreate": {
            "type": "object",
            "required": ["plate", "brand", "model"],
            "properties": {
                "plate": { "type": "string", "example": "34ABC123" },
                "brand": { "type": "string", "example": "Ford" },
                "model": { "type": "string", "example": "Transit" },
                "model_year": { "type": "integer", "nullable": true, "example": 2024 },
                "vehicle_type": { "type": "string", "nullable": true, "example": "van" },
                "fuel_type": { "type": "string", "nullable": true, "example": "diesel" },
                "transmission": { "type": "string", "nullable": true, "example": "manual" },
                "chassis_no": { "type": "string", "nullable": true },
                "engine_no": { "type": "string", "nullable": true },
                "warranty_status": { "type": "string", "nullable": true, "example": "active" },
                "warranty_end": { "type": "string", "format": "date", "nullable": true, "example": "2027-05-24" },
                "has_hgs": { "type": "boolean", "nullable": true, "example": true },
                "has_mobiliz": { "type": "boolean", "nullable": true, "example": true },
                "has_kopilot": { "type": "boolean", "nullable": true, "example": false },
                "has_k2": { "type": "boolean", "nullable": true, "example": false },
                "tasitmatik_company": { "type": "string", "nullable": true, "example": "Shell" },
                "spare_key_location": { "type": "string", "nullable": true, "example": "Merkez" },
                "company_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "department_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "user_id": { "type": "integer", "format": "int64", "nullable": true }
            }
        },
        "VehicleUpdate": {
            "type": "object",
            "properties": {
                "brand": { "type": "string", "nullable": true, "example": "Ford" },
                "model": { "type": "string", "nullable": true, "example": "Transit" },
                "model_year": { "type": "integer", "nullable": true, "example": 2024 },
                "vehicle_type": { "type": "string", "nullable": true, "example": "van" },
                "fuel_type": { "type": "string", "nullable": true, "example": "diesel" },
                "transmission": { "type": "string", "nullable": true, "example": "manual" },
                "chassis_no": { "type": "string", "nullable": true },
                "engine_no": { "type": "string", "nullable": true },
                "warranty_status": { "type": "string", "nullable": true },
                "warranty_end": { "type": "string", "format": "date", "nullable": true },
                "has_hgs": { "type": "boolean", "nullable": true },
                "has_mobiliz": { "type": "boolean", "nullable": true },
                "has_kopilot": { "type": "boolean", "nullable": true },
                "has_k2": { "type": "boolean", "nullable": true },
                "tasitmatik_company": { "type": "string", "nullable": true },
                "spare_key_location": { "type": "string", "nullable": true },
                "company_id": { "type": "integer", "format": "int64", "nullable": true },
                "department_id": { "type": "integer", "format": "int64", "nullable": true },
                "user_id": { "type": "integer", "format": "int64", "nullable": true }
            }
        },
        "SellVehicleRequest": {
            "type": "object",
            "required": ["sold_date", "sold_reason", "company_exit_reason"],
            "properties": {
                "sold_date": { "type": "string", "format": "date", "example": "2026-05-24" },
                "sold_reason": { "type": "string", "example": "renewal" },
                "sold_price": { "type": "number", "nullable": true, "example": 500000 },
                "buyer_info": { "type": "string", "nullable": true },
                "company_exit_reason": { "type": "string", "example": "sold" },
                "note": { "type": "string", "nullable": true }
            }
        },
        "AssignmentCreate": {
            "type": "object",
            "required": ["vehicle_id", "user_id"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "user_id": { "type": "integer", "format": "int64", "example": 1 },
                "note": { "type": "string", "nullable": true }
            }
        },
        "KmLogCreate": {
            "type": "object",
            "required": ["vehicle_id", "km"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "km": { "type": "integer", "example": 1200 },
                "entry_type": { "type": "string", "enum": ["manual", "ocr", "mobiliz_api", "kopilot_api"], "example": "manual" },
                "image_path": { "type": "string", "nullable": true, "example": "vehicles/1/km_photo/example.jpg" },
                "ocr_result": { "type": "object", "nullable": true, "example": { "status": "pending", "source": "mobile_km_photo" } },
                "device_info": { "type": "string", "nullable": true }
            }
        },
        "MaintenanceCreate": {
            "type": "object",
            "required": ["vehicle_id"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "last_maintenance_km": { "type": "integer", "nullable": true, "example": 95000 },
                "next_maintenance_km": { "type": "integer", "nullable": true, "example": 105000 },
                "maintenance_date": { "type": "string", "format": "date", "nullable": true, "example": "2026-06-01" },
                "service_company": { "type": "string", "nullable": true, "example": "Yetkili Servis" },
                "maintenance_type": { "type": "string", "example": "periodic" },
                "description": { "type": "string", "nullable": true, "example": "Periyodik bakım ve yağ değişimi" },
                "total_cost": { "type": "number", "nullable": true, "example": 7500 },
                "invoice_file": { "type": "string", "nullable": true },
                "maintenance_status": { "type": "string", "enum": ["planned", "scheduled", "completed", "cancelled"], "nullable": true, "example": "planned" }
            }
        },
        "MaintenanceStatusUpdate": {
            "type": "object",
            "required": ["maintenance_status"],
            "properties": {
                "maintenance_status": { "type": "string", "enum": ["planned", "scheduled", "completed", "cancelled"], "example": "completed" }
            }
        },
        "PolicyCreate": {
            "type": "object",
            "required": ["vehicle_id", "policy_type", "policy_number"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "policy_type": { "type": "string", "enum": ["trafik", "kasko", "imm", "ferdi_kaza"], "example": "trafik" },
                "policy_number": { "type": "string", "example": "POL-2026-001" },
                "insurance_company": { "type": "string", "nullable": true, "example": "Sigorta A.Ş." },
                "agency_name": { "type": "string", "nullable": true, "example": "Merkez Acente" },
                "start_date": { "type": "string", "format": "date", "nullable": true, "example": "2026-01-01" },
                "end_date": { "type": "string", "format": "date", "nullable": true, "example": "2026-12-31" },
                "amount": { "type": "number", "nullable": true, "example": 18500 },
                "previous_amount": { "type": "number", "nullable": true, "example": 16200 },
                "currency": { "type": "string", "nullable": true, "example": "TRY" },
                "pdf_file": { "type": "string", "nullable": true },
                "renewal_status": { "type": "string", "enum": ["active", "approaching", "renewing", "ended"], "nullable": true, "example": "active" }
            }
        },
        "RenewalStatusUpdate": {
            "type": "object",
            "required": ["renewal_status"],
            "properties": {
                "renewal_status": { "type": "string", "enum": ["active", "approaching", "renewing", "ended"], "example": "renewing" }
            }
        },
        "ExpenseCreate": {
            "type": "object",
            "required": ["vehicle_id", "expense_type", "amount", "expense_date"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "expense_type": { "type": "string", "enum": ["fuel", "insurance", "maintenance", "tax", "fine", "tire", "service"], "example": "fuel" },
                "amount": { "type": "number", "example": 2500 },
                "invoice_number": { "type": "string", "nullable": true },
                "invoice_file": { "type": "string", "nullable": true },
                "payment_status": { "type": "string", "nullable": true, "example": "pending" },
                "expense_date": { "type": "string", "format": "date", "example": "2026-05-24" },
                "note": { "type": "string", "nullable": true }
            }
        },
        "DamageCreate": {
            "type": "object",
            "required": ["vehicle_id", "damage_date"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "damage_date": { "type": "string", "format": "date", "example": "2026-05-24" },
                "damage_type": { "type": "string", "nullable": true, "example": "kaporta" },
                "description": { "type": "string", "example": "On tampon hasari" },
                "estimated_cost": { "type": "number", "nullable": true },
                "actual_cost": { "type": "number", "nullable": true },
                "insurance_claim_no": { "type": "string", "nullable": true },
                "damage_status": { "type": "string", "enum": ["open", "expertise", "insurance", "repaired", "closed", "cancelled"], "nullable": true, "example": "open" }
            }
        },
        "DamageUpdate": {
            "type": "object",
            "properties": {
                "damage_status": { "type": "string", "enum": ["open", "expertise", "insurance", "repaired", "closed", "cancelled"], "nullable": true, "example": "insurance" },
                "estimated_cost": { "type": "number", "nullable": true },
                "actual_cost": { "type": "number", "nullable": true },
                "insurance_claim_no": { "type": "string", "nullable": true },
                "description": { "type": "string", "nullable": true }
            }
        },
        "PaymentStatusUpdate": {
            "type": "object",
            "required": ["payment_status"],
            "properties": {
                "payment_status": { "type": "string", "example": "paid" }
            }
        },
        "VehicleInspectionCreate": {
            "type": "object",
            "required": ["vehicle_id", "inspection_date"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "branch": { "type": "string", "nullable": true, "example": "Merkez" },
                "inspection_date": { "type": "string", "format": "date", "example": "2026-05-25" },
                "inspector_user_id": { "type": "integer", "format": "int64", "nullable": true },
                "exterior_ok": { "type": "boolean", "nullable": true, "example": true },
                "interior_ok": { "type": "boolean", "nullable": true, "example": true },
                "equipment_ok": { "type": "boolean", "nullable": true, "example": true },
                "documents_ok": { "type": "boolean", "nullable": true, "example": true },
                "damage_note": { "type": "string", "nullable": true },
                "action_note": { "type": "string", "nullable": true },
                "expert_report": { "type": "string", "nullable": true },
                "fee": { "type": "number", "nullable": true },
                "inspection_status": { "type": "string", "nullable": true, "example": "open" }
            }
        },
        "VehicleInspectionUpdate": {
            "type": "object",
            "properties": {
                "branch": { "type": "string", "nullable": true },
                "inspection_date": { "type": "string", "format": "date", "nullable": true },
                "inspection_status": { "type": "string", "nullable": true, "example": "completed" },
                "damage_note": { "type": "string", "nullable": true },
                "action_note": { "type": "string", "nullable": true },
                "fee": { "type": "number", "nullable": true }
            }
        },
        "ValueLossClaimCreate": {
            "type": "object",
            "required": ["vehicle_id", "accident_date"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "accident_date": { "type": "string", "format": "date", "example": "2026-05-25" },
                "vehicle_purchase_date": { "type": "string", "format": "date", "nullable": true },
                "tramer_amount": { "type": "number", "nullable": true },
                "deprivation_days": { "type": "integer", "nullable": true },
                "requested_amount": { "type": "number", "nullable": true },
                "received_amount": { "type": "number", "nullable": true },
                "claim_status": { "type": "string", "nullable": true, "example": "open" },
                "note": { "type": "string", "nullable": true }
            }
        },
        "ValueLossClaimUpdate": {
            "type": "object",
            "properties": {
                "claim_status": { "type": "string", "nullable": true, "example": "lawsuit" },
                "requested_amount": { "type": "number", "nullable": true },
                "received_amount": { "type": "number", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "FuelEntryCreate": {
            "type": "object",
            "required": ["vehicle_id", "period_year", "period_month"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "period_year": { "type": "integer", "example": 2026 },
                "period_month": { "type": "integer", "minimum": 1, "maximum": 12, "example": 5 },
                "fuel_limit": { "type": "number", "nullable": true },
                "paid_amount": { "type": "number", "nullable": true },
                "remaining_limit": { "type": "number", "nullable": true },
                "distance_km": { "type": "integer", "nullable": true },
                "current_km": { "type": "integer", "nullable": true },
                "liter_amount": { "type": "number", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "FuelEntryUpdate": {
            "type": "object",
            "properties": {
                "fuel_limit": { "type": "number", "nullable": true },
                "paid_amount": { "type": "number", "nullable": true },
                "remaining_limit": { "type": "number", "nullable": true },
                "distance_km": { "type": "integer", "nullable": true },
                "current_km": { "type": "integer", "nullable": true },
                "liter_amount": { "type": "number", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "VehicleWashCreate": {
            "type": "object",
            "required": ["vehicle_id", "wash_date"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "branch": { "type": "string", "nullable": true },
                "wash_company": { "type": "string", "nullable": true },
                "wash_date": { "type": "string", "format": "date", "example": "2026-05-25" },
                "amount": { "type": "number", "nullable": true },
                "user_id": { "type": "integer", "format": "int64", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "VehicleWashUpdate": {
            "type": "object",
            "properties": {
                "branch": { "type": "string", "nullable": true },
                "wash_company": { "type": "string", "nullable": true },
                "wash_date": { "type": "string", "format": "date", "nullable": true },
                "amount": { "type": "number", "nullable": true },
                "user_id": { "type": "integer", "format": "int64", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "InsuranceQuoteCreate": {
            "type": "object",
            "required": ["vehicle_id"],
            "properties": {
                "vehicle_id": { "type": "integer", "format": "int64", "example": 1 },
                "quote_type": { "type": "string", "nullable": true, "example": "kasko" },
                "insurance_company": { "type": "string", "nullable": true },
                "agency_name": { "type": "string", "nullable": true },
                "gross_premium": { "type": "number", "nullable": true },
                "installment_count": { "type": "integer", "nullable": true },
                "quote_status": { "type": "string", "nullable": true, "example": "pending" },
                "valid_until": { "type": "string", "format": "date", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "InsuranceQuoteUpdate": {
            "type": "object",
            "properties": {
                "quote_type": { "type": "string", "nullable": true },
                "insurance_company": { "type": "string", "nullable": true },
                "agency_name": { "type": "string", "nullable": true },
                "gross_premium": { "type": "number", "nullable": true },
                "installment_count": { "type": "integer", "nullable": true },
                "quote_status": { "type": "string", "nullable": true, "example": "approved" },
                "valid_until": { "type": "string", "format": "date", "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "ReleaseAssignmentRequest": {
            "type": "object",
            "properties": {
                "note": { "type": "string", "nullable": true, "example": "Araç teslim alındı" }
            }
        },
        "VerifyKmLogRequest": {
            "type": "object",
            "required": ["verification_status"],
            "properties": {
                "verification_status": { "type": "string", "enum": ["pending", "verified", "suspicious", "rejected"], "example": "verified" },
                "note": { "type": "string", "nullable": true }
            }
        },
        "TaskCreate": {
            "type": "object",
            "required": ["task_type"],
            "properties": {
                "task_type": { "type": "string", "example": "vehicle_check" },
                "related_vehicle_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "assigned_user_id": { "type": "integer", "format": "int64", "nullable": true },
                "assigned_department_id": { "type": "integer", "format": "int64", "nullable": true },
                "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"], "example": "medium" },
                "due_date": { "type": "string", "format": "date", "nullable": true },
                "task_status": { "type": "string", "nullable": true, "example": "open" },
                "description": { "type": "string", "nullable": true }
            }
        },
        "NotificationCreate": {
            "type": "object",
            "required": ["notification_type", "message", "sent_via"],
            "properties": {
                "notification_type": { "type": "string", "example": "inventory_stock_alert" },
                "receiver_user_id": { "type": "integer", "format": "int64", "nullable": true },
                "related_vehicle_id": { "type": "integer", "format": "int64", "nullable": true },
                "message": { "type": "string", "example": "Kritik stok: Plastik Bardak mevcut 4, minimum 5." },
                "sent_via": { "type": "string", "enum": ["whatsapp", "system", "email"], "example": "system" },
                "delivery_status": { "type": "string", "enum": ["pending", "queued", "sent", "failed", "cancelled"], "nullable": true, "example": "pending" },
                "recipient_phone": { "type": "string", "nullable": true, "example": "+905551112233" },
                "provider_payload": { "type": "object", "nullable": true }
            }
        },
        "NotificationDeliveryUpdate": {
            "type": "object",
            "required": ["delivery_status"],
            "properties": {
                "delivery_status": { "type": "string", "enum": ["pending", "queued", "sent", "failed", "cancelled"], "example": "sent" },
                "external_message_id": { "type": "string", "nullable": true },
                "delivery_error": { "type": "string", "nullable": true },
                "provider_payload": { "type": "object", "nullable": true }
            }
        },
        "AiJobCreate": {
            "type": "object",
            "required": ["analysis_type"],
            "properties": {
                "analysis_type": { "type": "string", "example": "vehicle_risk_summary" },
                "related_vehicle_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "source_table": { "type": "string", "nullable": true, "example": "vehicles" },
                "source_record_id": { "type": "integer", "format": "int64", "nullable": true, "example": 1 },
                "input_data": { "type": "object", "nullable": true },
                "requires_human_approval": { "type": "boolean", "nullable": true, "example": true }
            }
        },
        "AiJobUpdate": {
            "type": "object",
            "required": ["job_status"],
            "properties": {
                "job_status": { "type": "string", "enum": ["pending", "queued", "running", "completed", "failed", "approved", "rejected"], "example": "completed" },
                "result_data": { "type": "object", "nullable": true },
                "confidence_score": { "type": "number", "nullable": true, "example": 0.92 }
            }
        },
        "VehicleAiAnalysisCreate": {
            "type": "object",
            "required": ["analysis_type"],
            "properties": {
                "analysis_type": { "type": "string", "example": "vehicle_risk_summary" },
                "requires_human_approval": { "type": "boolean", "nullable": true, "example": true },
                "note": { "type": "string", "nullable": true, "example": "Araç detay ekranından analiz isteği" }
            }
        },
        "GenericUpdate": {
            "type": "object",
            "additionalProperties": true
        },
        "InventoryProductCreate": {
            "type": "object",
            "required": ["product_name"],
            "properties": {
                "qr_code": { "type": "string", "nullable": true },
                "product_name": { "type": "string", "example": "Plastik Bardak" },
                "category": { "type": "string", "nullable": true, "example": "Sarf" },
                "sub_category": { "type": "string", "nullable": true, "example": "Mutfak" },
                "brand": { "type": "string", "nullable": true, "example": "Tuluklar" },
                "description": { "type": "string", "nullable": true },
                "main_unit": { "type": "string", "enum": ["adet", "koli", "paket", "kutu", "kg", "litre", "metre"], "example": "adet" },
                "package_unit": { "type": "string", "nullable": true, "example": "koli" },
                "package_multiplier": { "type": "number", "example": 1000 },
                "minimum_stock": { "type": "number", "example": 500 },
                "maximum_stock": { "type": "number", "nullable": true, "example": 10000 },
                "critical_stock": { "type": "number", "nullable": true, "example": 750 },
                "safety_stock": { "type": "number", "example": 1000 },
                "unit_cost": { "type": "number", "nullable": true, "example": 1.25 },
                "is_lot_tracked": { "type": "boolean", "example": true },
                "expiry_tracking": { "type": "boolean", "example": true }
            }
        },
        "InventoryProductUpdate": {
            "type": "object",
            "properties": {
                "qr_code": { "type": "string", "nullable": true },
                "product_name": { "type": "string", "nullable": true },
                "category": { "type": "string", "nullable": true },
                "sub_category": { "type": "string", "nullable": true },
                "brand": { "type": "string", "nullable": true },
                "description": { "type": "string", "nullable": true },
                "main_unit": { "type": "string", "nullable": true, "enum": ["adet", "koli", "paket", "kutu", "kg", "litre", "metre"] },
                "package_unit": { "type": "string", "nullable": true },
                "package_multiplier": { "type": "number", "nullable": true },
                "minimum_stock": { "type": "number", "nullable": true },
                "maximum_stock": { "type": "number", "nullable": true },
                "critical_stock": { "type": "number", "nullable": true },
                "safety_stock": { "type": "number", "nullable": true },
                "unit_cost": { "type": "number", "nullable": true },
                "is_lot_tracked": { "type": "boolean", "nullable": true },
                "expiry_tracking": { "type": "boolean", "nullable": true },
                "is_active": { "type": "boolean", "nullable": true }
            }
        },
        "InventoryMovementCreate": {
            "type": "object",
            "required": ["product_id", "movement_type", "quantity"],
            "properties": {
                "product_id": { "type": "integer", "format": "int64", "example": 1 },
                "lot_id": { "type": "integer", "format": "int64", "nullable": true },
                "movement_type": { "type": "string", "enum": ["warehouse_in", "warehouse_out", "branch_shipment", "branch_return", "count_adjustment", "scrap_out", "vehicle_usage", "manual_in", "manual_out"], "example": "warehouse_in" },
                "quantity": { "type": "number", "example": 5 },
                "unit": { "type": "string", "nullable": true, "example": "koli" },
                "unit_multiplier": { "type": "number", "nullable": true, "example": 1000 },
                "branch_id": { "type": "integer", "format": "int64", "nullable": true },
                "vehicle_id": { "type": "integer", "format": "int64", "nullable": true },
                "reference_table": { "type": "string", "nullable": true },
                "reference_id": { "type": "integer", "format": "int64", "nullable": true },
                "description": { "type": "string", "nullable": true }
            }
        },
        "CancelMovementRequest": {
            "type": "object",
            "required": ["reason"],
            "properties": {
                "reason": { "type": "string", "example": "Yanlis giris iptali" }
            }
        },
        "InventoryLotCreate": {
            "type": "object",
            "required": ["product_id", "lot_number"],
            "properties": {
                "product_id": { "type": "integer", "format": "int64", "example": 1 },
                "lot_number": { "type": "string", "example": "LOT-2026-001" },
                "production_date": { "type": "string", "format": "date", "nullable": true },
                "expiry_date": { "type": "string", "format": "date", "nullable": true },
                "supplier": { "type": "string", "nullable": true },
                "quantity": { "type": "number", "example": 5000 }
            }
        },
        "InventoryShipmentCreate": {
            "type": "object",
            "required": ["branch_id", "items"],
            "properties": {
                "branch_id": { "type": "integer", "format": "int64", "example": 1 },
                "sender_user_id": { "type": "integer", "format": "int64", "nullable": true },
                "note": { "type": "string", "nullable": true },
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["product_id", "quantity"],
                        "properties": {
                            "product_id": { "type": "integer", "format": "int64", "example": 1 },
                            "quantity": { "type": "number", "example": 150 },
                            "unit": { "type": "string", "nullable": true, "example": "adet" },
                            "unit_multiplier": { "type": "number", "nullable": true, "example": 1 }
                        }
                    }
                }
            }
        },
        "InventoryCountCreate": {
            "type": "object",
            "required": ["count_type", "items"],
            "properties": {
                "count_type": { "type": "string", "example": "partial" },
                "count_method": { "type": "string", "example": "manual" },
                "category": { "type": "string", "nullable": true },
                "note": { "type": "string", "nullable": true },
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["product_id", "counted_stock"],
                        "properties": {
                            "product_id": { "type": "integer", "format": "int64", "example": 1 },
                            "counted_stock": { "type": "number", "example": 4800 },
                            "reason": { "type": "string", "nullable": true }
                        }
                    }
                }
            }
        },
        "InventoryPurchaseRequestCreate": {
            "type": "object",
            "required": ["product_id", "requested_quantity"],
            "properties": {
                "product_id": { "type": "integer", "format": "int64", "example": 1 },
                "requested_quantity": { "type": "number", "example": 100 },
                "reason": { "type": "string", "nullable": true, "example": "Kritik stok" },
                "note": { "type": "string", "nullable": true }
            }
        },
        "InventoryPurchaseRequestUpdate": {
            "type": "object",
            "properties": {
                "requested_quantity": { "type": "number", "nullable": true },
                "approved_quantity": { "type": "number", "nullable": true },
                "request_status": { "type": "string", "enum": ["open", "approved", "ordered", "received", "cancelled"], "nullable": true },
                "note": { "type": "string", "nullable": true }
            }
        },
        "ApiError": {
            "type": "object",
            "properties": {
                "error": { "type": "string" }
            }
        }
    })
}

fn paths() -> Value {
    json!({
        "/health": { "get": public_op("Health", "Backend health check") },
        "/health/details": { "get": public_op("Health", "Backend detailed health check") },
        "/api/v1/auth/login": {
            "post": {
                "tags": ["Auth"],
                "summary": "Login",
                "requestBody": json_body("#/components/schemas/LoginRequest"),
                "responses": ok_schema("#/components/schemas/LoginResponse")
            }
        },
        "/api/v1/auth/refresh": { "post": secured_body_op("Auth", "Refresh token", "#/components/schemas/RefreshRequest") },
        "/api/v1/auth/logout": { "post": secured_op("Auth", "Logout") },
        "/api/v1/auth/change-password": { "post": secured_body_op("Auth", "Change password", "#/components/schemas/ChangePasswordRequest") },
        "/api/v1/auth/me": { "get": secured_op("Auth", "Current user") },

        "/api/v1/users": collection_ops("Users", "#/components/schemas/UserCreate"),
        "/api/v1/users/roles": { "get": secured_op("Users", "List roles") },
        "/api/v1/users/{id}/mobile-permissions": {
            "get": secured_op("Users", "List user mobile permissions"),
            "post": secured_body_op("Users", "Upsert user mobile permission", "#/components/schemas/MobilePermissionUpsert")
        },
        "/api/v1/users/{id}": item_ops_with_update("Users", "#/components/schemas/UserUpdate"),

        "/api/v1/organization/companies": collection_ops("Organization", "#/components/schemas/CompanyCreate"),
        "/api/v1/organization/companies/{id}": item_get_patch_ops("Organization"),
        "/api/v1/organization/departments": collection_ops("Organization", "#/components/schemas/DepartmentCreate"),
        "/api/v1/organization/departments/{id}": item_get_patch_ops("Organization"),

        "/api/v1/vehicles": collection_ops("Vehicles", "#/components/schemas/VehicleCreate"),
        "/api/v1/vehicles/{id}": item_ops_with_update("Vehicles", "#/components/schemas/VehicleUpdate"),
        "/api/v1/vehicles/{id}/profile": { "get": secured_op("Vehicles", "Vehicle profile/detail bundle") },
        "/api/v1/vehicles/{id}/sell": { "post": secured_body_op("Vehicles", "Sell vehicle", "#/components/schemas/SellVehicleRequest") },
        "/api/v1/vehicles/sold/{id}": { "get": secured_op("Vehicles", "Sold vehicle detail") },

        "/api/v1/assignments": collection_ops("Assignments", "#/components/schemas/AssignmentCreate"),
        "/api/v1/assignments/{id}/release": { "post": secured_body_op("Assignments", "Release assignment", "#/components/schemas/ReleaseAssignmentRequest") },
        "/api/v1/assignments/vehicles/{vehicle_id}": { "get": secured_op("Assignments", "Vehicle assignment history") },

        "/api/v1/dashboard": { "get": secured_op("Dashboard", "Front dashboard summary") },

        "/api/v1/tracking/km-logs": collection_ops("Tracking", "#/components/schemas/KmLogCreate"),
        "/api/v1/tracking/km-logs/{id}/verify": { "post": secured_body_op("Tracking", "Verify KM log", "#/components/schemas/VerifyKmLogRequest") },
        "/api/v1/tracking/vehicles/{vehicle_id}/km-logs": { "get": secured_op("Tracking", "Vehicle KM logs") },

        "/api/v1/maintenances": collection_ops("Maintenances", "#/components/schemas/MaintenanceCreate"),
        "/api/v1/maintenances/{id}": item_ops_with_update("Maintenances", "#/components/schemas/MaintenanceStatusUpdate"),

        "/api/v1/insurance-policies": collection_ops("Insurance", "#/components/schemas/PolicyCreate"),
        "/api/v1/insurance-policies/{id}": item_ops_with_update("Insurance", "#/components/schemas/RenewalStatusUpdate"),

        "/api/v1/expenses": collection_ops("Expenses", "#/components/schemas/ExpenseCreate"),
        "/api/v1/expenses/{id}": item_ops_with_update("Expenses", "#/components/schemas/PaymentStatusUpdate"),

        "/api/v1/damages": collection_ops("Damages", "#/components/schemas/DamageCreate"),
        "/api/v1/damages/{id}": item_ops_with_update("Damages", "#/components/schemas/DamageUpdate"),

        "/api/v1/tasks": collection_ops("Tasks", "#/components/schemas/TaskCreate"),
        "/api/v1/tasks/mine": { "get": secured_op("Tasks", "My tasks") },
        "/api/v1/tasks/{id}": item_get_patch_ops("Tasks"),

        "/api/v1/notifications": collection_ops("Notifications", "#/components/schemas/NotificationCreate"),
        "/api/v1/notifications/mine": { "get": secured_op("Notifications", "My notifications") },
        "/api/v1/notifications/provider-status": { "get": secured_op("Notifications", "Notification provider configuration status") },
        "/api/v1/notifications/whatsapp/webhook": { "get": public_op("Notifications", "Verify WhatsApp webhook"), "post": public_op("Notifications", "Receive WhatsApp webhook payload") },
        "/api/v1/notifications/{id}": item_get_patch_ops_with_update("Notifications", "#/components/schemas/NotificationDeliveryUpdate"),
        "/api/v1/notifications/{id}/dispatch": { "post": secured_op("Notifications", "Prepare WhatsApp dispatch payload") },
        "/api/v1/notifications/{id}/read": { "post": secured_op("Notifications", "Mark notification as read") },

        "/api/v1/operations/vehicle-inspections": collection_ops("Operations", "#/components/schemas/VehicleInspectionCreate"),
        "/api/v1/operations/vehicle-inspections/{id}": item_ops_with_update("Operations", "#/components/schemas/VehicleInspectionUpdate"),
        "/api/v1/operations/value-loss-claims": collection_ops("Operations", "#/components/schemas/ValueLossClaimCreate"),
        "/api/v1/operations/value-loss-claims/{id}": item_ops_with_update("Operations", "#/components/schemas/ValueLossClaimUpdate"),
        "/api/v1/operations/fuel-entries": collection_ops("Operations", "#/components/schemas/FuelEntryCreate"),
        "/api/v1/operations/fuel-entries/{id}": item_ops_with_update("Operations", "#/components/schemas/FuelEntryUpdate"),
        "/api/v1/operations/vehicle-washes": collection_ops("Operations", "#/components/schemas/VehicleWashCreate"),
        "/api/v1/operations/vehicle-washes/{id}": item_ops_with_update("Operations", "#/components/schemas/VehicleWashUpdate"),
        "/api/v1/operations/insurance-quotes": collection_ops("Operations", "#/components/schemas/InsuranceQuoteCreate"),
        "/api/v1/operations/insurance-quotes/{id}": item_ops_with_update("Operations", "#/components/schemas/InsuranceQuoteUpdate"),

        "/api/v1/files": { "get": secured_op("Files", "List files") },
        "/api/v1/files/upload": {
            "post": {
                "tags": ["Files"],
                "summary": "Upload file",
                "security": [{ "bearerAuth": [] }],
                "requestBody": {
                    "required": true,
                    "content": {
                        "multipart/form-data": {
                            "schema": {
                                "type": "object",
                                "required": ["module_name", "entity_id", "file_type", "file"],
                                "properties": {
                                    "module_name": {
                                        "type": "string",
                                        "enum": ["vehicles", "insurance_policies", "expenses", "maintenances", "damages", "sold_vehicles", "inventory_products", "support_tickets", "support_knowledge_base"],
                                        "example": "support_tickets"
                                    },
                                    "entity_id": { "type": "integer", "format": "int64", "example": 1 },
                                    "file_type": {
                                        "type": "string",
                                        "enum": ["registration", "delivery_form", "vehicle_photo", "km_photo", "policy_pdf", "offer", "invoice", "payment_receipt", "expense_document", "damage_report", "expert_report", "damage_photo", "sale_document", "transfer_document", "technical_document", "warranty_document", "product_photo", "usage_instruction", "support_screenshot", "support_video", "support_log", "support_document", "knowledge_attachment"],
                                        "example": "support_screenshot"
                                    },
                                    "file": { "type": "string", "format": "binary" }
                                }
                            }
                        }
                    }
                },
                "responses": ok_any()
            }
        },
        "/api/v1/files/{id}": item_get_delete_ops("Files"),

        "/api/v1/inventory/dashboard": { "get": secured_op("Inventory", "Inventory dashboard summary with recent movements and notifications") },
        "/api/v1/inventory/alerts": { "get": secured_op("Inventory", "Critical stock alerts") },
        "/api/v1/inventory/purchase-suggestions": { "get": secured_op("Inventory", "Smart purchase suggestions") },
        "/api/v1/inventory/products": collection_ops("Inventory", "#/components/schemas/InventoryProductCreate"),
        "/api/v1/inventory/products/{id}": item_ops_with_update("Inventory", "#/components/schemas/InventoryProductUpdate"),
        "/api/v1/inventory/lots": collection_ops("Inventory", "#/components/schemas/InventoryLotCreate"),
        "/api/v1/inventory/movements": collection_ops("Inventory", "#/components/schemas/InventoryMovementCreate"),
        "/api/v1/inventory/movements/{id}/cancel": { "post": secured_body_op("Inventory", "Cancel stock movement", "#/components/schemas/CancelMovementRequest") },
        "/api/v1/inventory/shipments": collection_ops("Inventory", "#/components/schemas/InventoryShipmentCreate"),
        "/api/v1/inventory/shipments/{id}": { "get": secured_op("Inventory", "Shipment detail with items") },
        "/api/v1/inventory/shipments/{id}/approve": { "post": secured_op("Inventory", "Approve shipment and reduce stock") },
        "/api/v1/inventory/counts": collection_ops("Inventory", "#/components/schemas/InventoryCountCreate"),
        "/api/v1/inventory/counts/{id}": { "get": secured_op("Inventory", "Count detail with items") },
        "/api/v1/inventory/counts/{id}/complete": { "post": secured_op("Inventory", "Complete count and apply adjustment") },
        "/api/v1/inventory/purchase-requests": collection_ops("Inventory", "#/components/schemas/InventoryPurchaseRequestCreate"),
        "/api/v1/inventory/purchase-requests/{id}": item_get_patch_ops_with_update("Inventory", "#/components/schemas/InventoryPurchaseRequestUpdate"),

        "/api/v1/reports/summary": { "get": secured_op("Reports", "Summary report") },
        "/api/v1/reports/management.xlsx": { "get": secured_op("Reports", "Management summary Excel XLSX") },
        "/api/v1/reports/vehicles.xlsx": { "get": secured_op("Reports", "Vehicles Excel XLSX") },
        "/api/v1/reports/maintenances.xlsx": { "get": secured_op("Reports", "Maintenances Excel XLSX") },
        "/api/v1/reports/insurance-policies.xlsx": { "get": secured_op("Reports", "Insurance policies Excel XLSX") },
        "/api/v1/reports/expenses.xlsx": { "get": secured_op("Reports", "Expenses Excel XLSX") },
        "/api/v1/reports/damages.xlsx": { "get": secured_op("Reports", "Damages Excel XLSX") },
        "/api/v1/reports/vehicle-inspections.xlsx": { "get": secured_op("Reports", "Vehicle inspections Excel XLSX") },
        "/api/v1/reports/value-loss-claims.xlsx": { "get": secured_op("Reports", "Value loss claims Excel XLSX") },
        "/api/v1/reports/fuel-entries.xlsx": { "get": secured_op("Reports", "Fuel and KM Excel XLSX") },
        "/api/v1/reports/vehicle-washes.xlsx": { "get": secured_op("Reports", "Vehicle washes Excel XLSX") },
        "/api/v1/reports/insurance-quotes.xlsx": { "get": secured_op("Reports", "Insurance quotes Excel XLSX") },
        "/api/v1/reports/inventory-products.xlsx": { "get": secured_op("Reports", "Inventory products Excel XLSX") },
        "/api/v1/reports/inventory-movements.xlsx": { "get": secured_op("Reports", "Inventory movements Excel XLSX") },
        "/api/v1/reports/inventory-critical.xlsx": { "get": secured_op("Reports", "Inventory critical stock Excel XLSX") },
        "/api/v1/reports/support-tickets.xlsx": { "get": secured_op("Reports", "IT support tickets Excel XLSX") },
        "/api/v1/reports/support-knowledge-base.xlsx": { "get": secured_op("Reports", "IT support knowledge base Excel XLSX") },

        "/api/v1/search": { "get": secured_op("Search", "Global search") },

        "/api/v1/imports/jobs": collection_ops("Imports", "#/components/schemas/GenericUpdate"),
        "/api/v1/imports/jobs/{id}": item_get_patch_ops("Imports"),
        "/api/v1/imports/jobs/{id}/errors": collection_ops("Imports", "#/components/schemas/GenericUpdate"),
        "/api/v1/imports/jobs/{id}/rows": collection_ops("Imports", "#/components/schemas/GenericUpdate"),
        "/api/v1/imports/jobs/{id}/validate": { "post": secured_op("Imports", "Validate import job") },

        "/api/v1/system-logs": collection_ops("System Logs", "#/components/schemas/GenericUpdate"),
        "/api/v1/system-logs/{id}": { "get": secured_op("System Logs", "Get system log") },

        "/api/v1/settings": collection_ops("Settings", "#/components/schemas/GenericUpdate"),
        "/api/v1/settings/{key}": item_get_patch_ops("Settings"),

        "/api/v1/support/summary": { "get": secured_op("IT Support", "Support ticket KPI summary") },
        "/api/v1/support/knowledge-base": collection_ops("IT Support", "#/components/schemas/GenericUpdate"),
        "/api/v1/support/knowledge-base/{id}": item_get_patch_ops("IT Support"),
        "/api/v1/support/tickets": collection_ops("IT Support", "#/components/schemas/GenericUpdate"),
        "/api/v1/support/tickets/{id}": item_get_patch_ops("IT Support"),
        "/api/v1/support/tickets/{id}/reply": { "post": secured_body_op("IT Support", "Prepare WhatsApp reply for support ticket", "#/components/schemas/GenericUpdate") },
        "/api/v1/support/tickets/{id}/events": { "get": secured_op("IT Support", "Support ticket event history") },

        "/api/v1/ai/capabilities": { "get": secured_op("AI", "List AI use cases and integration points") },
        "/api/v1/ai/provider": { "get": secured_op("AI", "AI provider configuration status") },
        "/api/v1/ai/prompt-templates": { "get": secured_op("AI", "AI prompt template contracts") },
        "/api/v1/ai/recommendations": { "get": secured_op("AI", "List AI recommendations from current ERP state") },
        "/api/v1/ai/jobs": collection_ops("AI", "#/components/schemas/AiJobCreate"),
        "/api/v1/ai/jobs/{id}": item_get_patch_ops_with_update("AI", "#/components/schemas/AiJobUpdate"),
        "/api/v1/ai/jobs/{id}/approve": { "post": secured_op("AI", "Approve AI job") },
        "/api/v1/ai/jobs/{id}/task-draft": { "get": secured_op("AI", "Create task draft from AI result") },
        "/api/v1/ai/jobs/{id}/notification-draft": { "get": secured_op("AI", "Create notification draft from AI result") },
        "/api/v1/ai/vehicles/{id}/context": { "get": secured_op("AI", "Vehicle AI context package") },
        "/api/v1/ai/vehicles/{id}/analyze": { "post": secured_body_op("AI", "Create vehicle AI analysis job", "#/components/schemas/VehicleAiAnalysisCreate") },
        "/api/v1/ai/support/tickets/{id}/context": { "get": secured_op("AI", "Support ticket AI context package") },
        "/api/v1/ai/support/tickets/{id}/analyze": { "post": secured_body_op("AI", "Create support ticket AI analysis job", "#/components/schemas/GenericUpdate") },

        "/api/v1/logs/audit": { "get": secured_op("Audit", "Audit logs with filters") }
    })
}

fn public_op(tag: &str, summary: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "responses": ok_any()
    })
}

fn secured_op(tag: &str, summary: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "security": [{ "bearerAuth": [] }],
        "responses": ok_any()
    })
}

fn secured_body_op(tag: &str, summary: &str, schema_ref: &str) -> Value {
    json!({
        "tags": [tag],
        "summary": summary,
        "security": [{ "bearerAuth": [] }],
        "requestBody": json_body(schema_ref),
        "responses": ok_any()
    })
}

fn collection_ops(tag: &str, create_schema_ref: &str) -> Value {
    json!({
        "get": secured_op(tag, "List"),
        "post": secured_body_op(tag, "Create", create_schema_ref)
    })
}

fn item_ops_with_update(tag: &str, update_schema_ref: &str) -> Value {
    json!({
        "get": secured_op(tag, "Get by id"),
        "patch": secured_body_op(tag, "Update", update_schema_ref),
        "delete": secured_op(tag, "Delete / archive")
    })
}

fn item_get_patch_ops(tag: &str) -> Value {
    json!({
        "get": secured_op(tag, "Get by id"),
        "patch": secured_body_op(tag, "Update", "#/components/schemas/GenericUpdate")
    })
}

fn item_get_patch_ops_with_update(tag: &str, update_schema_ref: &str) -> Value {
    json!({
        "get": secured_op(tag, "Get by id"),
        "patch": secured_body_op(tag, "Update", update_schema_ref)
    })
}

fn item_get_delete_ops(tag: &str) -> Value {
    json!({
        "get": secured_op(tag, "Get by id"),
        "delete": secured_op(tag, "Delete / archive")
    })
}

fn json_body(schema_ref: &str) -> Value {
    json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": { "$ref": schema_ref }
            }
        }
    })
}

fn ok_schema(schema_ref: &str) -> Value {
    json!({
        "200": {
            "description": "OK",
            "content": {
                "application/json": {
                    "schema": { "$ref": schema_ref }
                }
            }
        },
        "400": { "description": "Bad request" },
        "401": { "description": "Unauthorized" },
        "403": { "description": "Forbidden" },
        "404": { "description": "Not found" }
    })
}

fn ok_any() -> Value {
    json!({
        "200": { "description": "OK" },
        "201": { "description": "Created" },
        "400": { "description": "Bad request" },
        "401": { "description": "Unauthorized" },
        "403": { "description": "Forbidden" },
        "404": { "description": "Not found" }
    })
}

const SWAGGER_HTML: &str = r##"<!doctype html>
<html lang="tr">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Tuluklar ERP API Docs</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    body { margin: 0; background: #f7f7f7; }
    .topbar { display: none; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.ui = SwaggerUIBundle({
      url: "/openapi.json",
      dom_id: "#swagger-ui",
      deepLinking: true,
      persistAuthorization: true,
      displayRequestDuration: true
    });
  </script>
</body>
</html>"##;
