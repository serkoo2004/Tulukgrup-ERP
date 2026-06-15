# API Endpoints

Base path: `/api/v1`

Swagger UI:

- `GET /docs`
- `GET /openapi.json`

Swagger'da once `POST /api/v1/auth/login` ile token alinir. Sonra sag ustteki `Authorize` alanina `Bearer <access_token>` girilerek korumali endpointler test edilir.

## Health

- `GET /health`
- `GET /health/details`

## Auth

- `POST /auth/login`
- `POST /auth/logout`
- `POST /auth/refresh`
- `POST /auth/change-password`
- `GET /auth/me`

## Users

- `GET /users`
- `POST /users`
- `GET /users/roles`
- `GET /users/:id/mobile-permissions`
- `POST /users/:id/mobile-permissions`
- `GET /users/:id`
- `PATCH /users/:id`
- `DELETE /users/:id`

Mobil yetki anahtarları: `dashboard`, `inventory_view`, `inventory_order`, `vehicle_view`, `vehicle_fault`, `km_log`, `support_ticket`, `media_upload`.

## Organization

- `GET /organization/companies`
- `POST /organization/companies`
- `GET /organization/companies/:id`
- `PATCH /organization/companies/:id`
- `GET /organization/departments`
- `POST /organization/departments`
- `GET /organization/departments/:id`
- `PATCH /organization/departments/:id`

## Vehicles

- `GET /vehicles`
- `POST /vehicles`
- `GET /vehicles/:id`
- `GET /vehicles/:id/profile`
- `PATCH /vehicles/:id`
- `DELETE /vehicles/:id`
- `POST /vehicles/:id/sell`
- `GET /vehicles/sold/:id`

## Dashboard

- `GET /dashboard`

## Inventory

- `GET /inventory/dashboard` returns summary metrics, `recent_movements`, and `recent_notifications`
- `GET /inventory/alerts`
- `GET /inventory/purchase-suggestions`
- `GET /inventory/products?q=&category=&low_stock=&include_inactive=&limit=`
- `POST /inventory/products`
- `GET /inventory/products/:id`
- `PATCH /inventory/products/:id`
- `DELETE /inventory/products/:id`
- `GET /inventory/lots`
- `POST /inventory/lots`
- `GET /inventory/movements?product_id=&movement_type=&branch_id=&vehicle_id=&date_from=&date_to=&limit=`
- `POST /inventory/movements`
- `POST /inventory/movements/:id/cancel`
- `GET /inventory/shipments`
- `POST /inventory/shipments`
- `GET /inventory/shipments/:id`
- `POST /inventory/shipments/:id/approve`
- `GET /inventory/counts`
- `POST /inventory/counts`
- `GET /inventory/counts/:id`
- `POST /inventory/counts/:id/complete`
- `GET /inventory/purchase-requests`
- `POST /inventory/purchase-requests`
- `GET /inventory/purchase-requests/:id`
- `PATCH /inventory/purchase-requests/:id`

## Assignments

- `GET /assignments`
- `POST /assignments`
- `POST /assignments/:id/release`
- `GET /assignments/vehicles/:vehicle_id`

## Tracking

- `GET /tracking/km-logs`
- `POST /tracking/km-logs`
- `POST /tracking/km-logs/:id/verify`
- `GET /tracking/vehicles/:vehicle_id/km-logs`

`entry_type=ocr` ile KM kaydı açılırsa `image_path` zorunludur. Backend bu kayıt için `ocr_verification` AI job oluşturur; sonuç insan onayı olmadan KM doğrulamasını otomatik değiştirmez.

## Maintenances

- `GET /maintenances`
- `POST /maintenances`
- `GET /maintenances/:id`
- `PATCH /maintenances/:id`
- `DELETE /maintenances/:id`

## Insurance Policies

- `GET /insurance-policies`
- `POST /insurance-policies`
- `GET /insurance-policies/:id`
- `PATCH /insurance-policies/:id`
- `DELETE /insurance-policies/:id`

## Expenses

- `GET /expenses`
- `POST /expenses`
- `GET /expenses/:id` returns `expense` and `file_summary`
- `PATCH /expenses/:id`
- `DELETE /expenses/:id`

## Damages

- `GET /damages`
- `POST /damages`
- `GET /damages/:id`
- `PATCH /damages/:id`
- `DELETE /damages/:id`

## Tasks

- `GET /tasks`
- `POST /tasks`
- `GET /tasks/mine`
- `GET /tasks/:id`
- `PATCH /tasks/:id`

## Notifications

- `GET /notifications`
- `POST /notifications`
- `GET /notifications/mine`
- `GET /notifications/provider-status`
- `GET /notifications/whatsapp/webhook`
- `POST /notifications/whatsapp/webhook`
- `GET /notifications/:id`
- `PATCH /notifications/:id`
- `POST /notifications/:id/dispatch`
- `POST /notifications/:id/read`

## IT Support

- `GET /support/summary`
- `GET /support/knowledge-base`
- `POST /support/knowledge-base`
- `GET /support/knowledge-base/:id`
- `PATCH /support/knowledge-base/:id`
- `GET /support/tickets`
- `POST /support/tickets`
- `GET /support/tickets/:id`
- `PATCH /support/tickets/:id`
- `POST /support/tickets/:id/reply`
- `GET /support/tickets/:id/events`

Destek talebi listesi `ticket_status`, `priority`, `source_channel`, `assigned_user_id`, `reporter_phone`, `q`, `overdue_only`, `limit` filtrelerini destekler. Her talep SLA ilk cevap hedefi, SLA cozum hedefi, SLA durumu, ilk cevap zamani, eskalasyon seviyesi ve memnuniyet alanlariyla doner.

## Operations

