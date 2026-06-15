$ErrorActionPreference = "Stop"

$baseUrl = $env:BACKEND_BASE_URL
if (-not $baseUrl) {
    $baseUrl = "http://127.0.0.1:8080"
}

$email = $env:BOOTSTRAP_ADMIN_EMAIL
if (-not $email) {
    $email = "admin@tuluklar.local"
}

$password = $env:BOOTSTRAP_ADMIN_PASSWORD
if (-not $password) {
    $password = "Admin12345!"
}

Write-Host "Smoke test backend: $baseUrl"

$health = Invoke-RestMethod -Uri "$baseUrl/health"
if ($health.status -ne "ok") {
    throw "Health check failed"
}
Write-Host "[OK] health"

$loginBody = @{
    email = $email
    password = $password
} | ConvertTo-Json

$login = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/auth/login" `
    -ContentType "application/json" `
    -Body $loginBody

if (-not $login.access_token) {
    throw "Login did not return access token"
}
Write-Host "[OK] login"

$headers = @{ Authorization = "Bearer $($login.access_token)" }
$stamp = Get-Date -Format "yyyyMMddHHmmss"

$company = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/organization/companies" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        name = "Smoke Company $stamp"
        tax_number = "SMK$stamp"
    } | ConvertTo-Json)
Write-Host "[OK] company id=$($company.id)"

$department = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/organization/departments" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        company_id = $company.id
        name = "Operations $stamp"
    } | ConvertTo-Json)
Write-Host "[OK] department id=$($department.id)"

$vehicle = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/vehicles" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        plate = "34SMK$($stamp.Substring(8,6))"
        brand = "Ford"
        model = "Transit"
        model_year = 2024
        vehicle_type = "van"
        fuel_type = "diesel"
        transmission = "manual"
        chassis_no = "SMK-CHASSIS-$stamp"
        engine_no = "SMK-ENGINE-$stamp"
        warranty_status = "active"
        warranty_end = "2027-05-24"
        has_hgs = $true
        has_mobiliz = $true
        has_kopilot = $false
        has_k2 = $false
        tasitmatik_company = "Smoke Fuel"
        spare_key_location = "Smoke Key Cabinet"
        company_id = $company.id
        department_id = $department.id
    } | ConvertTo-Json)
Write-Host "[OK] vehicle id=$($vehicle.id)"

$kmLog = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/tracking/km-logs" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        km = 1200
        entry_type = "manual"
        device_info = "smoke-backend"
    } | ConvertTo-Json)
Write-Host "[OK] km log id=$($kmLog.id)"

$task = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/tasks" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        task_type = "vehicle_check"
        related_vehicle_id = $vehicle.id
        priority = "medium"
        description = "Smoke test task"
    } | ConvertTo-Json)
Write-Host "[OK] task id=$($task.id)"

$maintenance = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/maintenances" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        last_maintenance_km = 1000
        next_maintenance_km = 11000
        maintenance_date = "2026-05-24"
        service_company = "Smoke Service"
        maintenance_type = "periodic"
        description = "Smoke maintenance"
        total_cost = 1000
        maintenance_status = "planned"
    } | ConvertTo-Json)
Write-Host "[OK] maintenance id=$($maintenance.id)"

$policy = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/insurance-policies" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        policy_type = "trafik"
        policy_number = "SMK-POL-$stamp"
        insurance_company = "Smoke Insurance"
        agency_name = "Smoke Agency"
        start_date = "2026-01-01"
        end_date = "2026-12-31"
        amount = 1500
        previous_amount = 1200
        currency = "TRY"
        renewal_status = "active"
    } | ConvertTo-Json)
Write-Host "[OK] policy id=$($policy.id)"

$expense = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/expenses" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        expense_type = "fuel"
        amount = 500
        invoice_number = "SMK-INV-$stamp"
        payment_status = "pending"
        expense_date = "2026-05-24"
        note = "Smoke expense"
    } | ConvertTo-Json)
Write-Host "[OK] expense id=$($expense.id)"

$damage = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/damages" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        damage_date = "2026-05-24"
        damage_type = "test"
        description = "Smoke damage"
        estimated_cost = 750
        damage_status = "open"
    } | ConvertTo-Json)
Write-Host "[OK] damage id=$($damage.id)"

$inspection = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/operations/vehicle-inspections" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        branch = "Smoke Branch"
        inspection_date = "2026-05-24"
        inspector_user_id = 1
        exterior_ok = $true
        interior_ok = $true
        equipment_ok = $true
        documents_ok = $true
        damage_note = "Smoke inspection"
        action_note = "No action"
        fee = 100
        inspection_status = "open"
    } | ConvertTo-Json)
Write-Host "[OK] vehicle inspection id=$($inspection.id)"

$valueLoss = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/operations/value-loss-claims" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        accident_date = "2026-05-24"
        vehicle_purchase_date = "2026-01-01"
        tramer_amount = 1000
        deprivation_days = 5
        requested_amount = 2000
        claim_status = "open"
        note = "Smoke value loss"
    } | ConvertTo-Json)
Write-Host "[OK] value loss claim id=$($valueLoss.id)"

$fuel = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/operations/fuel-entries" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        period_year = 2026
        period_month = 5
        fuel_limit = 5000
        paid_amount = 1250
        remaining_limit = 3750
        distance_km = 800
        current_km = 2000
        liter_amount = 35
        note = "Smoke fuel"
    } | ConvertTo-Json)
Write-Host "[OK] fuel entry id=$($fuel.id)"

$wash = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/operations/vehicle-washes" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        branch = "Smoke Branch"
        wash_company = "Smoke Wash"
        wash_date = "2026-05-24"
        amount = 250
        user_id = 1
        note = "Smoke wash"
    } | ConvertTo-Json)
Write-Host "[OK] vehicle wash id=$($wash.id)"

$quote = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/operations/insurance-quotes" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        vehicle_id = $vehicle.id
        quote_type = "kasko"
        insurance_company = "Smoke Insurance"
        agency_name = "Smoke Agency"
        gross_premium = 4500
        installment_count = 6
        quote_status = "pending"
        valid_until = "2026-06-24"
        note = "Smoke quote"
    } | ConvertTo-Json)
Write-Host "[OK] insurance quote id=$($quote.id)"

$summary = Invoke-RestMethod -Uri "$baseUrl/api/v1/reports/summary" -Headers $headers
if (-not $summary.vehicles) {
    throw "Summary report did not return vehicle section"
}
Write-Host "[OK] reports summary"

$dashboard = Invoke-RestMethod -Uri "$baseUrl/api/v1/dashboard" -Headers $headers
if (-not $dashboard.vehicles -or -not $dashboard.warnings) {
    throw "Dashboard did not return expected sections"
}
Write-Host "[OK] dashboard"

$profile = Invoke-RestMethod -Uri "$baseUrl/api/v1/vehicles/$($vehicle.id)/profile" -Headers $headers
if (-not $profile.vehicle -or -not $profile.operations_summary) {
    throw "Vehicle profile did not return expected sections"
}
Write-Host "[OK] vehicle profile"

$search = Invoke-RestMethod -Uri "$baseUrl/api/v1/search?q=$($vehicle.plate)" -Headers $headers
if ($search.Count -lt 1) {
    throw "Global search did not return vehicle result"
}
Write-Host "[OK] global search"

$healthDetails = Invoke-RestMethod -Uri "$baseUrl/health/details"
if ($healthDetails.database -ne "ok") {
    throw "Health details did not report database ok"
}
Write-Host "[OK] health details"

$aiCapabilities = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/capabilities" -Headers $headers
if ($aiCapabilities.Count -lt 1) {
    throw "AI capabilities did not return rows"
}
if (-not ($aiCapabilities | Where-Object { $_.analysis_type -eq "support_ticket_triage" })) {
    throw "AI capabilities does not include support_ticket_triage"
}
Write-Host "[OK] ai capabilities"

$aiProvider = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/provider" -Headers $headers
if (-not $aiProvider.provider) {
    throw "AI provider status did not return provider"
}
Write-Host "[OK] ai provider"

$aiPrompts = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/prompt-templates" -Headers $headers
if ($aiPrompts.Count -lt 1) {
    throw "AI prompt templates did not return rows"
}
if (-not ($aiPrompts | Where-Object { $_.analysis_type -eq "support_sla_risk" })) {
    throw "AI prompt templates does not include support_sla_risk"
}
Write-Host "[OK] ai prompt templates"

$aiRecommendations = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/recommendations" -Headers $headers
if ($null -eq $aiRecommendations) {
    throw "AI recommendations did not return a response"
}
Write-Host "[OK] ai recommendations"

$aiContext = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/vehicles/$($vehicle.id)/context" -Headers $headers
if (-not $aiContext.vehicle) {
    throw "AI vehicle context did not return vehicle context"
}
Write-Host "[OK] ai vehicle context"

$aiJob = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/ai/vehicles/$($vehicle.id)/analyze" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        analysis_type = "vehicle_risk_summary"
        requires_human_approval = $true
        note = "Smoke AI analysis"
    } | ConvertTo-Json)
if (-not $aiJob.id) {
    throw "AI vehicle analysis did not create a job"
}
Write-Host "[OK] ai vehicle analysis job id=$($aiJob.id)"

$aiUpdated = Invoke-RestMethod `
    -Method Patch `
    -Uri "$baseUrl/api/v1/ai/jobs/$($aiJob.id)" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        job_status = "completed"
        result_data = @{
            summary = "Smoke AI result"
            recommended_actions = @("review_vehicle")
        }
        confidence_score = 0.91
    } | ConvertTo-Json -Depth 5)
if ($aiUpdated.job_status -ne "completed") {
    throw "AI job update failed"
}
Write-Host "[OK] ai job update"

$aiTaskDraft = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/jobs/$($aiJob.id)/task-draft" -Headers $headers
if (-not $aiTaskDraft.description) {
    throw "AI task draft did not return description"
}
Write-Host "[OK] ai task draft"

$aiNotificationDraft = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/jobs/$($aiJob.id)/notification-draft" -Headers $headers
if (-not $aiNotificationDraft.message) {
    throw "AI notification draft did not return message"
}
Write-Host "[OK] ai notification draft"

$notificationProvider = Invoke-RestMethod -Uri "$baseUrl/api/v1/notifications/provider-status" -Headers $headers
if ($null -eq $notificationProvider.whatsapp) {
    throw "Notification provider status did not return whatsapp section"
}
Write-Host "[OK] notification provider status"

$whatsappNotification = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/notifications" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        notification_type = "smoke_whatsapp"
        related_vehicle_id = $vehicle.id
        message = "Smoke WhatsApp dispatch"
        sent_via = "whatsapp"
        recipient_phone = "+905551112233"
    } | ConvertTo-Json)
Write-Host "[OK] whatsapp notification id=$($whatsappNotification.id)"

$whatsappDispatch = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/notifications/$($whatsappNotification.id)/dispatch" `
    -Headers $headers
