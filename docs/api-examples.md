# API Examples

Base path: `/api/v1`

All protected requests use:

```http
Authorization: Bearer <access_token>
```

## Login

```http
POST /api/v1/auth/login
Content-Type: application/json
```

```json
{
  "email": "admin@tuluklar.local",
  "password": "Admin12345!"
}
```

## Logout

```http
POST /api/v1/auth/logout
Content-Type: application/json
```

```json
{
  "refresh_token": "<refresh_token>"
}
```

## Change Password

```http
POST /api/v1/auth/change-password
Content-Type: application/json
```

```json
{
  "current_password": "Admin12345!",
  "new_password": "NewAdmin12345!"
}
```

## Create Company

```http
POST /api/v1/organization/companies
Content-Type: application/json
```

```json
{
  "name": "Tuluklar Group",
  "tax_number": "1234567890"
}
```

## Create Department

```http
POST /api/v1/organization/departments
Content-Type: application/json
```

```json
{
  "company_id": 1,
  "name": "Operasyon"
}
```

## Create User

```http
POST /api/v1/users
Content-Type: application/json
```

```json
{
  "email": "operasyon@tuluklar.local",
  "full_name": "Operasyon Sorumlusu",
  "password": "ChangeMe123!",
  "role": "operation",
  "company_id": 1,
  "department_id": 1
}
```

## Update User

```http
PATCH /api/v1/users/2
Content-Type: application/json
```

```json
{
  "full_name": "Operasyon Yetkilisi",
  "role": "operation",
  "phone": "+905551112233",
  "is_active": true
}
```

## Deactivate User

```http
DELETE /api/v1/users/2
```

## Assign Mobile Permission

```http
POST /api/v1/users/2/mobile-permissions
Content-Type: application/json
```

```json
{
  "company_id": 1,
  "department_id": 1,
  "permission_key": "inventory_order",
  "can_view": true,
  "can_create": true,
  "can_update": false,
  "can_approve": false
}
```

`company_id` ve `department_id` boş/null gönderilirse yetki global çalışır.

## Create Vehicle

```http
POST /api/v1/vehicles
Content-Type: application/json
```

```json
{
  "plate": "34 ABC 123",
  "brand": "Ford",
  "model": "Transit",
  "model_year": 2024,
  "vehicle_type": "panelvan",
  "fuel_type": "dizel",
  "company_id": 1,
  "department_id": 1,
  "user_id": 2
}
```

## Add KM Log

```http
POST /api/v1/tracking/km-logs
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "km": 42850,
  "entry_type": "manual",
  "device_info": "desktop"
}
```

## Add KM Log With Photo OCR Queue

Önce fotoğraf yüklenir:

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Form alanları:

```text
module_name=vehicles
entity_id=1
file_type=km_photo
note=Mobil KM fotoğrafı
file=<odometer-photo.jpg>
```

Sonra KM kaydı `ocr` tipiyle açılır:

```http
POST /api/v1/tracking/km-logs
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "km": 42850,
  "entry_type": "ocr",
  "image_path": "vehicles/1/km_photo/example.jpg",
  "ocr_result": {
    "status": "pending",
    "source": "mobile_km_photo",
    "submitted_km": 42850
  },
  "device_info": "mobile:km-photo"
}
```

Bu kayıt açıldığında backend `ocr_verification` AI job oluşturur. Yerel AI worker bağlandığında fotoğrafı okuyup job sonucunu yazar; doğrulama yine insan onaylıdır.

## Assign Vehicle

```http
POST /api/v1/assignments
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "user_id": 2,
  "note": "Operasyon atamasi"
}
```

## Release Vehicle Assignment

```http
POST /api/v1/assignments/1/release
Content-Type: application/json
```

```json
{
  "note": "Arac kullanicidan teslim alindi"
}
```

## Update Task

```http
PATCH /api/v1/tasks/1
Content-Type: application/json
```

```json
{
  "task_status": "in_progress",
  "assigned_user_id": 2,
  "priority": "high",
  "due_date": "2026-05-25",
  "description": "Supheli KM kaydi incelenecek"
}
```

## Mark Notification Read

```http
POST /api/v1/notifications/1/read
```

## Notification Provider Status

```http
GET /api/v1/notifications/provider-status
```

Returns WhatsApp readiness without exposing the access token.

## Create WhatsApp Notification

```http
POST /api/v1/notifications
Content-Type: application/json
```

```json
{
  "notification_type": "inventory_stock_alert",
  "message": "Kritik stok: Plastik Bardak mevcut 4, minimum 5.",
  "sent_via": "whatsapp",
  "recipient_phone": "+905551112233"
}
```

## Dispatch WhatsApp Notification

```http
POST /api/v1/notifications/1/dispatch
```

If WhatsApp env values are not configured, the notification is stored as `failed` with `delivery_error`.

## Create Maintenance

```http
POST /api/v1/maintenances
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "last_maintenance_km": 30000,
  "next_maintenance_km": 45000,
  "maintenance_date": "2026-05-18",
  "service_company": "Yetkili Servis",
  "maintenance_type": "periyodik",
  "total_cost": "12500.00",
  "maintenance_status": "planned"
}
```