- `GET /operations/vehicle-inspections`
- `POST /operations/vehicle-inspections`
- `GET /operations/vehicle-inspections/:id`
- `PATCH /operations/vehicle-inspections/:id`
- `DELETE /operations/vehicle-inspections/:id`
- `GET /operations/value-loss-claims`
- `POST /operations/value-loss-claims`
- `GET /operations/value-loss-claims/:id`
- `PATCH /operations/value-loss-claims/:id`
- `DELETE /operations/value-loss-claims/:id`
- `GET /operations/fuel-entries`
- `POST /operations/fuel-entries`
- `GET /operations/fuel-entries/:id`
- `PATCH /operations/fuel-entries/:id`
- `DELETE /operations/fuel-entries/:id`
- `GET /operations/vehicle-washes`
- `POST /operations/vehicle-washes`
- `GET /operations/vehicle-washes/:id`
- `PATCH /operations/vehicle-washes/:id`
- `DELETE /operations/vehicle-washes/:id`
- `GET /operations/insurance-quotes`
- `POST /operations/insurance-quotes`
- `GET /operations/insurance-quotes/:id`
- `PATCH /operations/insurance-quotes/:id`
- `DELETE /operations/insurance-quotes/:id`

## Files

- `GET /files`
- `POST /files/upload`
- `GET /files/:id`
- `DELETE /files/:id`

Araç KM fotoğrafı için `module_name=vehicles`, `file_type=km_photo`, `entity_id=<vehicle_id>` kullanılır. Mobil kullanıcı tarafında `km_log` create yetkisi yeterlidir.
KM fotoğrafı ile açılan `ocr` KM kaydı `ocr_verification` AI job oluşturur. `support_tickets` dosyalarındaki ekran görüntüsü/video/log/doküman ekleri `support_media_analysis`, `vehicles.vehicle_photo` ve `damages.damage_photo` ekleri `vehicle_media_inspection` AI job oluşturur. Bu işler local veya dış vision model worker tarafından işlenebilir ve sonuçlar insan onayı olmadan kritik kayıt değiştirmez.

Supported expense files use `module_name=expenses` with `file_type=invoice`, `payment_receipt`, or `expense_document`.
Supported damage files use `module_name=damages` with `file_type=damage_report`, `expert_report`, or `damage_photo`.
Supported inventory product files use `module_name=inventory_products` with `file_type=invoice`, `technical_document`, `warranty_document`, `product_photo`, or `usage_instruction`.
Supported IT support ticket files use `module_name=support_tickets` with `file_type=support_screenshot`, `support_video`, `support_log`, or `support_document`.
Supported IT knowledge base files use `module_name=support_knowledge_base` with `file_type=knowledge_attachment`, `technical_document`, or `usage_instruction`.
Upload rejects unknown `entity_id` values and module/file type combinations that are not allowed.

## Reports

- `GET /reports/summary`
- `GET /reports/management.xlsx`
- `GET /reports/vehicles.xlsx`
- `GET /reports/maintenances.xlsx`
- `GET /reports/insurance-policies.xlsx`
- `GET /reports/expenses.xlsx`
- `GET /reports/damages.xlsx`
- `GET /reports/vehicle-inspections.xlsx`
- `GET /reports/value-loss-claims.xlsx`
- `GET /reports/fuel-entries.xlsx`
- `GET /reports/vehicle-washes.xlsx`
- `GET /reports/insurance-quotes.xlsx`
- `GET /reports/inventory-products.xlsx`
- `GET /reports/inventory-movements.xlsx`
- `GET /reports/inventory-critical.xlsx`
- `GET /reports/support-tickets.xlsx`
- `GET /reports/support-knowledge-base.xlsx`

Rapor dosya ciktilari Excel `.xlsx` formatindadir.
Excel raporlari patron/yonetim kullanimi icin baslik bandi, olusturulma zamani, KPI kutulari, filtreli tablo, donmus baslik satiri, kolon genislikleri, durum renkleri ve yatay sayfa ayariyla uretilir. `management.xlsx` yonetim ozeti; arac, gorev, police, bakim, gider ve hasar takip metriklerini tek dosyada toplar.
Bakim, police, gider, hasar, operasyon, stok hareket ve IT destek XLSX raporlari uygun olduklari alanlarda `vehicle_id`, `status`, `start_date`, `end_date` query parametreleriyle filtrelenebilir.

## Search

- `GET /search?q=<aranacak metin>`

## Imports

- `GET /imports/jobs`
- `POST /imports/jobs`
- `GET /imports/jobs/:id`
- `PATCH /imports/jobs/:id`
- `GET /imports/jobs/:id/errors`
- `POST /imports/jobs/:id/errors`
- `GET /imports/jobs/:id/rows`
- `POST /imports/jobs/:id/rows`
- `POST /imports/jobs/:id/validate`

## System Logs

- `GET /system-logs`
- `POST /system-logs`
- `GET /system-logs/:id`

## Settings

- `GET /settings`
- `POST /settings`
- `GET /settings/:key`
- `PATCH /settings/:key`

## AI

- `GET /ai/capabilities`
- `GET /ai/provider`
- `GET /ai/prompt-templates`
- `GET /ai/recommendations`
- `GET /ai/jobs`
- `POST /ai/jobs`
- `GET /ai/jobs/:id`
- `PATCH /ai/jobs/:id`
- `POST /ai/jobs/:id/approve`
- `GET /ai/jobs/:id/task-draft`
- `GET /ai/jobs/:id/notification-draft`
- `GET /ai/vehicles/:id/context`
- `POST /ai/vehicles/:id/analyze`
- `GET /ai/support/tickets/:id/context`
- `POST /ai/support/tickets/:id/analyze`

## Audit

- `GET /logs/audit`