if (-not $whatsappDispatch.delivery_status) {
    throw "WhatsApp dispatch did not return delivery status"
}
Write-Host "[OK] whatsapp dispatch status=$($whatsappDispatch.delivery_status)"

$supportTicket = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/support/tickets" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        title = "Smoke IT destek talebi"
        description = "Smoke test destek talebi"
        priority = "high"
        source_channel = "web"
        reporter_phone = "+905551112233"
    } | ConvertTo-Json)
if (-not $supportTicket.ticket_no) {
    throw "Support ticket did not return ticket number"
}
if (-not $supportTicket.sla_response_due_at -or -not $supportTicket.sla_resolution_due_at -or -not $supportTicket.sla_status) {
    throw "Support ticket did not return SLA fields"
}
Write-Host "[OK] support ticket id=$($supportTicket.id)"

$supportAttachmentPath = Join-Path ([System.IO.Path]::GetTempPath()) "tuluklar-support-smoke.txt"
Set-Content -LiteralPath $supportAttachmentPath -Value "Smoke IT destek eki" -Encoding UTF8
try {
    $supportAttachment = Invoke-RestMethod `
        -Method Post `
        -Uri "$baseUrl/api/v1/files/upload" `
        -Headers $headers `
        -Form @{
            module_name = "support_tickets"
            entity_id = "$($supportTicket.id)"
            file_type = "support_document"
            note = "Smoke IT destek talebi eki"
            file = Get-Item -LiteralPath $supportAttachmentPath
        }
    if ($supportAttachment.module_name -ne "support_tickets" -or $supportAttachment.entity_id -ne $supportTicket.id) {
        throw "Support ticket attachment upload failed"
    }
    Write-Host "[OK] support ticket attachment id=$($supportAttachment.id)"
} finally {
    if (Test-Path $supportAttachmentPath) {
        Remove-Item -LiteralPath $supportAttachmentPath -Force
    }
}

