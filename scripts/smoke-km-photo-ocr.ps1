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

Write-Host "Smoke test KM photo OCR: $baseUrl"

$login = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/auth/login" `
    -ContentType "application/json" `
    -Body (@{
        email = $email
        password = $password
    } | ConvertTo-Json)

$headers = @{ Authorization = "Bearer $($login.access_token)" }
$stamp = Get-Date -Format "yyyyMMddHHmmss"

$company = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/organization/companies" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        name = "Smoke KM OCR Company $stamp"
        tax_number = "KMOCR$stamp"
    } | ConvertTo-Json)
Write-Host "[OK] company id=$($company.id)"

$department = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/organization/departments" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        company_id = $company.id
        name = "KM OCR $stamp"
    } | ConvertTo-Json)
Write-Host "[OK] department id=$($department.id)"

$vehicle = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/vehicles" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        plate = "34OCR$($stamp.Substring(8,6))"
        brand = "Ford"
        model = "Transit"
        model_year = 2025
        vehicle_type = "van"
        fuel_type = "diesel"
        company_id = $company.id
        department_id = $department.id
    } | ConvertTo-Json)
Write-Host "[OK] vehicle id=$($vehicle.id)"

$kmPhotoPath = Join-Path ([System.IO.Path]::GetTempPath()) "tuluklar-km-ocr-smoke.jpg"
[byte[]]$jpegBytes = 0xFF,0xD8,0xFF,0xD9
[System.IO.File]::WriteAllBytes($kmPhotoPath, $jpegBytes)

try {
    $file = Invoke-RestMethod `
        -Method Post `
        -Uri "$baseUrl/api/v1/files/upload" `
        -Headers $headers `
        -Form @{
            module_name = "vehicles"
            entity_id = "$($vehicle.id)"
            file_type = "km_photo"
            note = "Smoke KM OCR fotografi"
            file = Get-Item -LiteralPath $kmPhotoPath
        }
    if ($file.module_name -ne "vehicles" -or $file.file_type -ne "km_photo" -or $file.entity_id -ne $vehicle.id) {
        throw "KM photo upload returned unexpected payload"
    }
    Write-Host "[OK] km photo file id=$($file.id)"

    $kmLog = Invoke-RestMethod `
        -Method Post `
        -Uri "$baseUrl/api/v1/tracking/km-logs" `
        -Headers $headers `
        -ContentType "application/json" `
        -Body (@{
            vehicle_id = $vehicle.id
            km = 43210
            entry_type = "ocr"
            image_path = $file.storage_path
            ocr_result = @{
                status = "pending"
                source = "smoke_km_photo"
                file_id = $file.id
                submitted_km = 43210
            }
            device_info = "smoke:km-photo"
        } | ConvertTo-Json -Depth 5)
    if ($kmLog.entry_type -ne "ocr" -or -not $kmLog.image_path) {
        throw "OCR KM log did not return expected fields"
    }
    Write-Host "[OK] ocr km log id=$($kmLog.id)"

    $aiJobs = Invoke-RestMethod `
        -Uri "$baseUrl/api/v1/ai/jobs?analysis_type=ocr_verification&related_vehicle_id=$($vehicle.id)&limit=5" `
        -Headers $headers
    $job = $aiJobs | Where-Object { $_.source_table -eq "km_logs" -and $_.source_record_id -eq $kmLog.id } | Select-Object -First 1
    if (-not $job) {
        throw "OCR verification AI job was not created for KM log"
    }
    Write-Host "[OK] ocr ai job id=$($job.id)"
} finally {
    if (Test-Path $kmPhotoPath) {
        Remove-Item -LiteralPath $kmPhotoPath -Force
    }
}

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/vehicles/$($vehicle.id)" -Headers $headers | Out-Null
Write-Host "[OK] vehicle delete/archive"

Write-Host "KM photo OCR smoke test passed"