## Create Insurance Policy

```http
POST /api/v1/insurance-policies
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "policy_type": "trafik",
  "policy_number": "TRF-2026-0001",
  "insurance_company": "Sigorta A.S.",
  "agency_name": "Merkez Acente",
  "start_date": "2026-05-18",
  "end_date": "2027-05-18",
  "amount": "8500.00",
  "currency": "TRY",
  "renewal_status": "active"
}
```

## Create Expense

```http
POST /api/v1/expenses
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "expense_type": "fuel",
  "amount": "3200.00",
  "invoice_number": "INV-2026-0001",
  "payment_status": "pending",
  "expense_date": "2026-05-18",
  "note": "Mayis yakit gideri"
}
```

## Get Expense Detail

```http
GET /api/v1/expenses/1
```

Response includes `expense` and `file_summary`. Invoice/dekont documents are uploaded with `module_name=expenses`.

## Upload Expense Invoice

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `expenses`
- `entity_id`: `1`
- `file_type`: `invoice`
- `note`: optional
- `file`: binary file

## Upload Expense Payment Receipt

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `expenses`
- `entity_id`: `1`
- `file_type`: `payment_receipt`
- `note`: optional
- `file`: binary file

## Upload IT Support Attachment

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `support_tickets`
- `entity_id`: `1`
- `file_type`: `support_screenshot`, `support_video`, `support_log`, or `support_document`
- `note`: optional
- `file`: binary file

## Upload IT Knowledge Base Attachment

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `support_knowledge_base`
- `entity_id`: `1`
- `file_type`: `knowledge_attachment`, `technical_document`, or `usage_instruction`
- `note`: optional
- `file`: binary file

## Create Damage

```http
POST /api/v1/damages
Content-Type: application/json
```

```json
{
  "vehicle_id": 1,
  "damage_date": "2026-05-18",
  "damage_type": "kaza",
  "description": "On tampon hasari",
  "estimated_cost": "18000.00",
  "insurance_claim_no": "HSR-2026-0001",
  "damage_status": "open"
}
```

## Upload Damage Photo

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `damages`
- `entity_id`: `1`
- `file_type`: `damage_photo`
- `note`: optional
- `file`: binary file

## Export Expenses Excel

```http
GET /api/v1/reports/expenses.xlsx
```

## Export Management Excel

```http
GET /api/v1/reports/management.xlsx
```

## Summary Report

```http
GET /api/v1/reports/summary
```

Returns vehicle status counts, open task counts, upcoming policy counts, maintenance counts, current month expense totals, and open damage totals.

## Export Damages Excel

```http
GET /api/v1/reports/damages.xlsx
```

## Export IT Support Tickets Excel

```http
GET /api/v1/reports/support-tickets.xlsx
```

## Reply IT Support Ticket Over WhatsApp

```http
POST /api/v1/support/tickets/1/reply
Content-Type: application/json
```

```json
{
  "message": "Merhaba, destek talebiniz üzerinde işlem başlatıldı. Kontrol sonrası sizi bilgilendireceğiz.",
  "next_status": "waiting_user",
  "event_note": "WhatsApp yanıtı hazırlandı"
}
```

The endpoint creates a WhatsApp notification record, prepares provider payload, updates first response/SLA state, and writes a `whatsapp_reply` ticket event.

## Create Import Job

```http
POST /api/v1/imports/jobs
Content-Type: application/json
```

```json
{
  "target_module": "vehicles",
  "column_mapping": {
    "plate": "plate",
    "brand": "brand",
    "model": "model"
  },
  "note": "Arac Excel import denemesi"
}
```

## Stage Import Rows

```http
POST /api/v1/imports/jobs/1/rows
Content-Type: application/json
```

```json
{
  "rows": [
    {
      "plate": "34 ABC 123",
      "brand": "Ford",
      "model": "Transit",
      "model_year": 2024
    }
  ]
}
```

## Validate Import Job

```http
POST /api/v1/imports/jobs/1/validate
```

Validation writes invalid rows to `/api/v1/imports/jobs/1/errors` and marks clean rows in `/api/v1/imports/jobs/1/rows`.

## Upsert Setting

```http
POST /api/v1/settings
Content-Type: application/json
```

```json
{
  "setting_key": "notifications.whatsapp.enabled",
  "setting_value": false,
  "description": "WhatsApp provider aktiflik bayragi",
  "is_active": true
}
```

## Sell Vehicle

```http
POST /api/v1/vehicles/1/sell
Content-Type: application/json
```

```json
{
  "sold_date": "2026-05-18",
  "sold_reason": "satis",
  "sold_price": "750000.00",
  "buyer_info": "Alici firma bilgisi",
  "company_exit_reason": "satis",
  "note": "Devir evraklari dosya modulunde tutulacak"
}
```

## Upload File

```http
POST /api/v1/files/upload
Content-Type: multipart/form-data
```

Fields:

- `module_name`: `vehicles`
- `entity_id`: `1`
- `file_type`: `registration`
- `note`: optional
- `file`: binary file