$supportSummary = Invoke-RestMethod -Uri "$baseUrl/api/v1/support/summary" -Headers $headers
if ($supportSummary.total_count -lt 1 -or $supportSummary.open_count -lt 1) {
    throw "Support summary did not return expected counts"
}
Write-Host "[OK] support summary"

$knowledgeBase = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/support/knowledge-base" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        title = "Smoke destek cozum kaydi"
        category = "it_support"
        problem = "Smoke test destek problemi"
        solution = "Smoke test cozum adimlari"
        tags = @("smoke", "it", "destek")
        source_ticket_id = $supportTicket.id
        is_published = $true
    } | ConvertTo-Json -Depth 5)
if (-not $knowledgeBase.id -or $knowledgeBase.source_ticket_id -ne $supportTicket.id) {
    throw "Support knowledge base create failed"
}
Write-Host "[OK] support knowledge base id=$($knowledgeBase.id)"

$knowledgeAttachmentPath = Join-Path ([System.IO.Path]::GetTempPath()) "tuluklar-knowledge-smoke.txt"
Set-Content -LiteralPath $knowledgeAttachmentPath -Value "Smoke IT bilgi bankasi eki" -Encoding UTF8
try {
    $knowledgeAttachment = Invoke-RestMethod `
        -Method Post `
        -Uri "$baseUrl/api/v1/files/upload" `
        -Headers $headers `
        -Form @{
            module_name = "support_knowledge_base"
            entity_id = "$($knowledgeBase.id)"
            file_type = "knowledge_attachment"
            note = "Smoke IT bilgi bankasi eki"
            file = Get-Item -LiteralPath $knowledgeAttachmentPath
        }
    if ($knowledgeAttachment.module_name -ne "support_knowledge_base" -or $knowledgeAttachment.entity_id -ne $knowledgeBase.id) {
        throw "Support knowledge attachment upload failed"
    }
    Write-Host "[OK] support knowledge attachment id=$($knowledgeAttachment.id)"
} finally {
    if (Test-Path $knowledgeAttachmentPath) {
        Remove-Item -LiteralPath $knowledgeAttachmentPath -Force
    }
}

$knowledgeList = Invoke-RestMethod -Uri "$baseUrl/api/v1/support/knowledge-base?q=Smoke&limit=5" -Headers $headers
if ($knowledgeList.Count -lt 1) {
    throw "Support knowledge base list did not return rows"
}
Write-Host "[OK] support knowledge base list"

$knowledgeUpdate = Invoke-RestMethod `
    -Method Patch `
    -Uri "$baseUrl/api/v1/support/knowledge-base/$($knowledgeBase.id)" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        solution = "Smoke test cozum adimlari guncellendi"
        is_published = $true
    } | ConvertTo-Json)
if ($knowledgeUpdate.solution -notlike "*guncellendi*") {
    throw "Support knowledge base update failed"
}
Write-Host "[OK] support knowledge base update"

$knowledgeSearch = Invoke-RestMethod -Uri "$baseUrl/api/v1/search?q=cozum" -Headers $headers
if (-not ($knowledgeSearch | Where-Object { $_.result_type -eq "support_knowledge" })) {
    throw "Global search did not return support knowledge result"
}
Write-Host "[OK] support knowledge global search"

$supportAiContext = Invoke-RestMethod -Uri "$baseUrl/api/v1/ai/support/tickets/$($supportTicket.id)/context" -Headers $headers
if (-not $supportAiContext.ticket -or -not $supportAiContext.sla -or -not $supportAiContext.events) {
    throw "AI support ticket context did not return expected sections"
}
Write-Host "[OK] ai support ticket context"

$supportAiJob = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/ai/support/tickets/$($supportTicket.id)/analyze" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        analysis_type = "support_ticket_triage"
        requires_human_approval = $true
        note = "Smoke support AI analysis"
    } | ConvertTo-Json)
if (-not $supportAiJob.id -or $supportAiJob.source_table -ne "support_tickets") {
    throw "AI support ticket analysis did not create expected job"
}
Write-Host "[OK] ai support ticket analysis job id=$($supportAiJob.id)"

$supportReplyDraftJob = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/ai/support/tickets/$($supportTicket.id)/analyze" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        analysis_type = "support_reply_draft"
        requires_human_approval = $true
        note = "Smoke support AI reply draft"
    } | ConvertTo-Json)
if (-not $supportReplyDraftJob.id -or $supportReplyDraftJob.analysis_type -ne "support_reply_draft") {
    throw "AI support reply draft did not create expected job"
}
Write-Host "[OK] ai support reply draft job id=$($supportReplyDraftJob.id)"

$supportReply = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/support/tickets/$($supportTicket.id)/reply" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        message = "Smoke destek talebiniz uzerinde islem baslatildi."
        next_status = "waiting_user"
        event_note = "Smoke WhatsApp yaniti"
    } | ConvertTo-Json)
if ($supportReply.ticket.ticket_status -ne "waiting_user" -or $supportReply.notification.sent_via -ne "whatsapp") {
    throw "Support ticket WhatsApp reply failed"
}
Write-Host "[OK] support ticket whatsapp reply notification id=$($supportReply.notification.id)"

$supportUpdate = Invoke-RestMethod `
    -Method Patch `
    -Uri "$baseUrl/api/v1/support/tickets/$($supportTicket.id)" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        ticket_status = "resolved"
        resolution_note = "Smoke destek talebi cozuldu"
        satisfaction_score = 5
        satisfaction_note = "Smoke memnuniyet kontrolu"
        event_note = "Smoke update"
    } | ConvertTo-Json)
if ($supportUpdate.ticket_status -ne "resolved") {
    throw "Support ticket update failed"
}
if ($supportUpdate.sla_status -ne "closed" -or $supportUpdate.satisfaction_score -ne 5) {
    throw "Support ticket SLA/satisfaction update failed"
}
Write-Host "[OK] support ticket update"

$supportEvents = Invoke-RestMethod -Uri "$baseUrl/api/v1/support/tickets/$($supportTicket.id)/events" -Headers $headers
if ($supportEvents.Count -lt 1) {
    throw "Support ticket events did not return rows"
}
Write-Host "[OK] support ticket events"

$whatsappWebhookResult = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/notifications/whatsapp/webhook" `
    -ContentType "application/json" `
    -Body (@{
        object = "whatsapp_business_account"
        entry = @(@{
            changes = @(@{
                value = @{
                    contacts = @(@{ profile = @{ name = "Smoke Kullanici" }; wa_id = "905551112233" })
                    messages = @(@{ id = "wamid.smoke-support-$([guid]::NewGuid())"; from = "905551112233"; text = @{ body = "Bilgisayarim acilmiyor, destek rica ederim." } })
                }
            })
        })
    } | ConvertTo-Json -Depth 8)
if ($whatsappWebhookResult.support_ticket_ids.Count -lt 1) {
    throw "WhatsApp webhook did not create support ticket"
}
Write-Host "[OK] whatsapp webhook support ticket"

$audit = Invoke-RestMethod -Uri "$baseUrl/api/v1/logs/audit?limit=5" -Headers $headers
if ($audit.Count -lt 1) {
    throw "Audit log did not return rows"
}
Write-Host "[OK] audit"

$filteredAudit = Invoke-RestMethod -Uri "$baseUrl/api/v1/logs/audit?table_name=vehicles&action_type=create&limit=5" -Headers $headers
if ($filteredAudit.Count -lt 1) {
    throw "Filtered audit log did not return rows"
}
Write-Host "[OK] audit filters"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/damages/$($damage.id)" -Headers $headers | Out-Null
Write-Host "[OK] damage delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/operations/insurance-quotes/$($quote.id)" -Headers $headers | Out-Null
Write-Host "[OK] insurance quote delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/operations/vehicle-washes/$($wash.id)" -Headers $headers | Out-Null
Write-Host "[OK] vehicle wash delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/operations/fuel-entries/$($fuel.id)" -Headers $headers | Out-Null
Write-Host "[OK] fuel entry delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/operations/value-loss-claims/$($valueLoss.id)" -Headers $headers | Out-Null
Write-Host "[OK] value loss claim delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/operations/vehicle-inspections/$($inspection.id)" -Headers $headers | Out-Null
Write-Host "[OK] vehicle inspection delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/expenses/$($expense.id)" -Headers $headers | Out-Null
Write-Host "[OK] expense delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/insurance-policies/$($policy.id)" -Headers $headers | Out-Null
Write-Host "[OK] policy delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/maintenances/$($maintenance.id)" -Headers $headers | Out-Null
Write-Host "[OK] maintenance delete/archive"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/vehicles/$($vehicle.id)" -Headers $headers | Out-Null
Write-Host "[OK] vehicle delete/archive"

Write-Host "Smoke test passed"
